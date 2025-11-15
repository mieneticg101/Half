//! Health check endpoints and middleware
//!
//! Provides health check functionality for monitoring service status.

use crate::{Response, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Health check status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    /// Service is healthy
    Healthy,
    /// Service is degraded but operational
    Degraded,
    /// Service is unhealthy
    Unhealthy,
}

impl HealthStatus {
    /// Convert to HTTP status code
    pub fn to_status_code(&self) -> hyper::StatusCode {
        match self {
            HealthStatus::Healthy => hyper::StatusCode::OK,
            HealthStatus::Degraded => hyper::StatusCode::OK, // Still return 200 but with degraded status
            HealthStatus::Unhealthy => hyper::StatusCode::SERVICE_UNAVAILABLE,
        }
    }
}

/// Individual component health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// Component name
    pub name: String,
    /// Component status
    pub status: HealthStatus,
    /// Optional message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Response time in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_time_ms: Option<u64>,
}

/// Overall health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Overall status
    pub status: HealthStatus,
    /// Uptime in seconds
    pub uptime_seconds: u64,
    /// Timestamp (ISO 8601)
    pub timestamp: String,
    /// Component health checks
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub components: HashMap<String, ComponentHealth>,
    /// Service version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

/// Health check function type
pub type HealthCheckFn =
    Box<dyn Fn() -> Pin<Box<dyn Future<Output = Result<ComponentHealth>> + Send>> + Send + Sync>;

/// Health check manager
///
/// Manages health checks for the application and its components.
///
/// # Example
/// ```
/// use half_core::health::{HealthCheck, HealthStatus, ComponentHealth};
///
/// #[tokio::main]
/// async fn main() {
///     let mut health = HealthCheck::new()
///         .version("1.0.0");
///
///     // Add database health check
///     health.add_check("database", || {
///         Box::pin(async {
///             // Check database connection
///             Ok(ComponentHealth {
///                 name: "database".to_string(),
///                 status: HealthStatus::Healthy,
///                 message: Some("Connected".to_string()),
///                 response_time_ms: Some(5),
///             })
///         })
///     });
///
///     // Get health status
///     let response = health.check().await;
///     assert_eq!(response.status, HealthStatus::Healthy);
/// }
/// ```
pub struct HealthCheck {
    start_time: Instant,
    checks: Arc<RwLock<HashMap<String, HealthCheckFn>>>,
    version: Option<String>,
}

impl HealthCheck {
    /// Create a new health check manager
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            checks: Arc::new(RwLock::new(HashMap::new())),
            version: None,
        }
    }

    /// Set the service version
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Add a health check for a component
    pub async fn add_check<F, Fut>(&mut self, name: impl Into<String>, check: F)
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<ComponentHealth>> + Send + 'static,
    {
        let check_fn: HealthCheckFn = Box::new(move || Box::pin(check()));
        self.checks.write().await.insert(name.into(), check_fn);
    }

    /// Perform all health checks
    pub async fn check(&self) -> HealthResponse {
        let checks = self.checks.read().await;
        let mut components = HashMap::new();
        let mut overall_status = HealthStatus::Healthy;

        // Run all health checks
        for (name, check) in checks.iter() {
            match check().await {
                Ok(component) => {
                    // Update overall status based on component status
                    match component.status {
                        HealthStatus::Unhealthy => overall_status = HealthStatus::Unhealthy,
                        HealthStatus::Degraded if overall_status == HealthStatus::Healthy => {
                            overall_status = HealthStatus::Degraded
                        }
                        _ => {}
                    }
                    components.insert(name.clone(), component);
                }
                Err(e) => {
                    overall_status = HealthStatus::Unhealthy;
                    components.insert(
                        name.clone(),
                        ComponentHealth {
                            name: name.clone(),
                            status: HealthStatus::Unhealthy,
                            message: Some(format!("Check failed: {}", e)),
                            response_time_ms: None,
                        },
                    );
                }
            }
        }

        HealthResponse {
            status: overall_status,
            uptime_seconds: self.start_time.elapsed().as_secs(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            components,
            version: self.version.clone(),
        }
    }

    /// Create a simple liveness probe response
    ///
    /// Returns 200 OK if the service is running (no component checks).
    pub async fn liveness(&self) -> Response {
        Response::json(&serde_json::json!({
            "status": "alive",
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }))
        .unwrap_or_else(|_| Response::text("alive"))
    }

    /// Create a readiness probe response
    ///
    /// Returns 200 OK if all components are healthy, 503 otherwise.
    pub async fn readiness(&self) -> Response {
        let health = self.check().await;
        let status_code = health.status.to_status_code();

        Response::json(&health)
            .unwrap_or_else(|_| Response::text("unhealthy"))
            .status(status_code)
    }

    /// Create a detailed health endpoint response
    pub async fn health(&self) -> Response {
        let health = self.check().await;
        let status_code = health.status.to_status_code();

        Response::json(&health)
            .unwrap_or_else(|_| Response::text("error"))
            .status(status_code)
    }
}

