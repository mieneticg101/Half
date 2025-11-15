//! Response handling for Half framework
//!
//! Provides a builder API for constructing HTTP responses with security features.

use crate::error::Error;
use bytes::Bytes;
use hyper::{
    header::{HeaderMap, HeaderName, HeaderValue, CONTENT_TYPE, SET_COOKIE},
    StatusCode,
};
use http_body_util::Full;
use serde::Serialize;

/// HTTP Response builder
///
/// Provides a fluent API for building responses with automatic security headers.
#[derive(Clone)]
pub struct Response {
    pub(crate) status: StatusCode,
    pub(crate) headers: HeaderMap,
    pub(crate) body: Bytes,
}

impl Response {
    /// Create a new response with 200 OK status
    pub fn new() -> Self {
        let mut response = Self {
            status: StatusCode::OK,
            headers: HeaderMap::new(),
            body: Bytes::new(),
        };
        response.apply_security_headers();
        response
    }

    /// Apply security headers to the response
    ///
    /// Adds important security headers to prevent common attacks:
    /// - X-Content-Type-Options: nosniff
    /// - X-Frame-Options: DENY
    /// - Referrer-Policy: strict-origin-when-cross-origin
    /// - Permissions-Policy: restricts browser features
    fn apply_security_headers(&mut self) {
        // Prevent MIME sniffing
        self.headers.insert(
            HeaderName::from_static("x-content-type-options"),
            HeaderValue::from_static("nosniff"),
        );

        // Prevent clickjacking
        self.headers.insert(
            HeaderName::from_static("x-frame-options"),
            HeaderValue::from_static("DENY"),
        );

        // Control referrer information
        self.headers.insert(
            HeaderName::from_static("referrer-policy"),
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        );

        // Restrict browser features
        self.headers.insert(
            HeaderName::from_static("permissions-policy"),
            HeaderValue::from_static("geolocation=(), microphone=(), camera=()"),
        );

        // Cross-domain policy
        self.headers.insert(
            HeaderName::from_static("x-permitted-cross-domain-policies"),
            HeaderValue::from_static("none"),
        );
    }

    /// Set response status code
    pub fn status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }

    /// Set response body from string
    pub fn body(mut self, body: impl Into<Bytes>) -> Self {
        self.body = body.into();
        self
    }

    /// Set a header
    pub fn header(mut self, key: HeaderName, value: HeaderValue) -> Self {
        self.headers.insert(key, value);
        self
    }

    /// Set a header with string key and value (convenience method)
    pub fn header_str(mut self, key: &str, value: &str) -> Self {
        if let Ok(header_name) = HeaderName::from_bytes(key.as_bytes()) {
            if let Ok(header_value) = HeaderValue::from_str(value) {
                self.headers.insert(header_name, header_value);
            }
        }
        self
    }

    /// Set Content-Type header
    pub fn content_type(self, content_type: &str) -> Self {
        self.header(
            CONTENT_TYPE,
            HeaderValue::from_str(content_type).unwrap_or_else(|_| {
                HeaderValue::from_static("text/plain")
            }),
        )
    }

    /// Add a cookie to the response
    pub fn cookie(mut self, cookie: Cookie) -> Self {
        if let Ok(value) = HeaderValue::from_str(&cookie.to_string()) {
            self.headers.append(SET_COOKIE, value);
        }
        self
    }

    /// Create a JSON response
    ///
    /// # Security
    /// Automatically sets appropriate Content-Type and prevents MIME sniffing
    pub fn json<T: Serialize>(value: &T) -> Result<Self, Error> {
        let json = serde_json::to_string(value)?;
        Ok(Self::new()
            .content_type("application/json; charset=utf-8")
            .header(
                HeaderName::from_static("x-content-type-options"),
                HeaderValue::from_static("nosniff"),
            )
            .body(json))
    }

    /// Create an HTML response
    ///
    /// # Security
    /// Automatically escapes content to prevent XSS attacks
    pub fn html(content: &str) -> Self {
        let escaped = htmlescape::encode_minimal(content);
        Self::new()
            .content_type("text/html; charset=utf-8")
            .header(
                HeaderName::from_static("x-content-type-options"),
                HeaderValue::from_static("nosniff"),
            )
            .header(
                HeaderName::from_static("x-frame-options"),
                HeaderValue::from_static("DENY"),
            )
            .body(escaped)
    }

    /// Create a plain text response
    pub fn text(content: impl Into<String>) -> Self {
        Self::new()
            .content_type("text/plain; charset=utf-8")
            .body(content.into())
    }

    /// Create a redirect response
    pub fn redirect(location: &str, permanent: bool) -> Self {
        let status = if permanent {
            StatusCode::MOVED_PERMANENTLY
        } else {
            StatusCode::FOUND
        };

        Self::new()
            .status(status)
            .header(
                HeaderName::from_static("location"),
                HeaderValue::from_str(location)
                    .unwrap_or_else(|_| HeaderValue::from_static("/")),
            )
    }

    /// Create a 404 Not Found response
    pub fn not_found() -> Self {
        Self::text("Not Found").status(StatusCode::NOT_FOUND)
    }

    /// Create a 500 Internal Server Error response
    pub fn internal_error() -> Self {
        Self::text("Internal Server Error").status(StatusCode::INTERNAL_SERVER_ERROR)
    }

    /// Create an error response from an Error
    ///
    /// # Security
    /// In production, server errors (5xx) only show generic messages to prevent
    /// information leakage. Client errors (4xx) show the actual error message.
    pub fn from_error(error: &Error) -> Self {
        let status = StatusCode::from_u16(error.status_code())
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        // Sanitize error message for production
        let error_message = if error.is_server_error() {
            // For 5xx errors, don't leak internal details
            "Internal Server Error".to_string()
        } else {
            // For 4xx errors, show the actual error
            error.to_string()
        };

        // Try to create JSON response
        match Self::json(&ErrorResponse {
            error: error_message.clone(),
            status: error.status_code(),
        }) {
            Ok(mut response) => {
                response.status = status;
                response
            }
            Err(_) => {
                // Fallback to plain text
                Self::text(error_message).status(status)
            }
        }
    }

    /// Get a reference to the response body
    pub fn get_body(&self) -> &Bytes {
        &self.body
    }

    /// Get a reference to the response headers
    pub fn get_headers(&self) -> &HeaderMap {
        &self.headers
    }

    /// Get a mutable reference to the response headers
    pub fn get_headers_mut(&mut self) -> &mut HeaderMap {
        &mut self.headers
    }

    /// Get the response status code
    pub fn get_status(&self) -> StatusCode {
        self.status
    }

    /// Convert to Hyper response
    pub fn into_hyper(self) -> hyper::Response<Full<Bytes>> {
        let mut response = hyper::Response::new(Full::new(self.body));
        *response.status_mut() = self.status;
        *response.headers_mut() = self.headers;
        response
    }
}

