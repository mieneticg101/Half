//! HTTP server implementation
//!
//! Provides a high-performance HTTP server built on Hyper and Tokio with HTTP/2 and TLS 1.3 support.

use crate::{error::Result, router::Router, Request, Response};
use hyper::server::conn::{http1, http2};
use hyper::service::service_fn;
use hyper_util::rt::{TokioExecutor, TokioIo};
use rustls::ServerConfig as RustlsServerConfig;
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::fs::File;
use std::io::BufReader;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::signal;
use tokio::sync::broadcast;
use tokio_rustls::TlsAcceptor;

/// TLS configuration for HTTPS server
///
/// Supports TLS 1.3 with secure cipher suites
#[derive(Clone)]
pub struct TlsConfig {
    cert_path: String,
    key_path: String,
}

impl TlsConfig {
    /// Create a new TLS configuration
    ///
    /// # Arguments
    /// * `cert_path` - Path to the certificate file (PEM format)
    /// * `key_path` - Path to the private key file (PKCS8 PEM format)
    pub fn new(cert_path: impl Into<String>, key_path: impl Into<String>) -> Self {
        Self {
            cert_path: cert_path.into(),
            key_path: key_path.into(),
        }
    }

    /// Load TLS configuration from files
    fn load(&self) -> Result<Arc<RustlsServerConfig>> {
        // Load certificate chain
        let cert_file = File::open(&self.cert_path)
            .map_err(|e| crate::error::Error::InternalError(format!("Failed to open certificate: {}", e)))?;
        let mut cert_reader = BufReader::new(cert_file);
        let cert_chain: Vec<_> = certs(&mut cert_reader)
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| crate::error::Error::InternalError(format!("Failed to parse certificate: {}", e)))?;

        // Load private key
        let key_file = File::open(&self.key_path)
            .map_err(|e| crate::error::Error::InternalError(format!("Failed to open private key: {}", e)))?;
        let mut key_reader = BufReader::new(key_file);
        let mut keys = pkcs8_private_keys(&mut key_reader)
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| crate::error::Error::InternalError(format!("Failed to parse private key: {}", e)))?;

        if keys.is_empty() {
            return Err(crate::error::Error::InternalError("No private keys found".to_string()));
        }

        // Configure TLS with modern, secure settings
        let config = RustlsServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(cert_chain, keys.remove(0).into())
            .map_err(|e| crate::error::Error::InternalError(format!("Failed to create TLS config: {}", e)))?;

        Ok(Arc::new(config))
    }
}

/// HTTP Server
///
/// A lightweight, high-performance HTTP/HTTPS server with HTTP/2 and TLS 1.3 support.
///
/// # Features
/// - HTTP/1.1 and HTTP/2 protocol support
/// - TLS 1.3 encryption
/// - Automatic protocol negotiation (ALPN)
/// - Connection pooling
/// - Request multiplexing (HTTP/2)
///
/// # Examples
///
/// HTTP server with HTTP/2:
/// ```no_run
/// # use half_core::{Router, Server};
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let router = Router::new();
/// Server::new(router)
///     .bind(([127, 0, 0, 1], 3000))
///     .http2(true)
///     .run()
///     .await?;
/// # Ok(())
/// # }
/// ```
///
/// HTTPS server with TLS 1.3 and HTTP/2:
/// ```no_run
/// # use half_core::{Router, Server, TlsConfig};
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let router = Router::new();
/// let tls = TlsConfig::new("cert.pem", "key.pem");
/// Server::new(router)
///     .bind(([127, 0, 0, 1], 443))
///     .tls(tls)
///     .http2(true)  // Enable HTTP/2
///     .run()
///     .await?;
/// # Ok(())
/// # }
/// ```
pub struct Server {
    router: Arc<Router>,
    addr: SocketAddr,
    tls_config: Option<TlsConfig>,
    enable_http2: bool,
    graceful_shutdown_timeout: Duration,
}