impl Default for HealthCheck {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a simple health check that always returns healthy
pub fn simple_health_check(
    name: impl Into<String>,
) -> impl Fn() -> Pin<Box<dyn Future<Output = Result<ComponentHealth>> + Send>> + Send + Sync {
    let name = name.into();
    move || {
        let name = name.clone();
        Box::pin(async move {
            Ok(ComponentHealth {
                name,
                status: HealthStatus::Healthy,
                message: None,
                response_time_ms: None,
            })
        })
    }
}

/// Create a health check with timeout
pub fn health_check_with_timeout<F, Fut>(
    name: impl Into<String>,
    check: F,
    timeout: Duration,
) -> impl Fn() -> Pin<Box<dyn Future<Output = Result<ComponentHealth>> + Send>> + Send + Sync
where
    F: Fn() -> Fut + Send + Sync + Clone + 'static,
    Fut: Future<Output = Result<ComponentHealth>> + Send + 'static,
{
    let name = name.into();
    move || {
        let check = check.clone();
        let name = name.clone();
        Box::pin(async move {
            match tokio::time::timeout(timeout, check()).await {
                Ok(result) => result,
                Err(_) => Ok(ComponentHealth {
                    name,
                    status: HealthStatus::Unhealthy,
                    message: Some("Health check timed out".to_string()),
                    response_time_ms: Some(timeout.as_millis() as u64),
                }),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check_new() {
        let health = HealthCheck::new();
        let response = health.check().await;

        assert_eq!(response.status, HealthStatus::Healthy);
        assert_eq!(response.components.len(), 0);
    }

    #[tokio::test]
    async fn test_health_check_with_version() {
        let health = HealthCheck::new().version("1.0.0");
        let response = health.check().await;

        assert_eq!(response.version, Some("1.0.0".to_string()));
    }

    #[tokio::test]
    async fn test_add_healthy_check() {
        let mut health = HealthCheck::new();

        health
            .add_check("test", || async {
                Ok(ComponentHealth {
                    name: "test".to_string(),
                    status: HealthStatus::Healthy,
                    message: None,
                    response_time_ms: Some(1),
                })
            })
            .await;

        let response = health.check().await;
        assert_eq!(response.status, HealthStatus::Healthy);
        assert_eq!(response.components.len(), 1);
    }

    #[tokio::test]
    async fn test_add_unhealthy_check() {
        let mut health = HealthCheck::new();

        health
            .add_check("failing", || async {
                Ok(ComponentHealth {
                    name: "failing".to_string(),
                    status: HealthStatus::Unhealthy,
                    message: Some("Failed".to_string()),
                    response_time_ms: None,
                })
            })
            .await;

        let response = health.check().await;
        assert_eq!(response.status, HealthStatus::Unhealthy);
    }

    #[tokio::test]
    async fn test_liveness_probe() {
        let health = HealthCheck::new();
        let response = health.liveness().await;

        assert_eq!(response.get_status(), hyper::StatusCode::OK);
    }

    #[test]
    fn test_health_status_to_status_code() {
        assert_eq!(
            HealthStatus::Healthy.to_status_code(),
            hyper::StatusCode::OK
        );
        assert_eq!(
            HealthStatus::Degraded.to_status_code(),
            hyper::StatusCode::OK
        );
        assert_eq!(
            HealthStatus::Unhealthy.to_status_code(),
            hyper::StatusCode::SERVICE_UNAVAILABLE
        );
    }
}
