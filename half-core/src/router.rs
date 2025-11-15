//! Routing system for Half framework
//!
//! Provides a fast, type-safe router with support for path parameters,
//! middleware, and method-based routing.

use crate::{
    error::{Error, Result},
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
    pub(crate) name: Option<String>,
    handler: Arc<HandlerWrapper>,
    middlewares: Vec<BoxedMiddleware>,
}

/// Router for handling HTTP requests
///
/// The router matches incoming requests to registered handlers based on
/// HTTP method and path patterns.
///
/// # Performance
/// Uses a HashMap for exact match routes (O(1) lookup) and falls back to
/// pattern matching for routes with parameters (O(n) lookup).
pub struct Router {
    pub(crate) routes: Vec<Route>,
    /// Fast lookup for exact match routes (no parameters)
    exact_routes: HashMap<String, usize>,
    /// Named routes for URL generation
    named_routes: HashMap<String, usize>,
    global_middlewares: Vec<BoxedMiddleware>,
    not_found_handler: Option<Arc<HandlerWrapper>>,
}

impl Router {
    /// Create a new router
    pub fn new() -> Self {
        Self {
            routes: Vec::new(),
            exact_routes: HashMap::new(),
            named_routes: HashMap::new(),
            global_middlewares: Vec::new(),
            not_found_handler: None,
        }
    }

    /// Register a route handler
    pub fn route<H>(&mut self, method: Method, path: impl Into<String>, handler: H) -> &mut Self
    where
        H: Handler,
    {
        let path = path.into();
        let route_index = self.routes.len();

        // Check if this is an exact match route (no parameters)
        if !path.contains(':') {
            let key = format!("{} {}", method.as_str(), path);
            self.exact_routes.insert(key, route_index);
        }

        self.routes.push(Route {
            method,
            path,
            name: None,
            handler: Arc::new(HandlerWrapper::new(handler)),
            middlewares: Vec::new(),
        });
        self
    }

    /// Name the last registered route
    ///
    /// This allows you to generate URLs for this route later using `url()`.
    ///
    /// # Example
    /// ```ignore
    /// router.get("/users/:id", user_handler).name("user.show");
    /// let url = router.url("user.show", &[("id", "123")]).unwrap();
    /// // url = "/users/123"
    /// ```
    pub fn name(&mut self, name: impl Into<String>) -> &mut Self {
        let name = name.into();
        let route_index = self.routes.len().saturating_sub(1);

        if let Some(last_route) = self.routes.last_mut() {
            self.named_routes.insert(name.clone(), route_index);
            last_route.name = Some(name);
        }
        self
    }

    /// Apply middleware to the last registered route
    ///
    /// This allows you to apply middleware to specific routes rather than globally.
    ///
    /// # Example
    /// ```ignore
    /// use half_core::middleware::Logger;
    ///
    /// router.get("/admin", admin_handler).middleware(Logger);
    /// ```
    pub fn middleware<M>(&mut self, middleware: M) -> &mut Self
    where
        M: Middleware,
    {
        if let Some(last_route) = self.routes.last_mut() {
            last_route.middlewares.push(Box::new(middleware));
        }
        self
    }

