//! Error types for Half framework
//!
//! Provides comprehensive error handling with detailed error messages
//! and proper HTTP status code mapping.

use thiserror::Error;

/// Result type alias for Half operations
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for Half framework
#[derive(Error, Debug)]
pub enum Error {
    /// HTTP parsing or protocol errors
    #[error("HTTP error: {0}")]
    Http(String),

    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization/deserialization errors
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Route not found
    #[error("Route not found: {method} {path}")]
    NotFound { method: String, path: String },

    /// Method not allowed
    #[error("Method not allowed: {method} for {path}")]
    MethodNotAllowed { method: String, path: String },

    /// Bad request (client error)
    #[error("Bad request: {0}")]
    BadRequest(String),

    /// Unauthorized access
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    /// Forbidden access
    #[error("Forbidden: {0}")]
    Forbidden(String),

    /// Internal server error
    #[error("Internal server error: {0}")]
    InternalError(String),

    /// Invalid CSRF token
    #[error("Invalid CSRF token")]
    InvalidCsrfToken,

    /// Invalid nonce (replay protection)
    #[error("Invalid nonce: {0}")]
    InvalidNonce(String),

    /// Invalid input (validation error)
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Custom error with message
    #[error("{0}")]
    Custom(String),
}

impl Error {
    /// Get the HTTP status code for this error
    pub fn status_code(&self) -> u16 {
        match self {
            Error::NotFound { .. } => 404,
            Error::MethodNotAllowed { .. } => 405,
            Error::BadRequest(_) => 400,
            Error::Unauthorized(_) => 401,
            Error::Forbidden(_) | Error::InvalidCsrfToken | Error::InvalidNonce(_) => 403,
            Error::ValidationError(_) => 422,
            Error::Http(_) | Error::Io(_) | Error::Json(_) | Error::InternalError(_) => 500,
            Error::Custom(_) => 500,
        }
    }

    /// Check if this is a client error (4xx)
    pub fn is_client_error(&self) -> bool {
        let code = self.status_code();
        (400..500).contains(&code)
    }

    /// Check if this is a server error (5xx)
    pub fn is_server_error(&self) -> bool {
        let code = self.status_code();
        code >= 500
    }
}

/// Convert hyper errors to Half errors
impl From<hyper::Error> for Error {
    fn from(err: hyper::Error) -> Self {
        Error::Http(err.to_string())
    }
}

/// Convert hyper HTTP errors to Half errors
impl From<hyper::http::Error> for Error {
    fn from(err: hyper::http::Error) -> Self {
        Error::Http(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_status_codes() {
        assert_eq!(
            Error::NotFound {
                method: "GET".into(),
                path: "/".into()
            }
            .status_code(),
            404
        );
        assert_eq!(Error::BadRequest("test".into()).status_code(), 400);
        assert_eq!(Error::Unauthorized("test".into()).status_code(), 401);
        assert_eq!(Error::Forbidden("test".into()).status_code(), 403);
        assert_eq!(Error::InternalError("test".into()).status_code(), 500);
    }

    #[test]
    fn test_error_categories() {
        let client_err = Error::BadRequest("test".into());
        assert!(client_err.is_client_error());
        assert!(!client_err.is_server_error());

        let server_err = Error::InternalError("test".into());
        assert!(!server_err.is_client_error());
        assert!(server_err.is_server_error());
    }
}
