use std::time::Instant;
use tracing::{Level, Span};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

/// Logging configuration
#[derive(Debug, Clone)]
pub struct LogConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: Level,
    /// Enable JSON output
    pub json: bool,
    /// Enable request/response logging
    pub log_requests: bool,
    /// Enable performance metrics
    pub log_metrics: bool,
    /// Environment filter (e.g., "half=debug,tokio=info")
    pub env_filter: Option<String>,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: Level::INFO,
            json: false,
            log_requests: true,
            log_metrics: true,
            env_filter: None,
        }
    }
}

impl LogConfig {
    /// Create a new log configuration with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Set log level
    pub fn level(mut self, level: Level) -> Self {
        self.level = level;
        self
    }

    /// Enable JSON output
    pub fn json(mut self, json: bool) -> Self {
        self.json = json;
        self
    }

    /// Enable request logging
    pub fn log_requests(mut self, log_requests: bool) -> Self {
        self.log_requests = log_requests;
        self
    }

    /// Enable metrics logging
    pub fn log_metrics(mut self, log_metrics: bool) -> Self {
        self.log_metrics = log_metrics;
        self
    }

    /// Set environment filter
    pub fn env_filter(mut self, filter: impl Into<String>) -> Self {
        self.env_filter = Some(filter.into());
        self
    }

    /// Initialize the global tracing subscriber
    pub fn init(&self) -> Result<(), String> {
        let filter = if let Some(ref env_filter) = self.env_filter {
            EnvFilter::try_new(env_filter)
                .map_err(|e| format!("Invalid env filter: {}", e))?
        } else {
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(self.level.as_str()))
        };

        if self.json {
            tracing_subscriber::registry()
                .with(filter)
                .with(fmt::layer().json().with_span_events(FmtSpan::NEW | FmtSpan::CLOSE))
                .try_init()
                .map_err(|e| format!("Failed to initialize logger: {}", e))?;
        } else {
            tracing_subscriber::registry()
                .with(filter)
                .with(fmt::layer().with_span_events(FmtSpan::NEW | FmtSpan::CLOSE))
                .try_init()
                .map_err(|e| format!("Failed to initialize logger: {}", e))?;
        }

        Ok(())
    }
}

/// Request logging middleware
#[derive(Debug, Clone)]
pub struct RequestLogger {
    config: LogConfig,
}

impl RequestLogger {
    /// Create a new request logger
    pub fn new(config: LogConfig) -> Self {
        Self { config }
    }

    /// Log request start
    pub fn log_request(
        &self,
        method: &str,
        path: &str,
        request_id: Option<&str>,
    ) -> Option<(Span, Instant)> {
        if !self.config.log_requests {
            return None;
        }

        let span = if let Some(req_id) = request_id {
            tracing::info_span!(
                "request",
                method = %method,
                path = %path,
                request_id = %req_id
            )
        } else {
            tracing::info_span!(
                "request",
                method = %method,
                path = %path
            )
        };

        let _enter = span.enter();
        tracing::info!("Request started");
        drop(_enter);

        Some((span, Instant::now()))
    }

    /// Log request completion
    pub fn log_response(
        &self,
        span: Span,
        start: Instant,
        status: u16,
        size: usize,
    ) {
        if !self.config.log_requests {
            return;
        }

        let duration = start.elapsed();
        let _enter = span.enter();

        tracing::info!(
            status = %status,
            duration_ms = %duration.as_millis(),
            size_bytes = %size,
            "Request completed"
        );
    }
}

/// Performance metrics logger
pub struct MetricsLogger {
    enabled: bool,
}

impl MetricsLogger {
    /// Create a new metrics logger
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    /// Log a metric
    pub fn log(&self, name: &str, value: f64, unit: &str) {
        if !self.enabled {
            return;
        }

        tracing::info!(
            metric = %name,
            value = %value,
            unit = %unit,
            "Metric"
        );
    }

    /// Log a counter increment
    pub fn increment(&self, name: &str, value: u64) {
        if !self.enabled {
            return;
        }

        tracing::info!(
            metric = %name,
            value = %value,
            type = "counter",
            "Counter increment"
        );
    }

    /// Log a gauge value
    pub fn gauge(&self, name: &str, value: f64) {
        if !self.enabled {
            return;
        }

        tracing::info!(
            metric = %name,
            value = %value,
            type = "gauge",
            "Gauge value"
        );
    }

    /// Log a histogram value
    pub fn histogram(&self, name: &str, value: f64) {
        if !self.enabled {
            return;
        }

        tracing::info!(
            metric = %name,
            value = %value,
            type = "histogram",
            "Histogram value"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_config_default() {
        let config = LogConfig::default();
        assert_eq!(config.level, Level::INFO);
        assert!(!config.json);
        assert!(config.log_requests);
        assert!(config.log_metrics);
        assert!(config.env_filter.is_none());
    }

    #[test]
    fn test_log_config_builder() {
        let config = LogConfig::new()
            .level(Level::DEBUG)
            .json(true)
            .log_requests(false)
            .log_metrics(false)
            .env_filter("half=trace");

        assert_eq!(config.level, Level::DEBUG);
        assert!(config.json);
        assert!(!config.log_requests);
        assert!(!config.log_metrics);
        assert_eq!(config.env_filter.unwrap(), "half=trace");
    }

    #[test]
    fn test_request_logger_creation() {
        let config = LogConfig::default();
        let logger = RequestLogger::new(config);
        assert!(logger.config.log_requests);
    }

    #[test]
    fn test_metrics_logger() {
        let logger = MetricsLogger::new(true);
        assert!(logger.enabled);

        let logger_disabled = MetricsLogger::new(false);
        assert!(!logger_disabled.enabled);
    }

    #[test]
    fn test_metrics_logger_methods() {
        let logger = MetricsLogger::new(true);

        // These should not panic
        logger.log("test_metric", 42.0, "ms");
        logger.increment("test_counter", 1);
        logger.gauge("test_gauge", 75.5);
        logger.histogram("test_histogram", 123.45);
    }
}
