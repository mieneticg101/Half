//! Helmet middleware demonstration
//!
//! This example shows how to use Helmet for comprehensive security headers,
//! beyond what Helmet.js provides in the Node.js ecosystem.
//!
//! Run: cargo run --example helmet_demo

use half_core::{
    Request, Response, Router, Server,
    security::{Helmet, CspConfig, PermissionsPolicyConfig},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🛡️ Helmet Security Headers Demo");
    println!("===================================");

    let mut router = Router::new();

    // Example 1: Default Helmet (strictest security)
    let helmet_default = Helmet::new();

    // Example 2: Custom CSP configuration
    let mut csp = CspConfig::default();
    csp.script_src = vec![
        "'self'".to_string(),
        "https://cdn.example.com".to_string(),
    ];
    csp.style_src = vec![
        "'self'".to_string(),
        "'unsafe-inline'".to_string(), // Only if necessary
    ];
    csp.img_src = vec![
        "'self'".to_string(),
        "data:".to_string(),
        "https:".to_string(),
    ];

    // Example 3: Custom Permissions Policy
    let mut permissions = PermissionsPolicyConfig::default();
    permissions.camera = vec!["self".to_string()];
    permissions.microphone = vec!["self".to_string()];
    permissions.geolocation = vec!["self".to_string(), "https://maps.example.com".to_string()];

    // Example 4: Highly customized Helmet
    let helmet_custom = Helmet::new()
        .csp(csp)
        .permissions_policy(permissions)
        .hsts(31536000, true, true)  // 1 year, include subdomains, preload
        .frame_options("SAMEORIGIN")
        .referrer_policy("strict-origin-when-cross-origin");

    // Use the custom helmet
    router.use_middleware(helmet_custom);

    // Home page showing all security headers
    router.get("/", |_req: Request| async {
        Response::html(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Helmet Demo - Half Framework</title>
    <style>
        body {
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            max-width: 1000px;
            margin: 50px auto;
            padding: 20px;
            background: #f5f5f5;
        }
        .container {
            background: white;
            padding: 30px;
            border-radius: 10px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }
        h1 { color: #2c3e50; border-bottom: 3px solid #3498db; padding-bottom: 10px; }
        h2 { color: #34495e; margin-top: 30px; }
        .header-item {
            background: #ecf0f1;
            padding: 15px;
            margin: 10px 0;
            border-left: 4px solid #3498db;
            font-family: monospace;
        }
        .header-name {
            font-weight: bold;
            color: #2980b9;
        }
        .header-value {
            color: #27ae60;
            word-break: break-all;
        }
        .feature {
            background: #e8f4f8;
            padding: 15px;
            margin: 15px 0;
            border-radius: 5px;
        }
        code {
            background: #34495e;
            color: #ecf0f1;
            padding: 2px 6px;
            border-radius: 3px;
        }
        .emoji { font-size: 1.2em; }
    </style>
</head>
<body>
    <div class="container">
        <h1><span class="emoji">🛡️</span> Helmet Security Headers</h1>
        <p>This page demonstrates comprehensive security headers provided by Half's Helmet middleware.</p>

        <div class="feature">
            <h3><span class="emoji">✨</span> Beyond Helmet.js</h3>
            <p>Half's Helmet provides all features of Helmet.js plus:</p>
            <ul>
                <li><strong>TLS 1.3 Integration:</strong> Native HTTPS support</li>
                <li><strong>Nonce Protection:</strong> One-time request validation</li>
                <li><strong>Advanced CSP:</strong> Trusted Types support</li>
                <li><strong>Modern Standards:</strong> Latest security headers</li>
                <li><strong>Zero Runtime Cost:</strong> Compile-time optimizations</li>
            </ul>
        </div>

        <h2><span class="emoji">🔒</span> Active Security Headers</h2>
        <p>Open your browser's developer tools (Network tab) to see these headers:</p>

        <div class="header-item">
            <div class="header-name">Content-Security-Policy</div>
            <div class="header-value">Prevents XSS, clickjacking, and other code injection attacks</div>
        </div>

        <div class="header-item">
            <div class="header-name">Strict-Transport-Security</div>
            <div class="header-value">Forces HTTPS connections (HSTS)</div>
        </div>

        <div class="header-item">
            <div class="header-name">X-Frame-Options</div>
            <div class="header-value">Prevents clickjacking attacks</div>
        </div>

        <div class="header-item">
            <div class="header-name">X-Content-Type-Options</div>
            <div class="header-value">Prevents MIME-sniffing attacks</div>
        </div>

        <div class="header-item">
            <div class="header-name">Referrer-Policy</div>
            <div class="header-value">Controls referrer information</div>
        </div>

        <div class="header-item">
            <div class="header-name">Permissions-Policy</div>
            <div class="header-value">Controls browser features and APIs</div>
        </div>

        <div class="header-item">
            <div class="header-name">Cross-Origin-Embedder-Policy</div>
            <div class="header-value">Prevents cross-origin attacks</div>
        </div>

        <div class="header-item">
            <div class="header-name">Cross-Origin-Opener-Policy</div>
            <div class="header-value">Isolates browsing context</div>
        </div>

        <div class="header-item">
            <div class="header-name">Cross-Origin-Resource-Policy</div>
            <div class="header-value">Protects against side-channel attacks</div>
        </div>

        <div class="header-item">
            <div class="header-name">X-DNS-Prefetch-Control</div>
            <div class="header-value">Controls DNS prefetching</div>
        </div>

        <div class="header-item">
            <div class="header-name">Expect-CT</div>
            <div class="header-value">Certificate Transparency enforcement</div>
        </div>

        <div class="header-item">
            <div class="header-name">Origin-Agent-Cluster</div>
            <div class="header-value">Process isolation for security</div>
        </div>

        <h2><span class="emoji">📊</span> Security Score</h2>
        <div class="feature">
            <p>Test your security headers at:</p>
            <ul>
                <li><a href="https://securityheaders.com">SecurityHeaders.com</a></li>
                <li><a href="https://observatory.mozilla.org">Mozilla Observatory</a></li>
            </ul>
            <p>Expected score: <strong>A+</strong> with all Helmet defaults!</p>
        </div>

        <h2><span class="emoji">🚀</span> Quick Start</h2>
        <pre><code>use half_core::{Router, Server, security::Helmet};

let mut router = Router::new();

// Use default strict security
router.use_middleware(Helmet::new());

// Or customize as needed
let helmet = Helmet::new()
    .hsts(31536000, true, true)
    .frame_options("SAMEORIGIN");

router.use_middleware(helmet);
</code></pre>
    </div>
</body>
</html>
        "#)
    });

    // API endpoint to inspect headers
    router.get("/api/headers", |req: Request| async move {
        let mut headers_info = Vec::new();

        // In a real app, you'd inspect response headers
        // This is just a demo of the request
        for (name, value) in req.headers() {
            headers_info.push(format!("{}: {}", name, value.to_str().unwrap_or("(binary)")));
        }

        Response::json(&serde_json::json!({
            "message": "Check the response headers of this request",
            "request_headers_count": headers_info.len(),
            "note": "Security headers are added to the response, not visible here"
        }))
        .unwrap()
    });

    println!("\n🚀 Server starting on http://localhost:3000");
    println!("📋 Visit http://localhost:3000 to see all security headers");
    println!("🔍 Open browser DevTools > Network tab to inspect headers\n");

    Server::new(router)
        .bind(([127, 0, 0, 1], 3000))
        .run()
        .await?;

    Ok(())
}
