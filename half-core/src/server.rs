//! HTTP server implementation
//!
//! Provides a high-performance HTTP server built on Hyper and Tokio.

use crate::{error::Result, router::Router, Request, Response};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

/// HTTP Server
///
/// A lightweight, high-performance HTTP server.
pub struct Server {
    router: Arc<Router>,
    addr: SocketAddr,
}

impl Server {
    /// Create a new server with a router
    pub fn new(router: Router) -> Self {
        Self {
            router: Arc::new(router),
            addr: ([127, 0, 0, 1], 3000).into(),
        }
    }

    /// Set the server address
    pub fn bind(mut self, addr: impl Into<SocketAddr>) -> Self {
        self.addr = addr.into();
        self
    }

    /// Start the server
    ///
    /// This will block until the server is shut down.
    pub async fn run(self) -> Result<()> {
        let listener = TcpListener::bind(self.addr).await?;
        println!("🚀 Half server listening on http://{}", self.addr);

        loop {
            let (stream, remote_addr) = listener.accept().await?;
            let io = TokioIo::new(stream);
            let router = Arc::clone(&self.router);

            tokio::task::spawn(async move {
                let service = service_fn(move |hyper_req| {
                    let router = Arc::clone(&router);
                    async move {
                        handle_request(router, hyper_req).await
                    }
                });

                if let Err(err) = http1::Builder::new()
                    .serve_connection(io, service)
                    .await
                {
                    eprintln!("Error serving connection from {}: {}", remote_addr, err);
                }
            });
        }
    }
}

/// Handle a single HTTP request
async fn handle_request(
    router: Arc<Router>,
    hyper_req: hyper::Request<hyper::body::Incoming>,
) -> std::result::Result<hyper::Response<http_body_util::Full<bytes::Bytes>>, hyper::Error> {
    // Extract request components
    let (parts, body) = hyper_req.into_parts();

    // Convert to Half Request
    let request = match Request::from_hyper(
        parts.method,
        parts.uri,
        parts.version,
        parts.headers,
        body,
    )
    .await
    {
        Ok(req) => req,
        Err(error) => {
            // Error creating request, return error response
            let response = Response::from_error(&error);
            return Ok(response.into_hyper());
        }
    };

    // Route the request
    let response = router.handle(request).await;

    // Convert to Hyper response
    Ok(response.into_hyper())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_creation() {
        let router = Router::new();
        let server = Server::new(router);

        assert_eq!(server.addr, ([127, 0, 0, 1], 3000).into());
    }

    #[test]
    fn test_server_bind() {
        let router = Router::new();
        let server = Server::new(router).bind(([0, 0, 0, 0], 8080));

        assert_eq!(server.addr, ([0, 0, 0, 0], 8080).into());
    }
}
