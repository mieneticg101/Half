use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;

/// Request ID for distributed tracing
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RequestId(String);

impl RequestId {
    /// Generate a new request ID
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    /// Create from an existing string
    pub fn from_string(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the request ID as a string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for RequestId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for RequestId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for RequestId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Trace context for a request
#[derive(Debug, Clone)]
pub struct TraceContext {
    /// Request ID
    pub request_id: RequestId,
    /// Parent request ID (for distributed tracing)
    pub parent_id: Option<RequestId>,
    /// Trace ID (for grouping related requests)
    pub trace_id: Option<String>,
    /// Span ID (for nested operations)
    pub span_id: Option<String>,
    /// Start time
    pub start_time: Instant,
}

impl TraceContext {
    /// Create a new trace context
    pub fn new() -> Self {
        Self {
            request_id: RequestId::new(),
            parent_id: None,
            trace_id: None,
            span_id: None,
            start_time: Instant::now(),
        }
    }

    /// Create from an existing request ID
    pub fn from_request_id(request_id: RequestId) -> Self {
        Self {
            request_id,
            parent_id: None,
            trace_id: None,
            span_id: None,
            start_time: Instant::now(),
        }
    }

    /// Set parent request ID
    pub fn with_parent(mut self, parent_id: RequestId) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    /// Set trace ID
    pub fn with_trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.trace_id = Some(trace_id.into());
        self
    }

    /// Set span ID
    pub fn with_span_id(mut self, span_id: impl Into<String>) -> Self {
        self.span_id = Some(span_id.into());
        self
    }

    /// Get elapsed time since start
    pub fn elapsed(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }

    /// Get elapsed time in milliseconds
    pub fn elapsed_ms(&self) -> u128 {
        self.start_time.elapsed().as_millis()
    }
}

impl Default for TraceContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Request tracing configuration
#[derive(Debug, Clone)]
pub struct TraceConfig {
    /// Header name for request ID (default: X-Request-ID)
    pub request_id_header: String,
    /// Header name for trace ID (default: X-Trace-ID)
    pub trace_id_header: String,
    /// Header name for parent ID (default: X-Parent-ID)
    pub parent_id_header: String,
    /// Generate request ID if not provided
    pub auto_generate: bool,
}

impl Default for TraceConfig {
    fn default() -> Self {
        Self {
            request_id_header: "X-Request-ID".to_string(),
            trace_id_header: "X-Trace-ID".to_string(),
            parent_id_header: "X-Parent-ID".to_string(),
            auto_generate: true,
        }
    }
}

impl TraceConfig {
    /// Create a new trace configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set request ID header name
    pub fn request_id_header(mut self, header: impl Into<String>) -> Self {
        self.request_id_header = header.into();
        self
    }

    /// Set trace ID header name
    pub fn trace_id_header(mut self, header: impl Into<String>) -> Self {
        self.trace_id_header = header.into();
        self
    }

    /// Set parent ID header name
    pub fn parent_id_header(mut self, header: impl Into<String>) -> Self {
        self.parent_id_header = header.into();
        self
    }

    /// Enable/disable auto-generation of request IDs
    pub fn auto_generate(mut self, auto_generate: bool) -> Self {
        self.auto_generate = auto_generate;
        self
    }
}

/// Request tracer
#[derive(Debug, Clone)]
pub struct RequestTracer {
    config: Arc<TraceConfig>,
}

impl RequestTracer {
    /// Create a new request tracer
    pub fn new(config: TraceConfig) -> Self {
        Self {
            config: Arc::new(config),
        }
    }

    /// Create a trace context from request headers
    pub fn create_context(
        &self,
        request_id: Option<String>,
        trace_id: Option<String>,
        parent_id: Option<String>,
    ) -> TraceContext {
        let request_id = if let Some(id) = request_id {
            RequestId::from_string(id)
        } else if self.config.auto_generate {
            RequestId::new()
        } else {
            RequestId::from_string("unknown")
        };

        let mut context = TraceContext::from_request_id(request_id);

        if let Some(trace_id) = trace_id {
            context = context.with_trace_id(trace_id);
        }

        if let Some(parent_id) = parent_id {
            context = context.with_parent(RequestId::from_string(parent_id));
        }

        context
    }

    /// Get configuration
    pub fn config(&self) -> &TraceConfig {
        &self.config
    }
}

impl Default for RequestTracer {
    fn default() -> Self {
        Self::new(TraceConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_id_new() {
        let id1 = RequestId::new();
        let id2 = RequestId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_request_id_from_string() {
        let id = RequestId::from_string("test-id");
        assert_eq!(id.as_str(), "test-id");
    }

    #[test]
    fn test_request_id_display() {
        let id = RequestId::from_string("test-123");
        assert_eq!(format!("{}", id), "test-123");
    }

    #[test]
    fn test_trace_context_new() {
        let context = TraceContext::new();
        assert!(context.parent_id.is_none());
        assert!(context.trace_id.is_none());
        assert!(context.span_id.is_none());
    }

    #[test]
    fn test_trace_context_with_parent() {
        let parent_id = RequestId::from_string("parent-123");
        let context = TraceContext::new().with_parent(parent_id.clone());
        assert_eq!(context.parent_id.unwrap(), parent_id);
    }

    #[test]
    fn test_trace_context_with_trace_id() {
        let context = TraceContext::new().with_trace_id("trace-456");
        assert_eq!(context.trace_id.unwrap(), "trace-456");
    }

    #[test]
    fn test_trace_context_elapsed() {
        let context = TraceContext::new();
        std::thread::sleep(std::time::Duration::from_millis(10));
        assert!(context.elapsed_ms() >= 10);
    }

    #[test]
    fn test_trace_config_default() {
        let config = TraceConfig::default();
        assert_eq!(config.request_id_header, "X-Request-ID");
        assert_eq!(config.trace_id_header, "X-Trace-ID");
        assert_eq!(config.parent_id_header, "X-Parent-ID");
        assert!(config.auto_generate);
    }

    #[test]
    fn test_trace_config_builder() {
        let config = TraceConfig::new()
            .request_id_header("Request-ID")
            .trace_id_header("Trace-ID")
            .parent_id_header("Parent-ID")
            .auto_generate(false);

        assert_eq!(config.request_id_header, "Request-ID");
        assert_eq!(config.trace_id_header, "Trace-ID");
        assert_eq!(config.parent_id_header, "Parent-ID");
        assert!(!config.auto_generate);
    }

    #[test]
    fn test_request_tracer_create_context_auto_generate() {
        let tracer = RequestTracer::default();
        let context = tracer.create_context(None, None, None);
        assert!(!context.request_id.as_str().is_empty());
    }

    #[test]
    fn test_request_tracer_create_context_with_ids() {
        let tracer = RequestTracer::default();
        let context = tracer.create_context(
            Some("req-123".to_string()),
            Some("trace-456".to_string()),
            Some("parent-789".to_string()),
        );

        assert_eq!(context.request_id.as_str(), "req-123");
        assert_eq!(context.trace_id.unwrap(), "trace-456");
        assert_eq!(context.parent_id.unwrap().as_str(), "parent-789");
    }

    #[test]
    fn test_request_tracer_no_auto_generate() {
        let config = TraceConfig::new().auto_generate(false);
        let tracer = RequestTracer::new(config);
        let context = tracer.create_context(None, None, None);
        assert_eq!(context.request_id.as_str(), "unknown");
    }
}
