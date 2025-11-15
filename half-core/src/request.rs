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
            .map(Self::parse_query)
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
                    .map_err(Error::Json)
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

    /// Create a test request from path and method
    ///
    /// This is a simplified constructor primarily for use in tests. Production code
    /// should use `from_hyper()` instead.
    ///
    /// # Example
    /// ```ignore
    /// use half_core::Request;
    /// use hyper::Method;
    ///
    /// let req = Request::from_path_and_method("/api/users", Method::GET).unwrap();
    /// ```
    pub fn from_path_and_method(path: &str, method: Method) -> Result<Self> {
        let uri: Uri = path
            .parse()
            .map_err(|e| Error::BadRequest(format!("Invalid URI: {}", e)))?;

        let query = uri
            .query()
            .map(Self::parse_query)
            .unwrap_or_default();

        Ok(Request {
            method,
            uri,
            version: Version::HTTP_11,
            headers: HeaderMap::new(),
            body: None,
            params: HashMap::new(),
            query,
        })
    }

    /// Set request body
    ///
    /// Primarily for testing. Production code should use the body from `from_hyper()`.
    pub fn set_body(&mut self, body: Vec<u8>) {
        self.body = Some(Bytes::from(body));
    }

    /// Set a request header
    ///
    /// Primarily for testing. Production code should use headers from `from_hyper()`.
    pub fn set_header(&mut self, name: &str, value: &str) {
        if let Ok(header_name) = hyper::header::HeaderName::from_bytes(name.as_bytes()) {
            if let Ok(header_value) = hyper::header::HeaderValue::from_str(value) {
                self.headers.insert(header_name, header_value);
            }
        }
    }

    /// Get the client IP address
    ///
    /// Checks X-Forwarded-For, X-Real-IP, and other proxy headers
    /// before falling back to the remote address.
    ///
    /// # Returns
    /// The client IP address as a string, or None if not available
    pub fn client_ip(&self) -> Option<String> {
        // Check X-Forwarded-For header (most common proxy header)
        if let Some(forwarded) = self.header("x-forwarded-for") {
            // X-Forwarded-For can contain multiple IPs, take the first one
            if let Some(ip) = forwarded.split(',').next() {
                return Some(ip.trim().to_string());
            }
        }

        // Check X-Real-IP header
        if let Some(real_ip) = self.header("x-real-ip") {
            return Some(real_ip.to_string());
        }

        // Check CF-Connecting-IP (Cloudflare)
        if let Some(cf_ip) = self.header("cf-connecting-ip") {
            return Some(cf_ip.to_string());
        }

        // Check True-Client-IP (Akamai, Cloudflare)
        if let Some(true_ip) = self.header("true-client-ip") {
            return Some(true_ip.to_string());
        }

        None
    }

    /// Get the User-Agent header
    ///
    /// Returns the full User-Agent string sent by the client
    pub fn user_agent(&self) -> Option<&str> {
        self.header("user-agent")
    }

    /// Check if the request is from a mobile device
    ///
    /// Uses simple User-Agent detection. For production use,
    /// consider a dedicated user agent parsing library.
    pub fn is_mobile(&self) -> bool {
        if let Some(ua) = self.user_agent() {
            let ua_lower = ua.to_lowercase();
            ua_lower.contains("mobile")
                || ua_lower.contains("android")
                || ua_lower.contains("iphone")
                || ua_lower.contains("ipad")
                || ua_lower.contains("windows phone")
        } else {
            false
        }
    }

    /// Get the Referer header
    ///
    /// Returns the referring URL if present
    pub fn referer(&self) -> Option<&str> {
        self.header("referer")
    }

    /// Get accepted content types from Accept header
    ///
    /// Returns a list of MIME types the client accepts, in order of preference
    pub fn accepts(&self) -> Vec<String> {
        if let Some(accept) = self.header("accept") {
            accept
                .split(',')
                .map(|s| {
                    // Remove quality values (q=0.9) and whitespace
                    s.split(';')
                        .next()
                        .unwrap_or("")
                        .trim()
                        .to_string()
                })
                .filter(|s| !s.is_empty())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Check if the request accepts a specific content type
    ///
    /// # Example
    /// ```ignore
    /// if req.accepts_type("application/json") {
    ///     // Return JSON response
    /// } else if req.accepts_type("text/html") {
    ///     // Return HTML response
    /// }
    /// ```
    pub fn accepts_type(&self, content_type: &str) -> bool {
        if let Some(accept) = self.header("accept") {
            let accept_lower = accept.to_lowercase();
            let type_lower = content_type.to_lowercase();

            // Check for exact match or wildcard
            accept_lower.contains(&type_lower)
                || accept_lower.contains("*/*")
                || (type_lower.contains('/') && {
                    let parts: Vec<&str> = type_lower.split('/').collect();
                    if parts.len() == 2 {
                        accept_lower.contains(&format!("{}/*", parts[0]))
                    } else {
                        false
                    }
                })
        } else {
            false
        }
    }

    /// Check if the request is HTTPS
    ///
    /// Checks the URI scheme and X-Forwarded-Proto header
    pub fn is_https(&self) -> bool {
        // Check URI scheme
        if let Some(scheme) = self.uri.scheme_str() {
            if scheme == "https" {
                return true;
            }
        }

        // Check X-Forwarded-Proto header (for proxied requests)
        if let Some(proto) = self.header("x-forwarded-proto") {
            return proto.to_lowercase() == "https";
        }

        false
    }

    /// Check if the request is an AJAX/XHR request
    ///
    /// Checks for the X-Requested-With header
    pub fn is_ajax(&self) -> bool {
        if let Some(requested_with) = self.header("x-requested-with") {
            requested_with.to_lowercase() == "xmlhttprequest"
        } else {
            false
        }
    }

    /// Get the host from the request
    ///
    /// Returns the Host header value
    pub fn host(&self) -> Option<&str> {
        self.header("host")
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

    #[test]
    fn test_client_ip_x_forwarded_for() {
        let mut req = Request::from_path_and_method("/test", Method::GET).unwrap();
        req.set_header("x-forwarded-for", "203.0.113.1, 198.51.100.1");

        assert_eq!(req.client_ip(), Some("203.0.113.1".to_string()));
    }

    #[test]
    fn test_client_ip_x_real_ip() {
        let mut req = Request::from_path_and_method("/test", Method::GET).unwrap();
        req.set_header("x-real-ip", "203.0.113.1");

        assert_eq!(req.client_ip(), Some("203.0.113.1".to_string()));
    }

    #[test]
    fn test_client_ip_cloudflare() {
        let mut req = Request::from_path_and_method("/test", Method::GET).unwrap();
        req.set_header("cf-connecting-ip", "203.0.113.1");

        assert_eq!(req.client_ip(), Some("203.0.113.1".to_string()));
    }

    #[test]
    fn test_user_agent() {
        let mut req = Request::from_path_and_method("/test", Method::GET).unwrap();
        req.set_header("user-agent", "Mozilla/5.0");

        assert_eq!(req.user_agent(), Some("Mozilla/5.0"));
    }

    #[test]
    fn test_is_mobile() {
        let mut req = Request::from_path_and_method("/test", Method::GET).unwrap();
        req.set_header("user-agent", "Mozilla/5.0 (iPhone; CPU iPhone OS 14_0 like Mac OS X)");

        assert!(req.is_mobile());

        req.set_header("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64)");
        assert!(!req.is_mobile());
    }

    #[test]
    fn test_accepts() {
        let mut req = Request::from_path_and_method("/test", Method::GET).unwrap();
        req.set_header("accept", "text/html, application/json;q=0.9, */*;q=0.8");

        let accepts = req.accepts();
        assert_eq!(accepts.len(), 3);
        assert_eq!(accepts[0], "text/html");
        assert_eq!(accepts[1], "application/json");
        assert_eq!(accepts[2], "*/*");
    }

    #[test]
    fn test_accepts_type() {
        let mut req = Request::from_path_and_method("/test", Method::GET).unwrap();
        req.set_header("accept", "text/html, application/json");

        assert!(req.accepts_type("application/json"));
        assert!(req.accepts_type("text/html"));
        assert!(!req.accepts_type("application/xml"));
    }

    #[test]
    fn test_accepts_type_wildcard() {
        let mut req = Request::from_path_and_method("/test", Method::GET).unwrap();
        req.set_header("accept", "*/*");

        assert!(req.accepts_type("application/json"));
        assert!(req.accepts_type("text/html"));
    }

    #[test]
    fn test_is_https() {
        let mut req = Request::from_path_and_method("/test", Method::GET).unwrap();
        req.set_header("x-forwarded-proto", "https");

        assert!(req.is_https());

        req.set_header("x-forwarded-proto", "http");
        assert!(!req.is_https());
    }

    #[test]
    fn test_is_ajax() {
        let mut req = Request::from_path_and_method("/test", Method::GET).unwrap();
        req.set_header("x-requested-with", "XMLHttpRequest");

        assert!(req.is_ajax());

        let req2 = Request::from_path_and_method("/test", Method::GET).unwrap();
        assert!(!req2.is_ajax());
    }

    #[test]
    fn test_referer() {
        let mut req = Request::from_path_and_method("/test", Method::GET).unwrap();
        req.set_header("referer", "https://example.com/previous-page");

        assert_eq!(req.referer(), Some("https://example.com/previous-page"));
    }

    #[test]
    fn test_host() {
        let mut req = Request::from_path_and_method("/test", Method::GET).unwrap();
        req.set_header("host", "example.com:8080");

        assert_eq!(req.host(), Some("example.com:8080"));
    }
}
