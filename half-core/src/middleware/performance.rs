//! Performance monitoring middleware
//!
//! Tracks request performance metrics such as duration, sizes, and throughput.

use crate::{middleware::Middleware, Request, Response, Result};
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// Performance monitoring middleware configuration
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    /// Slow request threshold in milliseconds
    pub slow_threshold_ms: u64,
    /// Enable request logging
    pub log_requests: bool,
    /// Log only slow requests
    pub log_only_slow: bool,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            slow_threshold_ms: 1000, // 1 second
            log_requests: true,
            log_only_slow: false,
        }
    }
}

impl PerformanceConfig {
    /// Create new performance configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set slow request threshold in milliseconds
    pub fn slow_threshold_ms(mut self, threshold: u64) -> Self {
        self.slow_threshold_ms = threshold;
        self
    }

    /// Enable/disable request logging
    pub fn log_requests(mut self, enable: bool) -> Self {
        self.log_requests = enable;
        self
    }

    /// Log only slow requests
    pub fn log_only_slow(mut self, enable: bool) -> Self {
        self.log_only_slow = enable;
        self
    }
}

/// Performance statistics
#[derive(Debug)]
pub struct PerformanceStats {
    total_requests: AtomicU64,
    slow_requests: AtomicU64,
    total_duration_ms: AtomicU64,
    total_request_bytes: AtomicU64,
    total_response_bytes: AtomicU64,
}

impl PerformanceStats {
    /// Create new performance statistics
    pub fn new() -> Self {
        Self {
            total_requests: AtomicU64::new(0),
            slow_requests: AtomicU64::new(0),
            total_duration_ms: AtomicU64::new(0),
            total_request_bytes: AtomicU64::new(0),
            total_response_bytes: AtomicU64::new(0),
        }
    }

    /// Get total number of requests
    pub fn total_requests(&self) -> u64 {
        self.total_requests.load(Ordering::Relaxed)
    }

    /// Get number of slow requests
    pub fn slow_requests(&self) -> u64 {
        self.slow_requests.load(Ordering::Relaxed)
    }

    /// Get average request duration in milliseconds
    pub fn avg_duration_ms(&self) -> f64 {
        let total = self.total_requests();
        if total == 0 {
            return 0.0;
        }
        self.total_duration_ms.load(Ordering::Relaxed) as f64 / total as f64
    }

    /// Get total request bytes
    pub fn total_request_bytes(&self) -> u64 {
        self.total_request_bytes.load(Ordering::Relaxed)
    }

    /// Get total response bytes
    pub fn total_response_bytes(&self) -> u64 {
        self.total_response_bytes.load(Ordering::Relaxed)
    }

    /// Get average request size in bytes
    pub fn avg_request_size(&self) -> f64 {
        let total = self.total_requests();
        if total == 0 {
            return 0.0;
        }
        self.total_request_bytes() as f64 / total as f64
    }

    /// Get average response size in bytes
    pub fn avg_response_size(&self) -> f64 {
        let total = self.total_requests();
        if total == 0 {
            return 0.0;
        }
        self.total_response_bytes() as f64 / total as f64
    }

    /// Record a request
    fn record(
        &self,
        duration_ms: u64,
        is_slow: bool,
        request_bytes: usize,
        response_bytes: usize,
    ) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.total_duration_ms.fetch_add(duration_ms, Ordering::Relaxed);
        self.total_request_bytes.fetch_add(request_bytes as u64, Ordering::Relaxed);
        self.total_response_bytes.fetch_add(response_bytes as u64, Ordering::Relaxed);

        if is_slow {
            self.slow_requests.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Reset statistics
    pub fn reset(&self) {
        self.total_requests.store(0, Ordering::Relaxed);
        self.slow_requests.store(0, Ordering::Relaxed);
        self.total_duration_ms.store(0, Ordering::Relaxed);
        self.total_request_bytes.store(0, Ordering::Relaxed);
        self.total_response_bytes.store(0, Ordering::Relaxed);
    }
}

impl Default for PerformanceStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance monitoring middleware
///
/// Tracks request performance metrics and logs slow requests.
pub struct PerformanceMonitor {
    config: PerformanceConfig,
    stats: Arc<PerformanceStats>,
}

impl PerformanceMonitor {
    /// Create new performance monitor with default config
    pub fn new() -> Self {
        Self {
            config: PerformanceConfig::default(),
            stats: Arc::new(PerformanceStats::new()),
        }
    }