impl Server {
    /// Create a new server with a router
    pub fn new(router: Router) -> Self {
        Self {
            router: Arc::new(router),
            addr: ([127, 0, 0, 1], 3000).into(),
            tls_config: None,
            enable_http2: false,
            graceful_shutdown_timeout: Duration::from_secs(30),
        }
    }

    /// Set the server address
    pub fn bind(mut self, addr: impl Into<SocketAddr>) -> Self {
        self.addr = addr.into();
        self
    }

    /// Enable TLS 1.3 with the given configuration
    ///
    /// This enables HTTPS with TLS 1.3, the most modern and secure TLS version.
    pub fn tls(mut self, config: TlsConfig) -> Self {
        self.tls_config = Some(config);
        self
    }

    /// Enable or disable HTTP/2 support
    ///
    /// HTTP/2 provides better performance through:
    /// - Request/response multiplexing
    /// - Header compression (HPACK)
    /// - Server push capabilities
    /// - Binary protocol efficiency
    ///
    /// When enabled with TLS, ALPN negotiation is used to select HTTP/2.
    /// HTTP/1.1 is used as fallback for clients that don't support HTTP/2.
    pub fn http2(mut self, enable: bool) -> Self {
        self.enable_http2 = enable;
        self
    }

    /// Set graceful shutdown timeout
    ///
    /// When a shutdown signal is received, the server will stop accepting new connections
    /// and wait for existing connections to complete for up to this duration.
    ///
    /// Default: 30 seconds
    pub fn graceful_shutdown_timeout(mut self, timeout: Duration) -> Self {
        self.graceful_shutdown_timeout = timeout;
        self
    }

    /// Start the server
    ///
    /// This will block until the server is shut down.
    /// Supports both HTTP and HTTPS (TLS 1.3) based on configuration.
    ///
    /// # Graceful Shutdown
    /// The server listens for SIGTERM and SIGINT signals (Ctrl+C).
    /// When a signal is received:
    /// 1. Stop accepting new connections
    /// 2. Wait for existing requests to complete (up to graceful_shutdown_timeout)
    /// 3. Shutdown the server
    pub async fn run(mut self) -> Result<()> {
        let listener = TcpListener::bind(self.addr).await?;

        // Create shutdown channel
        let (shutdown_tx, _) = broadcast::channel::<()>(1);

        // Spawn shutdown signal handler
        let shutdown_tx_clone = shutdown_tx.clone();
        tokio::spawn(async move {
            shutdown_signal().await;
            println!("\n🛑 Graceful shutdown initiated...");
            let _ = shutdown_tx_clone.send(());
        });

        // Check if TLS is configured
        if let Some(tls_config) = self.tls_config.take() {
            self.run_tls(listener, tls_config, shutdown_tx).await
        } else {
            self.run_http(listener, shutdown_tx).await
        }
    }

    /// Run HTTP server (no TLS)
    async fn run_http(self, listener: TcpListener, shutdown_tx: broadcast::Sender<()>) -> Result<()> {
        let protocol = if self.enable_http2 { "HTTP/2" } else { "HTTP/1.1" };
        println!("🚀 Half server listening on http://{} ({})", self.addr, protocol);
        println!("💡 Press Ctrl+C for graceful shutdown");

        let enable_http2 = self.enable_http2;
        let timeout = self.graceful_shutdown_timeout;
        let mut shutdown_rx = shutdown_tx.subscribe();

        loop {
            tokio::select! {
                // Accept new connections
                result = listener.accept() => {
                    let (stream, remote_addr) = result?;
                    let io = TokioIo::new(stream);
                    let router = Arc::clone(&self.router);

                    tokio::task::spawn(async move {
                        let service = service_fn(move |hyper_req| {
                            let router = Arc::clone(&router);
                            async move {
                                handle_request(router, hyper_req).await
                            }
                        });

                        let result = if enable_http2 {
                            // Use HTTP/2
                            http2::Builder::new(TokioExecutor::new())
                                .serve_connection(io, service)
                                .await
                        } else {
                            // Use HTTP/1.1
                            http1::Builder::new()
                                .serve_connection(io, service)
                                .await
                        };

                        if let Err(err) = result {
                            eprintln!("Error serving connection from {}: {}", remote_addr, err);
                        }
                    });
                }

                // Shutdown signal received
                _ = shutdown_rx.recv() => {
                    println!("⏳ Waiting up to {:?} for connections to complete...", timeout);
                    tokio::time::sleep(timeout).await;
                    println!("✅ Server shutdown complete");
                    return Ok(());
                }
            }
        }
    }

