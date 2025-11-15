# Changelog

All notable changes to the Half framework will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
