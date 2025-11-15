//! Rate limiting middleware
//!
//! Provides flexible rate limiting with multiple strategies.

use crate::{
    error::Result,
    middleware::{Middleware, Next},
    Request, Response,
};
use governor::{
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter as GovernorRateLimiter,
};
use std::future::Future;
use std::num::NonZeroU32;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

/// Rate limit configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum number of requests
    pub max_requests: u32,
    /// Time window
    pub window: Duration,
}

impl RateLimitConfig {
    /// Create a new rate limit configuration
    pub fn new(max_requests: u32, window: Duration) -> Self {
        Self { max_requests, window }
    }

    /// Per second rate limit
    pub fn per_second(requests: u32) -> Self {
        Self::new(requests, Duration::from_secs(1))
    }

    /// Per minute rate limit
    pub fn per_minute(requests: u32) -> Self {
        Self::new(requests, Duration::from_secs(60))
    }

    /// Per hour rate limit
    pub fn per_hour(requests: u32) -> Self {
        Self::new(requests, Duration::from_secs(3600))
    }
}

/// Rate limiting middleware
///
/// Limits the number of requests based on configured quota.
/// Uses token bucket algorithm for smooth rate limiting.
///
/// # Features
/// - Token bucket algorithm
/// - Configurable time windows
/// - Automatic retry-after header
/// - Path exemptions
///
/// # Example
/// ```no_run
/// use half_core::middleware::{RateLimiter, RateLimitConfig};
///
/// let limiter = RateLimiter::new(RateLimitConfig::per_minute(100))
///     .exempt("/health");
///
/// // Use with your router
/// // router.use_middleware(limiter);
/// ```
pub struct RateLimiter {
    limiter: Arc<GovernorRateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
    exempt_paths: Vec<String>,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(config: RateLimitConfig) -> Self {
        let quota = Quota::with_period(config.window)
            .expect("Invalid duration")
            .allow_burst(NonZeroU32::new(config.max_requests).expect("Max requests must be > 0"));

        let limiter = Arc::new(GovernorRateLimiter::direct(quota));

        Self {
            limiter,
            exempt_paths: Vec::new(),
        }
    }

    /// Add a path to exempt from rate limiting
    pub fn exempt(mut self, path: impl Into<String>) -> Self {
        self.exempt_paths.push(path.into());
        self
    }

    /// Check if a path is exempt from rate limiting
    fn is_exempt(&self, path: &str) -> bool {
        self.exempt_paths.iter().any(|exempt| path.starts_with(exempt))
    }
}

impl Middleware for RateLimiter {
    fn handle(
        &self,
        req: Request,
        next: Next,
    ) -> Pin<Box<dyn Future<Output = Result<Response>> + Send + '_>> {
        Box::pin(async move {
            let path = req.path().to_string();

            // Check if path is exempt
            if self.is_exempt(&path) {
                return next(req).await;
            }

            // Check rate limit
            match self.limiter.check() {
                Ok(_) => {
                    // Request allowed
                    next(req).await
                }
                Err(_not_until) => {
                    // Rate limit exceeded
                    let mut response = Response::json(&serde_json::json!({
                        "error": "Rate limit exceeded",
                        "status": 429
                    }))?;

                    response = response.status(hyper::StatusCode::TOO_MANY_REQUESTS);
                    response = response.header_str("retry-after", "60");

                    Ok(response)
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_config() {
        let config = RateLimitConfig::per_second(10);
        assert_eq!(config.max_requests, 10);
        assert_eq!(config.window, Duration::from_secs(1));

        let config = RateLimitConfig::per_minute(100);
        assert_eq!(config.max_requests, 100);
        assert_eq!(config.window, Duration::from_secs(60));

        let config = RateLimitConfig::per_hour(1000);
        assert_eq!(config.max_requests, 1000);
        assert_eq!(config.window, Duration::from_secs(3600));
    }

    #[test]
    fn test_rate_limiter_exempt() {
        let limiter = RateLimiter::new(RateLimitConfig::per_second(10))
            .exempt("/health")
            .exempt("/metrics");

        assert!(limiter.is_exempt("/health"));
        assert!(limiter.is_exempt("/health/live"));
        assert!(limiter.is_exempt("/metrics"));
        assert!(!limiter.is_exempt("/api/users"));
    }
}
