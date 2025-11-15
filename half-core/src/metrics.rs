//! Metrics collection and monitoring
//!
//! Provides Prometheus-compatible metrics for monitoring application performance.

use crate::Response;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// Metric type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricType {
    /// Counter - monotonically increasing value
    Counter,
    /// Gauge - value that can go up or down
    Gauge,
    /// Histogram - distribution of values
    Histogram,
}

/// Counter metric
#[derive(Debug)]
pub struct Counter {
    value: AtomicU64,
}

impl Counter {
    /// Create a new counter
    pub fn new() -> Self {
        Self {
            value: AtomicU64::new(0),
        }
    }

    /// Increment the counter by 1
    pub fn inc(&self) {
        self.add(1);
    }

    /// Add a value to the counter
    pub fn add(&self, value: u64) {
        self.value.fetch_add(value, Ordering::Relaxed);
    }

    /// Get the current value
    pub fn get(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }

    /// Reset the counter to 0
    pub fn reset(&self) {
        self.value.store(0, Ordering::Relaxed);
    }
}

impl Default for Counter {
    fn default() -> Self {
        Self::new()
    }
}

/// Gauge metric
#[derive(Debug)]
pub struct Gauge {
    value: Arc<RwLock<f64>>,
}

impl Gauge {
    /// Create a new gauge
    pub fn new() -> Self {
        Self {
            value: Arc::new(RwLock::new(0.0)),
        }
    }

    /// Set the gauge value
    pub fn set(&self, value: f64) {
        *self.value.write() = value;
    }

    /// Increment the gauge
    pub fn inc(&self) {
        self.add(1.0);
    }

    /// Decrement the gauge
    pub fn dec(&self) {
        self.sub(1.0);
    }

    /// Add to the gauge
    pub fn add(&self, value: f64) {
        *self.value.write() += value;
    }

    /// Subtract from the gauge
    pub fn sub(&self, value: f64) {
        *self.value.write() -= value;
    }

    /// Get the current value
    pub fn get(&self) -> f64 {
        *self.value.read()
    }
}

impl Default for Gauge {
    fn default() -> Self {
        Self::new()
    }
}

/// Histogram metric for tracking distributions
#[derive(Debug)]
pub struct Histogram {
    sum: Arc<RwLock<f64>>,
    count: AtomicU64,
    #[allow(dead_code)] // Reserved for future bucket implementation
    buckets: Arc<RwLock<Vec<f64>>>,
}

impl Histogram {
    /// Create a new histogram with default buckets
    pub fn new() -> Self {
        Self::with_buckets(vec![
            0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
        ])
    }

    /// Create a histogram with custom buckets
    pub fn with_buckets(buckets: Vec<f64>) -> Self {
        Self {
            sum: Arc::new(RwLock::new(0.0)),
            count: AtomicU64::new(0),
            buckets: Arc::new(RwLock::new(buckets)),
        }
    }

    /// Observe a value
    pub fn observe(&self, value: f64) {
        *self.sum.write() += value;
        self.count.fetch_add(1, Ordering::Relaxed);
    }

    /// Get the sum of all observations
    pub fn sum(&self) -> f64 {
        *self.sum.read()
    }

    /// Get the count of observations
    pub fn count(&self) -> u64 {
        self.count.load(Ordering::Relaxed)
    }

    /// Get the average value
    pub fn average(&self) -> f64 {
        let count = self.count();
        if count == 0 {
            0.0
        } else {
            self.sum() / count as f64
        }
    }
}

impl Default for Histogram {
    fn default() -> Self {
        Self::new()
    }
}

/// Metrics registry
///
/// Central registry for all application metrics.
///
/// # Example
/// ```
/// use half_core::metrics::Metrics;
///
/// let metrics = Metrics::new();
///
/// // Register and use a counter
/// metrics.counter("http_requests_total").inc();
///
/// // Register and use a gauge
/// metrics.gauge("active_connections").set(42.0);
///
/// // Observe histogram values
/// metrics.histogram("request_duration_seconds").observe(0.125);
/// ```
pub struct Metrics {
    counters: Arc<RwLock<HashMap<String, Arc<Counter>>>>,
    gauges: Arc<RwLock<HashMap<String, Arc<Gauge>>>>,
    histograms: Arc<RwLock<HashMap<String, Arc<Histogram>>>>,
    start_time: Instant,
}

impl Metrics {
    /// Create a new metrics registry
    pub fn new() -> Self {
        Self {
            counters: Arc::new(RwLock::new(HashMap::new())),
            gauges: Arc::new(RwLock::new(HashMap::new())),
            histograms: Arc::new(RwLock::new(HashMap::new())),
            start_time: Instant::now(),
        }
    }

    /// Get or create a counter
    pub fn counter(&self, name: impl Into<String>) -> Arc<Counter> {
        let name = name.into();
        let mut counters = self.counters.write();
        counters
            .entry(name)
            .or_insert_with(|| Arc::new(Counter::new()))
            .clone()
    }

    /// Get or create a gauge
    pub fn gauge(&self, name: impl Into<String>) -> Arc<Gauge> {
        let name = name.into();
        let mut gauges = self.gauges.write();
        gauges
            .entry(name)
            .or_insert_with(|| Arc::new(Gauge::new()))
            .clone()
    }

