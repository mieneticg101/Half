//! Secure server example with TLS 1.3, Nonce Protection, and Helmet
//!
//! This example demonstrates:
//! - TLS 1.3 encryption for HTTPS
//! - One-time request protection with nonce
//! - Comprehensive security headers with Helmet
//! - CSRF protection
//! - XSS prevention
//!
//! To run this example:
//! 1. Generate self-signed certificate for testing:
//!    ```bash
//!    openssl req -x509 -newkey rsa:4096 -nodes \
//!      -keyout key.pem -out cert.pem -days 365 \
//!      -subj "/CN=localhost"
//!    ```
//! 2. Run the example:
//!    ```bash
//!    cargo run --example secure_server
//!    ```
//! 3. Test with curl:
//!    ```bash
//!    # Get a nonce first
//!    curl -k https://localhost:8443/nonce
//!
//!    # Use the nonce in a request
//!    curl -k -H "X-Nonce: YOUR_NONCE_HERE" https://localhost:8443/api/data
//!    ```

use half_core::{
    Request, Response, Router, Server, TlsConfig,
    security::{Helmet, NonceProtection, CsrfProtection, XssFilter, CspConfig},
};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔒 Secure Half Server Example");
    println!("=====================================");
    println!();

    // Create router
    let mut router = Router::new();

    // Configure Helmet with strict security headers
    let helmet = Helmet::new();
    router.use_middleware(helmet);

    // Configure CSRF protection
    let csrf = CsrfProtection::new(b"super-secret-key-must-be-32-bytes!!")
        .exempt("/public")
        .exempt("/nonce");
    router.use_middleware(csrf);

    // Configure XSS filter
    let xss = XssFilter::new()
        .csp("default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'");
    router.use_middleware(xss);

    // Configure Nonce Protection for API routes
    let nonce = NonceProtection::new()
        .ttl(300)  // 5 minutes
        .cleanup_interval(60)  // Clean every minute
        .exempt("/public")
        .exempt("/nonce");  // Exempt nonce generation endpoint
    router.use_middleware(nonce.clone());

    // Public routes (no authentication required)
    router.get("/", |_req: Request| async {
        Response::html(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Half Secure Server</title>
    <style>
        body { font-family: Arial, sans-serif; max-width: 800px; margin: 50px auto; padding: 20px; }
        .feature { background: #f0f0f0; padding: 15px; margin: 10px 0; border-radius: 5px; }
        code { background: #e0e0e0; padding: 2px 5px; border-radius: 3px; }
    </style>
</head>
<body>
    <h1>🔒 Half Framework - Secure Server</h1>
    <p>This server demonstrates advanced security features:</p>

    <div class="feature">
        <h3>✅ TLS 1.3</h3>
        <p>Latest TLS version with strongest cipher suites</p>
    </div>

    <div class="feature">
        <h3>✅ Nonce Protection</h3>
        <p>One-time request tokens prevent replay attacks</p>
        <p>Get nonce: <code>GET /nonce</code></p>
    </div>

    <div class="feature">
        <h3>✅ Helmet Security Headers</h3>
        <p>Comprehensive headers including CSP, HSTS, and more</p>
    </div>

    <div class="feature">
        <h3>✅ CSRF Protection</h3>
        <p>Token-based protection for state-changing requests</p>
    </div>

    <div class="feature">
        <h3>✅ XSS Prevention</h3>
        <p>Automatic HTML escaping and Content Security Policy</p>
    </div>

    <h2>API Endpoints</h2>
    <ul>
        <li><code>GET /</code> - This page</li>
        <li><code>GET /nonce</code> - Generate a new nonce</li>
        <li><code>GET /api/data</code> - Protected API endpoint (requires nonce)</li>
        <li><code>GET /public/status</code> - Public endpoint (no nonce required)</li>
    </ul>
</body>
</html>
        "#)
    });

    // Nonce generation endpoint
    router.get("/nonce", move |_req: Request| {
        let nonce_clone = nonce.clone();
        async move {
            let nonce = nonce_clone.generate_nonce();
            Response::json(&json!({
                "nonce": nonce,
                "expires_in": 300,
                "usage": "Include this nonce in X-Nonce header for protected endpoints"
            }))
            .unwrap()
        }
    });

    // Protected API endpoint (requires nonce)
    router.get("/api/data", |_req: Request| async {
        Response::json(&json!({
            "status": "success",
            "message": "This endpoint is protected by nonce",
            "data": {
                "users": 42,
                "posts": 128
            },
            "security": {
                "tls": "1.3",
                "nonce_protected": true,
                "csrf_protected": true
            }
        }))
        .unwrap()
    });

    // Public endpoint (no nonce required)
    router.get("/public/status", |_req: Request| async {
        Response::json(&json!({
            "status": "online",
            "version": "0.2.0",
            "security_features": [
                "TLS 1.3",
                "Nonce Protection",
                "Helmet Headers",
                "CSRF Protection",
                "XSS Prevention"
            ]
        }))
        .unwrap()
    });

    // Configure TLS 1.3
    // Note: For production, use proper certificates from Let's Encrypt or CA
    let tls_config = TlsConfig::new("cert.pem", "key.pem");

    // Start HTTPS server
    println!("🔒 Starting HTTPS server with TLS 1.3...");
    println!("📍 Listening on: https://localhost:8443");
    println!();
    println!("Security Features Enabled:");
    println!("  ✅ TLS 1.3 encryption");
    println!("  ✅ Nonce-based replay protection");
    println!("  ✅ Helmet security headers");
    println!("  ✅ CSRF protection");
    println!("  ✅ XSS prevention");
    println!();
    println!("Test commands:");
    println!("  curl -k https://localhost:8443/");
    println!("  curl -k https://localhost:8443/nonce");
    println!("  curl -k https://localhost:8443/public/status");
    println!();

    Server::new(router)
        .bind(([127, 0, 0, 1], 8443))
        .tls(tls_config)
        .run()
        .await?;

    Ok(())
}
