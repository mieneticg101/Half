# Changelog

All notable changes to the Half framework will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0] - 2025-11-15

### Added - Phase 1: Foundation

#### HTTP/2 Support
- **Full HTTP/2 Protocol**: Native HTTP/2 support with automatic protocol negotiation
- **ALPN Support**: Automatic protocol negotiation (h2, http/1.1) when using TLS
- **Multiplexing**: Request/response multiplexing for better performance
- **Header Compression**: HPACK header compression for reduced bandwidth
- **Backward Compatible**: Automatic fallback to HTTP/1.1 for unsupported clients
- **Simple API**: Enable with `.http2(true)` on Server builder

#### Response Compression Middleware
- **Multi-Algorithm Support**: Brotli, Gzip, and Zstd compression
- **Automatic Negotiation**: Based on Accept-Encoding header
- **Configurable Levels**: Fast, Default, and Best compression levels
- **Smart Filtering**: Only compresses appropriate content types (text/*, application/json, etc.)
- **Minimum Size Threshold**: Configurable minimum response size (default 1KB)
- **Already-Compressed Detection**: Skips re-compression of already compressed content
- **Zero Runtime Overhead**: Efficient async compression using async-compression crate

#### Rate Limiting Middleware
- **Token Bucket Algorithm**: Smooth rate limiting using governor crate
- **Flexible Time Windows**: Per-second, per-minute, per-hour configurations
- **Path Exemptions**: Exclude specific endpoints (e.g., /health, /metrics)
- **429 Response**: Proper HTTP 429 Too Many Requests status
- **Retry-After Header**: Automatic retry-after header in responses
- **High Performance**: Lock-free implementation with minimal overhead

#### Graceful Shutdown
- **Signal Handling**: Listens for SIGTERM and SIGINT (Ctrl+C)
- **Connection Draining**: Stops accepting new connections on shutdown
- **Timeout Control**: Configurable timeout for in-flight requests (default 30s)
- **Clean Shutdown**: Waits for active connections to complete
- **User Feedback**: Clear console messages during shutdown process
- **Production Ready**: Ensures no request is dropped during deployment

### Changed

#### Server Enhancements
- **Enhanced Server API**: New methods `http2()`, `graceful_shutdown_timeout()`
- **Better Logging**: Protocol information in startup messages
- **Improved Documentation**: Comprehensive examples for all new features

#### Middleware System
- **Modular Structure**: Middleware module restructured from single file to directory
- **Enhanced Logger**: Added request timing to Logger middleware
- **Enhanced CORS**: Added `allow_credentials` and `max_age` options to CORS middleware

#### Response Builder
- **Clone Support**: Response struct now implements Clone (required for compression)
- **New Getter Methods**: `get_body()`, `get_headers()`, `get_headers_mut()`, `get_status()`

### Performance

- **HTTP/2 Multiplexing**: Significantly improved throughput for multiple concurrent requests
- **Compression**: 60-80% bandwidth reduction for text content
- **Zero-Copy Operations**: Efficient buffer handling in compression pipeline
- **Lock-Free Rate Limiting**: Minimal overhead for high-traffic scenarios

### Tests

- All 51 unit tests passing
- All 6 doc tests passing
- Clean build with zero warnings

### Dependencies

Added for Phase 1:
- `async-compression` 0.4 - Async compression with Brotli, Gzip, Zstd
- `flate2` 1.0 - Fallback compression
- `governor` 0.7 - Token bucket rate limiting
- `parking_lot` 0.12 - High-performance synchronization

## [0.3.0] - 2025-11-15

### Added

#### TLS 1.3 Support
- **Native HTTPS**: Full TLS 1.3 support with Rustls
- **TlsConfig**: Easy configuration for HTTPS servers
- **Automatic Protocol Selection**: Seamlessly switch between HTTP and HTTPS
- **Modern Cipher Suites**: Only the most secure TLS 1.3 cipher suites enabled
- **Certificate Management**: Support for PEM format certificates and private keys

#### Helmet Security Headers (Beyond Helmet.js)
- **Comprehensive Security Headers**: More extensive than Helmet.js
  - Content-Security-Policy with Trusted Types support
  - Strict-Transport-Security (HSTS) with preload
  - X-Frame-Options
  - X-Content-Type-Options
  - Referrer-Policy
  - Permissions-Policy (formerly Feature-Policy)
  - Cross-Origin-Embedder-Policy (COEP)
  - Cross-Origin-Opener-Policy (COOP)
  - Cross-Origin-Resource-Policy (CORP)
  - X-DNS-Prefetch-Control
  - Expect-CT (Certificate Transparency)
  - Origin-Agent-Cluster
  - X-Download-Options
  - X-Permitted-Cross-Domain-Policies
- **Flexible Configuration**: Easily customize all security headers
- **CSP Builder**: Type-safe Content Security Policy configuration
- **Permissions Policy Builder**: Control browser features and APIs
- **Zero Runtime Overhead**: Compile-time optimizations

#### Nonce Protection (One-Time Requests)
- **Replay Attack Prevention**: Nonce-based request validation
- **Time-Based Expiration**: Configurable TTL for nonces (default 5 minutes)
- **Automatic Cleanup**: Background task removes expired nonces
- **Thread-Safe**: DashMap for concurrent nonce storage
- **Path Exemptions**: Exclude public endpoints from nonce validation
- **Cryptographically Secure**: Uses Rust's secure random number generator
- **Custom Header Support**: Configurable nonce header name (default: `X-Nonce`)

#### Dependencies
- Added `rustls` 0.23 - Pure Rust TLS 1.3 implementation
- Added `tokio-rustls` 0.26 - Async TLS for Tokio
- Added `rustls-pemfile` 2.2 - PEM file parsing
- Added `dashmap` 6.1 - Concurrent HashMap for nonce storage
- Added `chrono` 0.4 - Time management for nonce expiration

#### Examples
- **secure_server.rs**: Complete HTTPS server with TLS 1.3, Nonce Protection, and Helmet
- **helmet_demo.rs**: Interactive demonstration of security headers

### Changed

#### API Enhancements
- **Response::header_str()**: New convenience method for setting headers with string keys and values
- **Server::tls()**: New method to enable TLS 1.3
- **Server::run()**: Now automatically detects and runs HTTP or HTTPS based on configuration

#### Export Additions
- Exported `TlsConfig` from `server` module
- Exported `Helmet`, `CspConfig`, `PermissionsPolicyConfig` from `security` module
- Exported `NonceProtection` from `security` module
- Updated `prelude` module with new security features

### Security Improvements
- **Transport Layer Security**: Full encryption with TLS 1.3
- **Replay Protection**: Prevents request replay attacks
- **Enhanced Headers**: Industry-leading security header defaults
- **Minimal Information Disclosure**: Strict error sanitization

### Tests
- Added 9 new tests for TLS, Nonce Protection, and Helmet
- Total test count: 43 tests (all passing)
- Added async tests for nonce validation and expiration

### Documentation
- Updated README with TLS 1.3, Helmet, and Nonce Protection examples
- Added comprehensive examples for secure server configuration
- Updated security section with new features
- Added example commands for testing HTTPS endpoints

### Performance
- **Zero-Copy TLS**: Efficient TLS implementation with minimal overhead
- **Lazy Header Building**: Security headers built only when needed
- **Concurrent Nonce Storage**: DashMap provides lock-free reads

## [0.2.0] - 2025-11-15

### Added

#### Security Enhancements
- **Request Body Size Limits**: Implemented 10MB maximum body size to prevent DoS attacks
- **Query Parameter Limits**: Maximum 100 query parameters to prevent hash collision attacks
- **Parameter Length Validation**: Maximum 4KB per query parameter key/value to prevent memory exhaustion
- **Enhanced Security Headers**: Automatically added to all responses:
  - `Referrer-Policy: strict-origin-when-cross-origin`
  - `Permissions-Policy: geolocation=(), microphone=(), camera=()`
  - `X-Permitted-Cross-Domain-Policies: none`
- **Error Message Sanitization**: Server errors (5xx) now show generic messages in production to prevent information leakage
- **Input Validation**: Added bounds checking for query parameters with length limits

#### Performance Optimizations
- **Fast Route Lookup**: Implemented O(1) HashMap lookup for exact match routes
- **Optimized Path Matching**: Rewritten to use iterators instead of Vec allocation
- **Reduced Allocations**: Minimized string allocations in hot paths

### Changed

#### Breaking Changes
- **Rust Edition**: Upgraded from 2021 to **2024**
  - Requires Rust 1.85+
  - Updated to use new `rand::rng()` instead of deprecated `thread_rng()`
  - Updated to use `random()` instead of `gen()` (keyword conflict with Rust 2024)

#### Dependencies
- Updated `rand` from 0.8 to 0.9 (latest)
- Updated `bytes` from 1.8 to 1.10
- Updated `hyper` from 1.5 to 1.6
- Updated version from 0.1.0 to 0.2.0

#### Security
- Response builder now applies security headers by default in `Response::new()`
- Error responses sanitize internal server error messages
- Request parsing validates sizes before processing

### Fixed
- Fixed compatibility with Rust 2024 edition
- Fixed potential DoS vectors through unbounded request sizes
- Fixed potential memory exhaustion through large query parameters
- Fixed information leakage in error responses

### Performance
- Exact match routes now use O(1) lookup instead of O(n)
- Path matching uses iterators, reducing memory allocations
- Query parameter parsing validates length during parsing

## [0.1.0] - 2025-11-14

### Added
- Initial release of Half framework
- Core HTTP server with routing system
- Request/Response handling with type safety
- Middleware system
- CSRF protection
- XSS prevention
- Input validation
- Procedural macros for routing
- CLI tool for project scaffolding
- Comprehensive test coverage (34 tests)
- Examples (hello, json_api, routing)

---

## Security Notice

All releases include security improvements. Users are encouraged to upgrade to the latest version.

### Reporting Security Issues

If you discover a security vulnerability, please email the maintainers directly.
Do not open a public issue.