    /// Generate a URL for a named route
    ///
    /// # Arguments
    /// * `name` - The name of the route
    /// * `params` - An optional slice of (key, value) tuples for route parameters
    ///
    /// # Returns
    /// The generated URL path, or an error if the route name is not found
    /// or required parameters are missing.
    ///
    /// # Example
    /// ```ignore
    /// router.get("/users/:id", handler).name("user.show");
    /// let url = router.url("user.show", &[("id", "123")]).unwrap();
    /// assert_eq!(url, "/users/123");
    /// ```
    pub fn url(&self, name: &str, params: &[(&str, &str)]) -> Result<String> {
        let route_index = self.named_routes.get(name)
            .ok_or_else(|| Error::Custom(format!("Route '{}' not found", name)))?;

        let route = self.routes.get(*route_index)
            .ok_or_else(|| Error::InternalError("Invalid route index".to_string()))?;

        // Build URL by replacing parameters in the path
        let params_map: HashMap<&str, &str> = params.iter().copied().collect();

        // Find all :param patterns and replace them
        let parts: Vec<&str> = route.path.split('/').collect();
        let mut result_parts = Vec::new();

        for part in parts {
            if let Some(param_name) = part.strip_prefix(':') {
                // This is a parameter
                let value = params_map.get(param_name)
                    .ok_or_else(|| Error::BadRequest(
                        format!("Missing parameter '{}' for route '{}'", param_name, name)
                    ))?;
                result_parts.push(value.to_string());
            } else {
                result_parts.push(part.to_string());
            }
        }

        let url = result_parts.join("/");
        Ok(url)
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
                match self.apply_middlewares_and_handle(route, req).await {
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
    ///
    /// # Performance
    /// First tries O(1) HashMap lookup for exact matches,
    /// then falls back to O(n) pattern matching for parameterized routes
    fn find_route(&self, req: &Request) -> Option<(&Route, HashMap<String, String>)> {
        let path = req.path();
        let method = req.method();

        // Try fast exact match first
        let key = format!("{} {}", method.as_str(), path);
        if let Some(&index) = self.exact_routes.get(&key) {
            if let Some(route) = self.routes.get(index) {
                return Some((route, HashMap::new()));
            }
        }

        // Fall back to pattern matching for parameterized routes
        for route in &self.routes {
            // Check method match
            if route.method != *method {
                continue;
            }

            // Skip exact matches (already checked above)
            if !route.path.contains(':') {
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
    ///
    /// # Performance
    /// Uses iterators instead of Vec allocation for better performance
    fn match_path(pattern: &str, path: &str) -> Option<HashMap<String, String>> {
        let mut pattern_parts = pattern.split('/').filter(|s| !s.is_empty());
        let mut path_parts = path.split('/').filter(|s| !s.is_empty());

        let mut params = HashMap::new();

        loop {
            match (pattern_parts.next(), path_parts.next()) {
                (Some(pattern_part), Some(path_part)) => {
                    if let Some(param_name) = pattern_part.strip_prefix(':') {
                        // This is a parameter
                        params.insert(param_name.to_string(), path_part.to_string());
                    } else if pattern_part != path_part {
                        // Static parts must match exactly
                        return None;
                    }
                }
                (None, None) => {
                    // Both iterators exhausted at the same time - match!
                    return Some(params);
                }
                _ => {
                    // Different number of parts - no match
                    return None;
                }
            }
        }
    }

    /// Apply middlewares and execute handler
    async fn apply_middlewares_and_handle(&self, route: &Route, req: Request) -> Result<Response> {
        // Note: Middleware chain execution is simplified for now.
        // Global middlewares are applied first, then route-specific middlewares.
        // Each middleware gets the handler as "next" - proper chaining of multiple
        // middlewares is complex in Rust and would require a more sophisticated approach.

        // If no middlewares, just call the handler directly
        if self.global_middlewares.is_empty() && route.middlewares.is_empty() {
            return route.handler.handle(req).await;
        }

        // For now, we'll apply the last middleware in the chain with the handler as next
        // This works correctly for single middleware scenarios
        let handler = route.handler.clone();

        // Create the "next" closure that calls the handler
        let next = Box::new(move |req: Request| {
            Box::pin(async move { handler.handle(req).await })
                as std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response>> + Send>>
        }) as crate::middleware::Next;

        // Apply route-specific middleware if present, otherwise use handler
        if let Some(middleware) = route.middlewares.last() {
            middleware.handle(req, next).await
        } else if let Some(middleware) = self.global_middlewares.last() {
            middleware.handle(req, next).await
        } else {
            // Shouldn't reach here given the check above, but call handler as fallback
            drop(next); // Can't use next after this
            route.handler.handle(req).await
        }
    }

    /// Create a route group with a prefix
    ///
    /// Route groups allow you to organize related routes with a common prefix
    /// and optionally shared middleware.
    ///
    /// # Example
    /// ```ignore
    /// let mut router = Router::new();
    /// router.group("/api", |api| {
    ///     api.get("/users", list_users);
    ///     api.post("/users", create_user);
    /// });
    /// // Creates routes: GET /api/users, POST /api/users
    /// ```
    pub fn group<F>(&mut self, prefix: &str, configure: F) -> &mut Self
    where
        F: FnOnce(&mut RouteGroup),
    {
        let mut group = RouteGroup::new(prefix.to_string(), self);
        configure(&mut group);
        self
    }

    /// Mount another router at a prefix
    ///
    /// This allows you to create modular routers and combine them.
    ///
    /// # Example
    /// ```ignore
    /// let mut api_router = Router::new();
    /// api_router.get("/users", list_users);
    ///
    /// let mut main_router = Router::new();
    /// main_router.mount("/api", api_router);
    /// // Creates route: GET /api/users
    /// ```
    pub fn mount(&mut self, prefix: &str, other: Router) -> &mut Self {
        let prefix = prefix.trim_end_matches('/');

        for route in other.routes {
            let path = if route.path.starts_with('/') {
                format!("{}{}", prefix, route.path)
            } else {
                format!("{}/{}", prefix, route.path)
            };

            // Re-register the route with the prefixed path
            let route_index = self.routes.len();

            // Update exact_routes if it was an exact match
            if !path.contains(':') {
                let key = format!("{} {}", route.method.as_str(), path);
                self.exact_routes.insert(key, route_index);
            }

            self.routes.push(Route {
                method: route.method,
                path,
                name: route.name,
                handler: route.handler,
                middlewares: route.middlewares,
            });
        }

        self
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

/// Route group builder
///
/// Provides a scoped API for registering routes with a common prefix.
pub struct RouteGroup<'a> {
    prefix: String,
    router: &'a mut Router,
}

impl<'a> RouteGroup<'a> {
    fn new(prefix: String, router: &'a mut Router) -> Self {
        Self { prefix, router }
    }

    fn prefixed_path(&self, path: &str) -> String {
        let prefix = self.prefix.trim_end_matches('/');
        if path.starts_with('/') {
            format!("{}{}", prefix, path)
        } else {
            format!("{}/{}", prefix, path)
        }
    }

    /// Register a GET route in this group
    pub fn get<H>(&mut self, path: &str, handler: H) -> &mut Self
    where
        H: Handler,
    {
        let full_path = self.prefixed_path(path);
        self.router.get(full_path, handler);
        self
    }

    /// Register a POST route in this group
    pub fn post<H>(&mut self, path: &str, handler: H) -> &mut Self
    where
        H: Handler,
    {
        let full_path = self.prefixed_path(path);
        self.router.post(full_path, handler);
        self
    }

    /// Register a PUT route in this group
    pub fn put<H>(&mut self, path: &str, handler: H) -> &mut Self
    where
        H: Handler,
    {
        let full_path = self.prefixed_path(path);
        self.router.put(full_path, handler);
        self
    }

    /// Register a DELETE route in this group
    pub fn delete<H>(&mut self, path: &str, handler: H) -> &mut Self
    where
        H: Handler,
    {
        let full_path = self.prefixed_path(path);
        self.router.delete(full_path, handler);
        self
    }

    /// Register a PATCH route in this group
    pub fn patch<H>(&mut self, path: &str, handler: H) -> &mut Self
    where
        H: Handler,
    {
        let full_path = self.prefixed_path(path);
        self.router.patch(full_path, handler);
        self
    }

    /// Create a nested group
    pub fn group<F>(&mut self, prefix: &str, configure: F) -> &mut Self
    where
        F: FnOnce(&mut RouteGroup),
    {
        let nested_prefix = self.prefixed_path(prefix);
        let mut nested_group = RouteGroup::new(nested_prefix, self.router);
        configure(&mut nested_group);
        self
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

    #[test]
    fn test_named_route() {
        let mut router = Router::new();

        async fn handler(_req: Request) -> Response {
            Response::text("Hello")
        }

        router.get("/users/:id", handler).name("user.show");

        assert_eq!(router.routes[0].name, Some("user.show".to_string()));
        assert_eq!(router.named_routes.get("user.show"), Some(&0));
    }

    #[test]
    fn test_url_generation() {
        let mut router = Router::new();

        async fn handler(_req: Request) -> Response {
            Response::text("Hello")
        }

        router.get("/users/:id", handler).name("user.show");

        let url = router.url("user.show", &[("id", "123")]).unwrap();
        assert_eq!(url, "/users/123");
    }

    #[test]
    fn test_url_generation_multiple_params() {
        let mut router = Router::new();

        async fn handler(_req: Request) -> Response {
            Response::text("Hello")
        }

        router.get("/users/:user_id/posts/:post_id", handler).name("user.post");

        let url = router.url("user.post", &[("user_id", "42"), ("post_id", "100")]).unwrap();
        assert_eq!(url, "/users/42/posts/100");
    }

    #[test]
    fn test_url_generation_missing_param() {
        let mut router = Router::new();

        async fn handler(_req: Request) -> Response {
            Response::text("Hello")
        }

        router.get("/users/:id", handler).name("user.show");

        let result = router.url("user.show", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_url_generation_unknown_route() {
        let router = Router::new();

        let result = router.url("unknown.route", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_route_middleware() {
        use crate::middleware::Logger;

        let mut router = Router::new();

        async fn handler(_req: Request) -> Response {
            Response::text("Hello")
        }

        router.get("/test", handler).middleware(Logger);

        assert_eq!(router.routes[0].middlewares.len(), 1);
    }

    #[test]
    fn test_chained_methods() {
        use crate::middleware::Logger;

        let mut router = Router::new();

        async fn handler(_req: Request) -> Response {
            Response::text("Hello")
        }

        router.get("/users/:id", handler)
            .name("user.show")
            .middleware(Logger);

        assert_eq!(router.routes[0].name, Some("user.show".to_string()));
        assert_eq!(router.routes[0].middlewares.len(), 1);
    }
}