    /// Get or create a histogram
    pub fn histogram(&self, name: impl Into<String>) -> Arc<Histogram> {
        let name = name.into();
        let mut histograms = self.histograms.write();
        histograms
            .entry(name)
            .or_insert_with(|| Arc::new(Histogram::new()))
            .clone()
    }

    /// Export metrics in Prometheus text format
    pub fn export_prometheus(&self) -> String {
        let mut output = String::new();

        // Add process uptime
        output.push_str(&format!(
            "# HELP process_uptime_seconds Process uptime in seconds\n\
             # TYPE process_uptime_seconds gauge\n\
             process_uptime_seconds {}\n\n",
            self.start_time.elapsed().as_secs_f64()
        ));

        // Export counters
        let counters = self.counters.read();
        for (name, counter) in counters.iter() {
            output.push_str(&format!(
                "# HELP {} Counter metric\n\
                 # TYPE {} counter\n\
                 {} {}\n\n",
                name,
                name,
                name,
                counter.get()
            ));
        }

        // Export gauges
        let gauges = self.gauges.read();
        for (name, gauge) in gauges.iter() {
            output.push_str(&format!(
                "# HELP {} Gauge metric\n\
                 # TYPE {} gauge\n\
                 {} {}\n\n",
                name,
                name,
                name,
                gauge.get()
            ));
        }

        // Export histograms
        let histograms = self.histograms.read();
        for (name, histogram) in histograms.iter() {
            output.push_str(&format!(
                "# HELP {} Histogram metric\n\
                 # TYPE {} histogram\n\
                 {}_sum {}\n\
                 {}_count {}\n\n",
                name,
                name,
                name,
                histogram.sum(),
                name,
                histogram.count()
            ));
        }

        output
    }

    /// Create a /metrics endpoint response
    pub fn metrics_endpoint(&self) -> Response {
        Response::text(self.export_prometheus())
            .header_str("Content-Type", "text/plain; version=0.0.4; charset=utf-8")
    }

    /// Get uptime in seconds
    pub fn uptime_seconds(&self) -> f64 {
        self.start_time.elapsed().as_secs_f64()
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Timer for measuring durations
pub struct Timer {
    start: Instant,
}

impl Timer {
    /// Start a new timer
    pub fn start() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    /// Get elapsed time in seconds
    pub fn elapsed_seconds(&self) -> f64 {
        self.start.elapsed().as_secs_f64()
    }

    /// Observe the elapsed time in a histogram
    pub fn observe_histogram(self, histogram: &Histogram) {
        histogram.observe(self.elapsed_seconds());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter() {
        let counter = Counter::new();
        assert_eq!(counter.get(), 0);

        counter.inc();
        assert_eq!(counter.get(), 1);

        counter.add(5);
        assert_eq!(counter.get(), 6);

        counter.reset();
        assert_eq!(counter.get(), 0);
    }

    #[test]
    fn test_gauge() {
        let gauge = Gauge::new();
        assert_eq!(gauge.get(), 0.0);

        gauge.set(42.5);
        assert_eq!(gauge.get(), 42.5);

        gauge.inc();
        assert_eq!(gauge.get(), 43.5);

        gauge.dec();
        assert_eq!(gauge.get(), 42.5);

        gauge.add(7.5);
        assert_eq!(gauge.get(), 50.0);

        gauge.sub(10.0);
        assert_eq!(gauge.get(), 40.0);
    }

    #[test]
    fn test_histogram() {
        let histogram = Histogram::new();
        assert_eq!(histogram.count(), 0);
        assert_eq!(histogram.sum(), 0.0);

        histogram.observe(1.0);
        histogram.observe(2.0);
        histogram.observe(3.0);

        assert_eq!(histogram.count(), 3);
        assert_eq!(histogram.sum(), 6.0);
        assert_eq!(histogram.average(), 2.0);
    }

    #[test]
    fn test_metrics_registry() {
        let metrics = Metrics::new();

        let counter = metrics.counter("test_counter");
        counter.inc();
        assert_eq!(counter.get(), 1);

        let gauge = metrics.gauge("test_gauge");
        gauge.set(42.0);
        assert_eq!(gauge.get(), 42.0);

        let histogram = metrics.histogram("test_histogram");
        histogram.observe(1.5);
        assert_eq!(histogram.count(), 1);
    }

    #[test]
    fn test_prometheus_export() {
        let metrics = Metrics::new();

        metrics.counter("http_requests_total").add(100);
        metrics.gauge("active_connections").set(5.0);
        metrics.histogram("request_duration").observe(0.5);

        let export = metrics.export_prometheus();

        assert!(export.contains("http_requests_total 100"));
        assert!(export.contains("active_connections 5"));
        assert!(export.contains("request_duration_count 1"));
        assert!(export.contains("request_duration_sum 0.5"));
    }

    #[test]
    fn test_timer() {
        let timer = Timer::start();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let elapsed = timer.elapsed_seconds();

        assert!(elapsed >= 0.01);
        assert!(elapsed < 1.0);
    }
}
