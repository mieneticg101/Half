//! Routing system for Half framework
//!
//! Provides a fast, type-safe router with support for path parameters,
//! middleware, and method-based routing.

use crate::{
    error::Result,
    handler::{Handler, HandlerWrapper},
    middleware::{BoxedMiddleware, Middleware},
    Request, Response,
};
use hyper::Method;
use std::collections::HashMap;
use std::sync::Arc;

/// Route definition
pub(crate) struct Route {
    pub(crate) method: Method,
    pub(crate) path: String,
    handler: Arc<HandlerWrapper>,
    #[allow(dead_code)]
    middlewares: Vec<BoxedMiddleware>,
}

/// Router for handling HTTP requests
///
/// The router matches incoming requests to registered handlers based on
/// HTTP method and path patterns.
pub struct Router {
    pub(crate) routes: Vec<Route>,
    global_middlewares: Vec<BoxedMiddleware>,
    not_found_handler: Option<Arc<HandlerWrapper>>,
}

impl Router {
    /// Create a new router
    pub fn new() -> Self {
        Self {
            routes: Vec::new(),
            global_middlewares: Vec::new(),
            not_found_handler: None,
        }
    }

    /// Register a route handler
    pub fn route<H>(&mut self, method: Method, path: impl Into<String>, handler: H) -> &mut Self
    where
        H: Handler,
    {
        self.routes.push(Route {
            method,
            path: path.into(),
            handler: Arc::new(HandlerWrapper::new(handler)),
            middlewares: Vec::new(),
        });
        self
    }

    /// Register a GET route
    pub fn get<H>(&mut self, path: impl Into<String>, handler: H) -> &mut Self
    where
        H: Handler,
    {
        self.route(Method::GET, path, handler)
    }

    /// Register a POST route
    pub fn post<H>(&mut self, path: impl Into<String>, handler: H) -> &mut Self
    where
        H: Handler,
    {
        self.route(Method::POST, path, handler)
    }

    /// Register a PUT route
    pub fn put<H>(&mut self, path: impl Into<String>, handler: H) -> &mut Self
    where
        H: Handler,
    {
        self.route(Method::PUT, path, handler)
    }

    /// Register a DELETE route
    pub fn delete<H>(&mut self, path: impl Into<String>, handler: H) -> &mut Self
    where
        H: Handler,
    {
        self.route(Method::DELETE, path, handler)
    }

    /// Register a PATCH route
    pub fn patch<H>(&mut self, path: impl Into<String>, handler: H) -> &mut Self
    where
        H: Handler,
    {
        self.route(Method::PATCH, path, handler)
    }

    /// Add a global middleware
    ///
    /// Global middlewares are applied to all routes
    pub fn use_middleware<M>(&mut self, middleware: M) -> &mut Self
    where
        M: Middleware,
    {
        self.global_middlewares.push(Box::new(middleware));
        self
    }

    /// Set a custom 404 handler
    pub fn not_found<H>(&mut self, handler: H) -> &mut Self
    where
        H: Handler,
    {
        self.not_found_handler = Some(Arc::new(HandlerWrapper::new(handler)));
        self
    }

    /// Handle an incoming request
    pub async fn handle(&self, mut req: Request) -> Response {
        // Try to find a matching route
        match self.find_route(&req) {
            Some((route, params)) => {
                // Set path parameters
                req.set_params(params);

                // Apply middlewares and handle request
                match self.apply_middlewares_and_handle(&route, req).await {
                    Ok(response) => response,
                    Err(error) => Response::from_error(&error),
                }
            }
            None => {
                // No route found, use 404 handler or default
                if let Some(handler) = &self.not_found_handler {
                    match handler.handle(req).await {
                        Ok(response) => response,
                        Err(error) => Response::from_error(&error),
                    }
                } else {
                    Response::not_found()
                }
            }
        }
    }

    /// Find a matching route for the request
    fn find_route(&self, req: &Request) -> Option<(&Route, HashMap<String, String>)> {
        let path = req.path();
        let method = req.method();

        for route in &self.routes {
            // Check method match
            if &route.method != method {
                continue;
            }

            // Check path match and extract parameters
            if let Some(params) = Self::match_path(&route.path, path) {
                return Some((route, params));
            }
        }

        None
    }

    /// Match a route pattern against a request path
    ///
    /// Supports path parameters like `/users/:id`
    fn match_path(pattern: &str, path: &str) -> Option<HashMap<String, String>> {
        let pattern_parts: Vec<&str> = pattern.split('/').filter(|s| !s.is_empty()).collect();
        let path_parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

        // Must have same number of parts
        if pattern_parts.len() != path_parts.len() {
            return None;
        }

        let mut params = HashMap::new();

        for (pattern_part, path_part) in pattern_parts.iter().zip(path_parts.iter()) {
            if pattern_part.starts_with(':') {
                // This is a parameter
                let param_name = &pattern_part[1..]; // Remove the ':'
                params.insert(param_name.to_string(), path_part.to_string());
            } else if pattern_part != path_part {
                // Static parts must match exactly
                return None;
            }
        }

        Some(params)
    }

    /// Apply middlewares and execute handler
    async fn apply_middlewares_and_handle(&self, route: &Route, req: Request) -> Result<Response> {
        // For now, just call the handler directly
        // TODO: Implement middleware chain execution
        route.handler.handle(req).await
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_path_exact() {
        let params = Router::match_path("/users", "/users");
        assert!(params.is_some());
        assert!(params.unwrap().is_empty());
    }

    #[test]
    fn test_match_path_with_param() {
        let params = Router::match_path("/users/:id", "/users/123");
        assert!(params.is_some());

        let params = params.unwrap();
        assert_eq!(params.get("id"), Some(&"123".to_string()));
    }

    #[test]
    fn test_match_path_multiple_params() {
        let params = Router::match_path("/users/:id/posts/:post_id", "/users/123/posts/456");
        assert!(params.is_some());

        let params = params.unwrap();
        assert_eq!(params.get("id"), Some(&"123".to_string()));
        assert_eq!(params.get("post_id"), Some(&"456".to_string()));
    }

    #[test]
    fn test_match_path_no_match() {
        let params = Router::match_path("/users/:id", "/posts/123");
        assert!(params.is_none());

        let params = Router::match_path("/users/:id", "/users/123/extra");
        assert!(params.is_none());
    }

    #[tokio::test]
    async fn test_router_get() {
        let mut router = Router::new();

        async fn handler(_req: Request) -> Response {
            Response::text("Hello")
        }

        router.get("/test", handler);

        assert_eq!(router.routes.len(), 1);
        assert_eq!(router.routes[0].method, Method::GET);
        assert_eq!(router.routes[0].path, "/test");
    }
}