    /// Create performance monitor with custom config
    pub fn with_config(config: PerformanceConfig) -> Self {
        Self {
            config,
            stats: Arc::new(PerformanceStats::new()),
        }
    }

    /// Get performance statistics
    pub fn stats(&self) -> &Arc<PerformanceStats> {
        &self.stats
    }

    /// Get configuration
    pub fn config(&self) -> &PerformanceConfig {
        &self.config
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl Middleware for PerformanceMonitor {
    fn handle(
        &self,
        req: Request,
        next: crate::middleware::Next,
    ) -> Pin<Box<dyn Future<Output = Result<Response>> + Send + '_>> {
        let config = self.config.clone();
        let stats = self.stats.clone();

        Box::pin(async move {
            let method = req.method().clone();
            let path = req.path().to_string();
            let request_size = req.body_bytes().map(|b| b.len()).unwrap_or(0);

            // Start timing
            let start = Instant::now();

            // Execute request
            let result = next(req).await;

            // Calculate duration
            let duration = start.elapsed();
            let duration_ms = duration.as_millis() as u64;

            // Check if slow
            let is_slow = duration_ms >= config.slow_threshold_ms;

            // Get response size
            let response_size = result.as_ref()
                .map(|r| r.get_body().len())
                .unwrap_or(0);

            // Record statistics
            stats.record(duration_ms, is_slow, request_size, response_size);

            // Log if enabled
            if config.log_requests && (!config.log_only_slow || is_slow) {
                let slow_marker = if is_slow { " [SLOW]" } else { "" };
                eprintln!(
                    "[PERF]{} {} {} - {}ms (req: {} bytes, res: {} bytes)",
                    slow_marker,
                    method.as_str(),
                    path,
                    duration_ms,
                    request_size,
                    response_size
                );
            }

            result
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_config_default() {
        let config = PerformanceConfig::default();
        assert_eq!(config.slow_threshold_ms, 1000);
        assert!(config.log_requests);
        assert!(!config.log_only_slow);
    }

    #[test]
    fn test_performance_config_builder() {
        let config = PerformanceConfig::new()
            .slow_threshold_ms(500)
            .log_requests(false)
            .log_only_slow(true);

        assert_eq!(config.slow_threshold_ms, 500);
        assert!(!config.log_requests);
        assert!(config.log_only_slow);
    }

    #[test]
    fn test_performance_stats() {
        let stats = PerformanceStats::new();

        assert_eq!(stats.total_requests(), 0);
        assert_eq!(stats.slow_requests(), 0);
        assert_eq!(stats.avg_duration_ms(), 0.0);

        // Record some requests
        stats.record(100, false, 1000, 2000);
        stats.record(2000, true, 1500, 3000);

        assert_eq!(stats.total_requests(), 2);
        assert_eq!(stats.slow_requests(), 1);
        assert_eq!(stats.avg_duration_ms(), 1050.0);
        assert_eq!(stats.total_request_bytes(), 2500);
        assert_eq!(stats.total_response_bytes(), 5000);
        assert_eq!(stats.avg_request_size(), 1250.0);
        assert_eq!(stats.avg_response_size(), 2500.0);
    }

    #[test]
    fn test_performance_stats_reset() {
        let stats = PerformanceStats::new();

        stats.record(100, false, 1000, 2000);
        assert_eq!(stats.total_requests(), 1);

        stats.reset();
        assert_eq!(stats.total_requests(), 0);
        assert_eq!(stats.avg_duration_ms(), 0.0);
    }

    #[test]
    fn test_performance_monitor_creation() {
        let monitor = PerformanceMonitor::new();
        assert_eq!(monitor.config().slow_threshold_ms, 1000);
        assert_eq!(monitor.stats().total_requests(), 0);
    }

    #[test]
    fn test_performance_monitor_with_config() {
        let config = PerformanceConfig::new().slow_threshold_ms(250);
        let monitor = PerformanceMonitor::with_config(config);
        assert_eq!(monitor.config().slow_threshold_ms, 250);
    }
}