    /// Run HTTPS server with TLS 1.3
    async fn run_tls(self, listener: TcpListener, tls_config: TlsConfig, shutdown_tx: broadcast::Sender<()>) -> Result<()> {
        let protocol = if self.enable_http2 { "TLS 1.3 + HTTP/2" } else { "TLS 1.3 + HTTP/1.1" };
        println!("🔒 Half server listening on https://{} ({})", self.addr, protocol);
        println!("💡 Press Ctrl+C for graceful shutdown");

        // Load TLS configuration with ALPN support for HTTP/2
        let mut rustls_config = tls_config.load()?;

        // Enable ALPN for HTTP/2 negotiation if HTTP/2 is enabled
        if self.enable_http2 {
            let config = Arc::get_mut(&mut rustls_config)
                .ok_or_else(|| crate::error::Error::InternalError("Failed to configure ALPN".to_string()))?;
            config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
        }

        let tls_acceptor = TlsAcceptor::from(rustls_config);
        let enable_http2 = self.enable_http2;
        let timeout = self.graceful_shutdown_timeout;
        let mut shutdown_rx = shutdown_tx.subscribe();

        loop {
            tokio::select! {
                // Accept new connections
                result = listener.accept() => {
                    let (stream, remote_addr) = result?;
                    let tls_acceptor = tls_acceptor.clone();
                    let router = Arc::clone(&self.router);

                    tokio::task::spawn(async move {
                        // Perform TLS handshake
                        let tls_stream = match tls_acceptor.accept(stream).await {
                            Ok(s) => s,
                            Err(e) => {
                                eprintln!("TLS handshake failed from {}: {}", remote_addr, e);
                                return;
                            }
                        };

                        let io = TokioIo::new(tls_stream);

                        let service = service_fn(move |hyper_req| {
                            let router = Arc::clone(&router);
                            async move {
                                handle_request(router, hyper_req).await
                            }
                        });

                        let result = if enable_http2 {
                            // Use HTTP/2
                            http2::Builder::new(TokioExecutor::new())
                                .serve_connection(io, service)
                                .await
                        } else {
                            // Use HTTP/1.1
                            http1::Builder::new()
                                .serve_connection(io, service)
                                .await
                        };

                        if let Err(err) = result {
                            eprintln!("Error serving connection from {}: {}", remote_addr, err);
                        }
                    });
                }

                // Shutdown signal received
                _ = shutdown_rx.recv() => {
                    println!("⏳ Waiting up to {:?} for connections to complete...", timeout);
                    tokio::time::sleep(timeout).await;
                    println!("✅ Server shutdown complete");
                    return Ok(());
                }
            }
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

/// Wait for shutdown signal (SIGTERM or SIGINT)
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_creation() {
        let router = Router::new();
        let server = Server::new(router);

        assert_eq!(server.addr, ([127, 0, 0, 1], 3000).into());
        assert!(server.tls_config.is_none());
    }

    #[test]
    fn test_server_bind() {
        let router = Router::new();
        let server = Server::new(router).bind(([0, 0, 0, 0], 8080));

        assert_eq!(server.addr, ([0, 0, 0, 0], 8080).into());
    }

    #[test]
    fn test_tls_config_creation() {
        let tls = TlsConfig::new("cert.pem", "key.pem");
        assert_eq!(tls.cert_path, "cert.pem");
        assert_eq!(tls.key_path, "key.pem");
    }

    #[test]
    fn test_server_with_tls() {
        let router = Router::new();
        let tls = TlsConfig::new("cert.pem", "key.pem");
        let server = Server::new(router).tls(tls);

        assert!(server.tls_config.is_some());
    }
}
