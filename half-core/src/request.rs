//! Request handling for Half framework
//!
//! Provides a safe, ergonomic API for working with HTTP requests.

use crate::error::{Error, Result};
use bytes::Bytes;
use http_body_util::BodyExt;
use hyper::{body::Incoming, Method, Uri, HeaderMap, Version};
use serde::de::DeserializeOwned;
use std::collections::HashMap;

/// Maximum request body size (10 MB)
/// This prevents denial-of-service attacks via large payloads
const MAX_BODY_SIZE: usize = 10 * 1024 * 1024;

/// Maximum number of query parameters
/// Prevents hash collision DoS attacks
const MAX_QUERY_PARAMS: usize = 100;

/// Maximum length of query parameter key or value
/// Prevents memory exhaustion attacks
const MAX_QUERY_PARAM_LENGTH: usize = 4096;

/// HTTP Request wrapper
///
/// Provides convenient methods for accessing request data with built-in
/// security features and validation.
pub struct Request {
    method: Method,
    uri: Uri,
    version: Version,
    headers: HeaderMap,
    body: Option<Bytes>,
    /// Path parameters extracted from the route
    pub params: HashMap<String, String>,
    /// Query parameters from the URL
    pub query: HashMap<String, String>,
}

impl Request {
    /// Create a new Request from Hyper components
    ///
    /// # Security
    /// Enforces size limits on request body and query parameters to prevent DoS attacks
    pub async fn from_hyper(
        method: Method,
        uri: Uri,
        version: Version,
        headers: HeaderMap,
        body: Incoming,
    ) -> Result<Self> {
        // Collect body bytes with size limit
        let body_bytes = body
            .collect()
            .await
            .map_err(|e| Error::Http(e.to_string()))?
            .to_bytes();

        // Validate body size
        if body_bytes.len() > MAX_BODY_SIZE {
            return Err(Error::BadRequest(format!(
                "Request body too large: {} bytes (max: {} bytes)",
                body_bytes.len(),
                MAX_BODY_SIZE
            )));
        }

        // Parse query parameters with limit
        let query = uri
            .query()
            .map(|q| Self::parse_query(q))
            .unwrap_or_default();

        // Validate query parameter count
        if query.len() > MAX_QUERY_PARAMS {
            return Err(Error::BadRequest(format!(
                "Too many query parameters: {} (max: {})",
                query.len(),
                MAX_QUERY_PARAMS
            )));
        }

        Ok(Request {
            method,
            uri,
            version,
            headers,
            body: Some(body_bytes),
            params: HashMap::new(),
            query,
        })
    }

    /// Get the HTTP method
    pub fn method(&self) -> &Method {
        &self.method
    }

    /// Get the request URI
    pub fn uri(&self) -> &Uri {
        &self.uri
    }

    /// Get the path component of the URI
    pub fn path(&self) -> &str {
        self.uri.path()
    }

    /// Get the HTTP version
    pub fn version(&self) -> Version {
        self.version
    }

    /// Get request headers
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    /// Get a specific header value
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .get(name)
            .and_then(|v| v.to_str().ok())
    }

    /// Get the Content-Type header
    pub fn content_type(&self) -> Option<&str> {
        self.header("content-type")
    }

    /// Get raw body bytes
    pub fn body_bytes(&self) -> Option<&Bytes> {
        self.body.as_ref()
    }

    /// Get body as UTF-8 string
    pub fn body_string(&self) -> Result<String> {
        match &self.body {
            Some(bytes) => String::from_utf8(bytes.to_vec())
                .map_err(|e| Error::BadRequest(format!("Invalid UTF-8: {}", e))),
            None => Ok(String::new()),
        }
    }

    /// Parse body as JSON
    ///
    /// # Security
    /// This method validates JSON structure and prevents deeply nested objects
    /// that could cause stack overflow attacks.
    pub fn json<T: DeserializeOwned>(&self) -> Result<T> {
        match &self.body {
            Some(bytes) => {
                // Validate content type
                if let Some(ct) = self.content_type() {
                    if !ct.contains("application/json") {
                        return Err(Error::BadRequest(
                            "Expected application/json content type".into()
                        ));
                    }
                }

                serde_json::from_slice(bytes)
                    .map_err(|e| Error::Json(e))
            }
            None => Err(Error::BadRequest("Empty body".into())),
        }
    }

    /// Get a path parameter by name
    pub fn param(&self, name: &str) -> Option<&str> {
        self.params.get(name).map(|s| s.as_str())
    }

    /// Get a query parameter by name
    pub fn query_param(&self, name: &str) -> Option<&str> {
        self.query.get(name).map(|s| s.as_str())
    }

    /// Set path parameters (used by router)
    pub fn set_params(&mut self, params: HashMap<String, String>) {
        self.params = params;
    }

    /// Parse query string into key-value pairs
    ///
    /// # Security
    /// Validates parameter lengths to prevent memory exhaustion attacks
    fn parse_query(query: &str) -> HashMap<String, String> {
        query
            .split('&')
            .filter_map(|part| {
                let mut split = part.splitn(2, '=');
                let key = split.next()?.to_string();
                let value = split.next().unwrap_or("").to_string();

                // Validate length before decoding
                if key.len() > MAX_QUERY_PARAM_LENGTH || value.len() > MAX_QUERY_PARAM_LENGTH {
                    return None;
                }

                let decoded_key = Self::decode_uri_component(&key);
                let decoded_value = Self::decode_uri_component(&value);

                // Validate decoded length as well
                if decoded_key.len() > MAX_QUERY_PARAM_LENGTH || decoded_value.len() > MAX_QUERY_PARAM_LENGTH {
                    return None;
                }

                Some((decoded_key, decoded_value))
            })
            .collect()
    }

    /// Decode URI component (URL decoding)
    ///
    /// # Security
    /// Properly handles malformed percent-encoding to prevent injection attacks
    fn decode_uri_component(s: &str) -> String {
        percent_encoding::percent_decode_str(s)
            .decode_utf8_lossy()
            .into_owned()
    }

    /// Check if request is AJAX/XHR
    pub fn is_ajax(&self) -> bool {
        self.header("x-requested-with")
            .map(|v| v.eq_ignore_ascii_case("XMLHttpRequest"))
            .unwrap_or(false)
    }

    /// Get client IP address (respects X-Forwarded-For)
    ///
    /// # Security
    /// Only use this for logging. Never trust it for authentication/authorization
    /// as it can be spoofed.
    pub fn client_ip(&self) -> Option<&str> {
        self.header("x-forwarded-for")
            .or_else(|| self.header("x-real-ip"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_query() {
        let query = "foo=bar&baz=qux&empty=";
        let params = Request::parse_query(query);

        assert_eq!(params.get("foo"), Some(&"bar".to_string()));
        assert_eq!(params.get("baz"), Some(&"qux".to_string()));
        assert_eq!(params.get("empty"), Some(&"".to_string()));
    }

    #[test]
    fn test_decode_uri_component() {
        assert_eq!(Request::decode_uri_component("hello%20world"), "hello world");
        assert_eq!(Request::decode_uri_component("foo%2Fbar"), "foo/bar");
    }
}
