//! Error recovery middleware
//!
//! Catches panics and errors, converts them to appropriate HTTP responses.

use crate::{middleware::Middleware, Request, Response, Result};
use futures_util::FutureExt;
use hyper::StatusCode;
use std::future::Future;
use std::panic::AssertUnwindSafe;
use std::pin::Pin;

/// Recovery mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryMode {
    /// Development mode - Show detailed error messages
    Development,
    /// Production mode - Show generic error messages
    Production,
}

/// Recovery middleware configuration
#[derive(Debug, Clone)]
pub struct RecoveryConfig {
    /// Recovery mode
    pub mode: RecoveryMode,
    /// Custom 500 error message for production
    pub production_message: String,
    /// Enable error logging
    pub log_errors: bool,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            mode: RecoveryMode::Production,
            production_message: "Internal Server Error".to_string(),
            log_errors: true,
        }
    }
}

impl RecoveryConfig {
    /// Create new recovery configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set recovery mode
    pub fn mode(mut self, mode: RecoveryMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set production error message
    pub fn production_message(mut self, msg: impl Into<String>) -> Self {
        self.production_message = msg.into();
        self
    }

    /// Enable/disable error logging
    pub fn log_errors(mut self, enable: bool) -> Self {
        self.log_errors = enable;
        self
    }
}

/// Recovery middleware
///
/// Catches panics and errors, converting them to appropriate HTTP responses.
pub struct Recovery {
    config: RecoveryConfig,
}

impl Recovery {
    /// Create new recovery middleware with default config
    pub fn new() -> Self {
        Self {
            config: RecoveryConfig::default(),
        }
    }

    /// Create recovery middleware for development
    pub fn development() -> Self {
        Self {
            config: RecoveryConfig::new().mode(RecoveryMode::Development),
        }
    }

    /// Create recovery middleware for production
    pub fn production() -> Self {
        Self {
            config: RecoveryConfig::new().mode(RecoveryMode::Production),
        }
    }

    /// Create recovery middleware with custom config
    pub fn with_config(config: RecoveryConfig) -> Self {
        Self { config }
    }

    /// Get configuration
    pub fn config(&self) -> &RecoveryConfig {
        &self.config
    }
}

impl Default for Recovery {
    fn default() -> Self {
        Self::new()
    }
}

impl Middleware for Recovery {
    fn handle(
        &self,
        req: Request,
        next: crate::middleware::Next,
    ) -> Pin<Box<dyn Future<Output = Result<Response>> + Send + '_>> {
        let config = self.config.clone();

        Box::pin(async move {
            // Catch panics
            let result = AssertUnwindSafe(next(req))
                .catch_unwind()
                .await;

            match result {
                Ok(response_result) => {
                    // Handle normal errors
                    match response_result {
                        Ok(response) => Ok(response),
                        Err(err) => {
                            if config.log_errors {
                                eprintln!("Error: {:?}", err);
                            }

                            let error_msg = match config.mode {
                                RecoveryMode::Development => format!("{}", err),
                                RecoveryMode::Production => config.production_message,
                            };

                            Ok(Response::new()
                                .status(StatusCode::INTERNAL_SERVER_ERROR)
                                .body(error_msg.into_bytes()))
                        }
                    }
                }
                Err(_panic) => {
                    // Handle panic
                    if config.log_errors {
                        eprintln!("Panic occurred in request handler");
                    }

                    let error_msg = match config.mode {
                        RecoveryMode::Development => {
                            "Panic occurred in request handler".to_string()
                        }
                        RecoveryMode::Production => config.production_message,
                    };

                    Ok(Response::new()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(error_msg.into_bytes()))
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recovery_config_default() {
        let config = RecoveryConfig::default();
        assert_eq!(config.mode, RecoveryMode::Production);
        assert_eq!(config.production_message, "Internal Server Error");
        assert!(config.log_errors);
    }

    #[test]
    fn test_recovery_config_builder() {
        let config = RecoveryConfig::new()
            .mode(RecoveryMode::Development)
            .production_message("Custom error")
            .log_errors(false);

        assert_eq!(config.mode, RecoveryMode::Development);
        assert_eq!(config.production_message, "Custom error");
        assert!(!config.log_errors);
    }

    #[test]
    fn test_recovery_creation() {
        let recovery = Recovery::new();
        assert_eq!(recovery.config().mode, RecoveryMode::Production);

        let recovery = Recovery::development();
        assert_eq!(recovery.config().mode, RecoveryMode::Development);

        let recovery = Recovery::production();
        assert_eq!(recovery.config().mode, RecoveryMode::Production);
    }

    #[test]
    fn test_recovery_modes() {
        assert_eq!(RecoveryMode::Development, RecoveryMode::Development);
        assert_ne!(RecoveryMode::Development, RecoveryMode::Production);
    }
}
