//! Request timeout middleware
//!
//! Automatically cancels requests that exceed a specified duration.

use crate::{Error, Request, Response, Result, middleware::Middleware};
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use tokio::time::timeout;

/// Timeout middleware configuration
#[derive(Debug, Clone)]
pub struct TimeoutConfig {
    /// Maximum request duration
    pub duration: Duration,
    /// Custom timeout error message
    pub message: Option<String>,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            duration: Duration::from_secs(30),
            message: None,
        }
    }
}

impl TimeoutConfig {
    /// Create new timeout configuration
    pub fn new(duration: Duration) -> Self {
        Self {
            duration,
            message: None,
        }
    }

    /// Set custom timeout message
    pub fn message(mut self, msg: impl Into<String>) -> Self {
        self.message = Some(msg.into());
        self
    }
}

/// Timeout middleware
///
/// Cancels requests that take longer than the configured duration.
pub struct Timeout {
    config: TimeoutConfig,
}

impl Timeout {
    /// Create new timeout middleware with default config (30 seconds)
    pub fn new() -> Self {
        Self {
            config: TimeoutConfig::default(),
        }
    }

    /// Create timeout middleware with custom duration
    pub fn with_duration(duration: Duration) -> Self {
        Self {
            config: TimeoutConfig::new(duration),
        }
    }

    /// Create timeout middleware with custom config
    pub fn with_config(config: TimeoutConfig) -> Self {
        Self { config }
    }

    /// Get configuration
    pub fn config(&self) -> &TimeoutConfig {
        &self.config
    }
}

impl Default for Timeout {
    fn default() -> Self {
        Self::new()
    }
}

impl Middleware for Timeout {
    fn handle(
        &self,
        req: Request,
        next: crate::middleware::Next,
    ) -> Pin<Box<dyn Future<Output = Result<Response>> + Send + '_>> {
        let duration = self.config.duration;
        let message = self.config.message.clone();

        Box::pin(async move {
            match timeout(duration, next(req)).await {
                Ok(result) => result,
                Err(_) => {
                    let msg =
                        message.unwrap_or_else(|| format!("Request timeout after {:?}", duration));
                    Err(Error::InternalError(msg))
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeout_config_default() {
        let config = TimeoutConfig::default();
        assert_eq!(config.duration, Duration::from_secs(30));
        assert!(config.message.is_none());
    }

    #[test]
    fn test_timeout_config_builder() {
        let config = TimeoutConfig::new(Duration::from_secs(10)).message("Custom timeout message");

        assert_eq!(config.duration, Duration::from_secs(10));
        assert_eq!(config.message, Some("Custom timeout message".to_string()));
    }

    #[test]
    fn test_timeout_creation() {
        let timeout = Timeout::new();
        assert_eq!(timeout.config().duration, Duration::from_secs(30));

        let timeout = Timeout::with_duration(Duration::from_secs(5));
        assert_eq!(timeout.config().duration, Duration::from_secs(5));
    }

    #[test]
    fn test_timeout_default() {
        let timeout = Timeout::default();
        assert_eq!(timeout.config().duration, Duration::from_secs(30));
    }
}
