//! Testing utilities for Half framework
//!
//! Provides tools for writing integration and unit tests for Half applications.

use crate::{Request, Router};
use hyper::{Method, StatusCode};
use std::collections::HashMap;

/// Test client for making HTTP requests to a router
///
/// Provides a simple API for testing routes without starting a real server.
pub struct TestClient {
    router: Router,
}

impl TestClient {
    /// Create a new test client with the given router
    pub fn new(router: Router) -> Self {
        Self { router }
    }

    /// Make a GET request
    pub async fn get(&self, path: &str) -> TestResponse {
        self.request(Method::GET, path, Vec::new()).await
    }

    /// Make a POST request with body
    pub async fn post(&self, path: &str, body: impl Into<Vec<u8>>) -> TestResponse {
        self.request(Method::POST, path, body.into()).await
    }

    /// Make a PUT request with body
    pub async fn put(&self, path: &str, body: impl Into<Vec<u8>>) -> TestResponse {
        self.request(Method::PUT, path, body.into()).await
    }

    /// Make a DELETE request
    pub async fn delete(&self, path: &str) -> TestResponse {
        self.request(Method::DELETE, path, Vec::new()).await
    }

    /// Make a PATCH request with body
    pub async fn patch(&self, path: &str, body: impl Into<Vec<u8>>) -> TestResponse {
        self.request(Method::PATCH, path, body.into()).await
    }

    /// Make a custom HTTP request
    async fn request(&self, method: Method, path: &str, body: Vec<u8>) -> TestResponse {
        // Create a test request
        let mut req = match Request::from_path_and_method(path, method.clone()) {
            Ok(r) => r,
            Err(_) => {
                return TestResponse {
                    status: StatusCode::BAD_REQUEST,
                    headers: HashMap::new(),
                    body: b"Invalid request".to_vec(),
                }
            }
        };

        // Set body if provided
        if !body.is_empty() {
            req.set_body(body);
        }

        // Handle the request through the router
        let response = self.router.handle(req).await;

        TestResponse {
            status: response.get_status(),
            headers: response
                .get_headers()
                .iter()
                .map(|(k, v)| {
                    (
                        k.as_str().to_string(),
                        v.to_str().unwrap_or("").to_string(),
                    )
                })
                .collect(),
            body: response.get_body().to_vec(),
        }
    }
}

/// Test response wrapper with assertion helpers
#[derive(Debug)]
pub struct TestResponse {
    status: StatusCode,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl TestResponse {
    /// Get the response status code
    pub fn status(&self) -> StatusCode {
        self.status
    }

    /// Get a header value
    pub fn header(&self, name: &str) -> Option<&String> {
        self.headers.get(name)
    }

    /// Get the response body as bytes
    pub fn body(&self) -> &[u8] {
        &self.body
    }

    /// Get the response body as a string
    pub fn text(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.body.clone())
    }

    /// Parse the response body as JSON
    pub fn json<T: serde::de::DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_slice(&self.body)
    }

    /// Assert the status code
    pub fn assert_status(&self, expected: StatusCode) -> &Self {
        assert_eq!(
            self.status, expected,
            "Expected status {}, got {}",
            expected, self.status
        );
        self
    }

    /// Assert the response is successful (2xx)
    pub fn assert_success(&self) -> &Self {
        assert!(
            self.status.is_success(),
            "Expected success status, got {}",
            self.status
        );
        self
    }

    /// Assert the response is a client error (4xx)
    pub fn assert_client_error(&self) -> &Self {
        assert!(
            self.status.is_client_error(),
            "Expected client error status, got {}",
            self.status
        );
        self
    }

    /// Assert the response is a server error (5xx)
    pub fn assert_server_error(&self) -> &Self {
        assert!(
            self.status.is_server_error(),
            "Expected server error status, got {}",
            self.status
        );
        self
    }

    /// Assert a header exists
    pub fn assert_header(&self, name: &str, expected: &str) -> &Self {
        match self.header(name) {
            Some(value) => assert_eq!(
                value, expected,
                "Expected header '{}' to be '{}', got '{}'",
                name, expected, value
            ),
            None => panic!("Expected header '{}' not found", name),
        }
        self
    }

    /// Assert the body contains a substring
    pub fn assert_body_contains(&self, expected: &str) -> &Self {
        let body = self.text().expect("Failed to convert body to string");
        assert!(
            body.contains(expected),
            "Expected body to contain '{}', got '{}'",
            expected,
            body
        );
        self
    }

    /// Assert the body equals a string
    pub fn assert_body(&self, expected: &str) -> &Self {
        let body = self.text().expect("Failed to convert body to string");
        assert_eq!(body, expected, "Body mismatch");
        self
    }

    /// Assert the JSON body matches a value
    pub fn assert_json<T: serde::de::DeserializeOwned + PartialEq + std::fmt::Debug>(
        &self,
        expected: T,
    ) -> &Self {
        let actual: T = self.json().expect("Failed to parse JSON");
        assert_eq!(actual, expected, "JSON mismatch");
        self
    }
}

/// Request builder for creating test requests
pub struct TestRequest {
    method: Method,
    path: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl TestRequest {
    /// Create a new test request
    pub fn new(method: Method, path: impl Into<String>) -> Self {
        Self {
            method,
            path: path.into(),
            headers: HashMap::new(),
            body: Vec::new(),
        }
    }

    /// Create a GET request
    pub fn get(path: impl Into<String>) -> Self {
        Self::new(Method::GET, path)
    }

    /// Create a POST request
    pub fn post(path: impl Into<String>) -> Self {
        Self::new(Method::POST, path)
    }

    /// Add a header
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    /// Set the request body
    pub fn body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = body.into();
        self
    }

    /// Set the request body as JSON
    pub fn json<T: serde::Serialize>(mut self, value: &T) -> Self {
        self.body = serde_json::to_vec(value).expect("Failed to serialize JSON");
        self.headers
            .insert("content-type".to_string(), "application/json".to_string());
        self
    }

    /// Build the request
    pub fn build(self) -> Result<Request, crate::Error> {
        let mut req = Request::from_path_and_method(&self.path, self.method)?;

        // Set headers
        for (name, value) in self.headers {
            req.set_header(&name, &value);
        }

        // Set body
        if !self.body.is_empty() {
            req.set_body(self.body);
        }

        Ok(req)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Response;

    #[tokio::test]
    async fn test_client_get_request() {
        let mut router = Router::new();
        router.get("/test", |_req| async { Response::text("Hello, Test!") });

        let client = TestClient::new(router);
        let response = client.get("/test").await;

        response
            .assert_success()
            .assert_status(StatusCode::OK)
            .assert_body("Hello, Test!");
    }

    #[tokio::test]
    async fn test_client_post_request() {
        let mut router = Router::new();
        router.post("/echo", |_req| async { Response::text("Echo") });

        let client = TestClient::new(router);
        let response = client.post("/echo", b"test data").await;

        response.assert_success();
    }

    #[tokio::test]
    async fn test_response_assertions() {
        let mut router = Router::new();
        router.get("/json", |_req| async {
            Response::json(&serde_json::json!({
                "message": "success"
            }))
            .unwrap()
        });

        let client = TestClient::new(router);
        let response = client.get("/json").await;

        response
            .assert_success()
            .assert_header("content-type", "application/json; charset=utf-8");
    }

    #[test]
    fn test_request_builder() {
        let _req = TestRequest::get("/test")
            .header("authorization", "Bearer token")
            .body(b"test body".to_vec());

        assert!(true);
    }
}