impl Default for Response {
    fn default() -> Self {
        Self::new()
    }
}

/// Cookie builder
#[derive(Debug, Clone)]
pub struct Cookie {
    name: String,
    value: String,
    path: Option<String>,
    domain: Option<String>,
    max_age: Option<i64>,
    secure: bool,
    http_only: bool,
    same_site: Option<SameSite>,
}

impl Cookie {
    /// Create a new cookie
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            path: None,
            domain: None,
            max_age: None,
            secure: true, // Secure by default
            http_only: true, // HttpOnly by default for security
            same_site: Some(SameSite::Strict), // Strict by default for CSRF protection
        }
    }

    /// Set cookie path
    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// Set cookie domain
    pub fn domain(mut self, domain: impl Into<String>) -> Self {
        self.domain = Some(domain.into());
        self
    }

    /// Set max age in seconds
    pub fn max_age(mut self, seconds: i64) -> Self {
        self.max_age = Some(seconds);
        self
    }

    /// Set secure flag
    pub fn secure(mut self, secure: bool) -> Self {
        self.secure = secure;
        self
    }

    /// Set HttpOnly flag
    pub fn http_only(mut self, http_only: bool) -> Self {
        self.http_only = http_only;
        self
    }

    /// Set SameSite attribute
    pub fn same_site(mut self, same_site: SameSite) -> Self {
        self.same_site = Some(same_site);
        self
    }
}

impl std::fmt::Display for Cookie {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}={}", self.name, self.value)?;

        if let Some(path) = &self.path {
            write!(f, "; Path={}", path)?;
        }

        if let Some(domain) = &self.domain {
            write!(f, "; Domain={}", domain)?;
        }

        if let Some(max_age) = self.max_age {
            write!(f, "; Max-Age={}", max_age)?;
        }

        if self.secure {
            write!(f, "; Secure")?;
        }

        if self.http_only {
            write!(f, "; HttpOnly")?;
        }

        if let Some(same_site) = &self.same_site {
            write!(f, "; SameSite={}", same_site)?;
        }

        Ok(())
    }
}

/// SameSite cookie attribute
#[derive(Debug, Clone, Copy)]
pub enum SameSite {
    Strict,
    Lax,
    None,
}

impl std::fmt::Display for SameSite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SameSite::Strict => write!(f, "Strict"),
            SameSite::Lax => write!(f, "Lax"),
            SameSite::None => write!(f, "None"),
        }
    }
}

/// Error response JSON structure
#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    status: u16,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_builder() {
        let response = Response::new()
            .status(StatusCode::OK)
            .content_type("text/plain")
            .body("Hello");

        assert_eq!(response.status, StatusCode::OK);
        assert_eq!(response.body, "Hello");
    }

    #[test]
    fn test_cookie_to_string() {
        let cookie = Cookie::new("session", "abc123")
            .path("/")
            .max_age(3600);

        let cookie_str = cookie.to_string();
        assert!(cookie_str.contains("session=abc123"));
        assert!(cookie_str.contains("Path=/"));
        assert!(cookie_str.contains("Max-Age=3600"));
        assert!(cookie_str.contains("Secure"));
        assert!(cookie_str.contains("HttpOnly"));
        assert!(cookie_str.contains("SameSite=Strict"));
    }

    #[test]
    fn test_json_response() {
        #[derive(Serialize)]
        struct TestData {
            message: String,
        }

        let data = TestData {
            message: "Hello".to_string(),
        };

        let response = Response::json(&data).unwrap();
        assert_eq!(response.status, StatusCode::OK);

        let body_str = String::from_utf8(response.body.to_vec()).unwrap();
        assert!(body_str.contains("Hello"));
    }
}
