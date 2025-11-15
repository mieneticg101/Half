//! Response compression middleware
//!
//! Provides automatic response compression with Brotli, Gzip, and Zstd support.
//! Automatically negotiates the best compression algorithm based on Accept-Encoding header.

use crate::{
    Request, Response,
    error::Result,
    middleware::{Middleware, Next},
};
use async_compression::tokio::bufread::{BrotliEncoder, GzipEncoder, ZstdEncoder};
use bytes::Bytes;
use std::future::Future;
use std::io::Cursor;
use std::pin::Pin;
use tokio::io::AsyncReadExt;

/// Compression algorithm
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionAlgorithm {
    /// Brotli compression (best compression ratio)
    Brotli,
    /// Gzip compression (widely supported)
    Gzip,
    /// Zstd compression (fast and efficient)
    Zstd,
}

impl CompressionAlgorithm {
    /// Get the Content-Encoding header value
    fn encoding_name(&self) -> &'static str {
        match self {
            CompressionAlgorithm::Brotli => "br",
            CompressionAlgorithm::Gzip => "gzip",
            CompressionAlgorithm::Zstd => "zstd",
        }
    }
}

/// Compression level configuration
#[derive(Debug, Clone, Copy)]
pub struct CompressionLevel {
    brotli: i32,
    gzip: i32,
    zstd: i32,
}

impl Default for CompressionLevel {
    fn default() -> Self {
        Self {
            brotli: 6, // 0-11, 6 is balanced
            gzip: 6,   // 0-9, 6 is balanced
            zstd: 3,   // 1-21, 3 is balanced
        }
    }
}

impl CompressionLevel {
    /// Create a new compression level configuration
    pub fn new(brotli: i32, gzip: i32, zstd: i32) -> Self {
        Self { brotli, gzip, zstd }
    }

    /// Fast compression (lower quality, faster)
    pub fn fast() -> Self {
        Self {
            brotli: 4,
            gzip: 4,
            zstd: 1,
        }
    }

    /// Best compression (higher quality, slower)
    pub fn best() -> Self {
        Self {
            brotli: 11,
            gzip: 9,
            zstd: 21,
        }
    }
}

/// Response compression middleware
///
/// Automatically compresses responses based on Accept-Encoding header.
/// Supports Brotli, Gzip, and Zstd compression algorithms.
///
/// # Features
/// - Automatic content negotiation
/// - Configurable compression levels
/// - Smart content-type filtering
/// - Minimum size threshold
/// - Already-compressed content detection
///
/// # Example
/// ```no_run
/// use half_core::middleware::{Compression, CompressionLevel};
///
/// let compression = Compression::new()
///     .brotli(true)
///     .gzip(true)
///     .level(CompressionLevel::fast())
///     .min_size(1024);  // Only compress responses >= 1KB
///
/// // Use with your router
/// // router.use_middleware(compression);
/// ```
#[derive(Debug, Clone)]
pub struct Compression {
    enable_brotli: bool,
    enable_gzip: bool,
    enable_zstd: bool,
    level: CompressionLevel,
    min_size: usize,
    compressible_types: Vec<String>,
}

impl Default for Compression {
    fn default() -> Self {
        Self::new()
    }
}

impl Compression {
    /// Create a new compression middleware with default settings
    pub fn new() -> Self {
        Self {
            enable_brotli: true,
            enable_gzip: true,
            enable_zstd: true,
            level: CompressionLevel::default(),
            min_size: 1024, // 1KB minimum
            compressible_types: vec![
                "text/".to_string(),
                "application/json".to_string(),
                "application/javascript".to_string(),
                "application/xml".to_string(),
                "application/x-javascript".to_string(),
                "application/xhtml+xml".to_string(),
            ],
        }
    }

    /// Enable or disable Brotli compression
    pub fn brotli(mut self, enable: bool) -> Self {
        self.enable_brotli = enable;
        self
    }

    /// Enable or disable Gzip compression
    pub fn gzip(mut self, enable: bool) -> Self {
        self.enable_gzip = enable;
        self
    }

    /// Enable or disable Zstd compression
    pub fn zstd(mut self, enable: bool) -> Self {
        self.enable_zstd = enable;
        self
    }

    /// Set compression level
    pub fn level(mut self, level: CompressionLevel) -> Self {
        self.level = level;
        self
    }

    /// Set minimum response size for compression (in bytes)
    pub fn min_size(mut self, size: usize) -> Self {
        self.min_size = size;
        self
    }

    /// Add a compressible content type pattern
    pub fn add_type(mut self, content_type: impl Into<String>) -> Self {
        self.compressible_types.push(content_type.into());
        self
    }

    /// Parse Accept-Encoding header and choose best compression
    fn choose_compression(&self, accept_encoding: &str) -> Option<CompressionAlgorithm> {
        let encodings: Vec<&str> = accept_encoding
            .split(',')
            .map(|s| s.trim().split(';').next().unwrap_or("").trim())
            .collect();

        // Prefer Brotli > Zstd > Gzip
        if self.enable_brotli && encodings.contains(&"br") {
            Some(CompressionAlgorithm::Brotli)
        } else if self.enable_zstd && encodings.contains(&"zstd") {
            Some(CompressionAlgorithm::Zstd)
        } else if self.enable_gzip && (encodings.contains(&"gzip") || encodings.contains(&"*")) {
            Some(CompressionAlgorithm::Gzip)
        } else {
            None
        }
    }

    /// Check if content type is compressible
    fn is_compressible(&self, content_type: Option<&str>) -> bool {
        if let Some(ct) = content_type {
            self.compressible_types
                .iter()
                .any(|pattern| ct.starts_with(pattern))
        } else {
            false
        }
    }

    /// Compress response body
    async fn compress_body(&self, body: Bytes, algorithm: CompressionAlgorithm) -> Result<Bytes> {
        let cursor = Cursor::new(body);

        let compressed = match algorithm {
            CompressionAlgorithm::Brotli => {
                let mut encoder = BrotliEncoder::with_quality(
                    cursor,
                    async_compression::Level::Precise(self.level.brotli),
                );
                let mut buf = Vec::new();
                encoder.read_to_end(&mut buf).await?;
                Bytes::from(buf)
            }
            CompressionAlgorithm::Gzip => {
                let mut encoder = GzipEncoder::with_quality(
                    cursor,
                    async_compression::Level::Precise(self.level.gzip),
                );
                let mut buf = Vec::new();
                encoder.read_to_end(&mut buf).await?;
                Bytes::from(buf)
            }
            CompressionAlgorithm::Zstd => {
                let mut encoder = ZstdEncoder::with_quality(
                    cursor,
                    async_compression::Level::Precise(self.level.zstd),
                );
                let mut buf = Vec::new();
                encoder.read_to_end(&mut buf).await?;
                Bytes::from(buf)
            }
        };

        Ok(compressed)
    }
}

impl Middleware for Compression {
    fn handle(
        &self,
        req: Request,
        next: Next,
    ) -> Pin<Box<dyn Future<Output = Result<Response>> + Send + '_>> {
        Box::pin(async move {
            // Get Accept-Encoding header
            let accept_encoding = req.header("accept-encoding").unwrap_or("").to_string();

            // Get response
            let mut response = next(req).await?;

            // Check if we should compress
            if accept_encoding.is_empty() {
                return Ok(response);
            }

            // Choose compression algorithm
            let algorithm = match self.choose_compression(&accept_encoding) {
                Some(algo) => algo,
                None => return Ok(response),
            };

            // Check content type
            let content_type = response
                .get_headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok());

            if !self.is_compressible(content_type) {
                return Ok(response);
            }

            // Get response body
            let body = response.get_body().clone();

            // Check minimum size
            if body.len() < self.min_size {
                return Ok(response);
            }

            // Check if already compressed
            if response.get_headers().get("content-encoding").is_some() {
                return Ok(response);
            }

            // Compress the body
            let compressed = match self.compress_body(body, algorithm).await {
                Ok(c) => c,
                Err(_) => return Ok(response), // Return original on compression error
            };

            // Update response
            response = response.body(compressed);
            response = response.header_str("content-encoding", algorithm.encoding_name());
            response = response.header_str("vary", "Accept-Encoding");

            // Remove content-length as it's now incorrect
            response.get_headers_mut().remove("content-length");

            Ok(response)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_level_default() {
        let level = CompressionLevel::default();
        assert_eq!(level.brotli, 6);
        assert_eq!(level.gzip, 6);
        assert_eq!(level.zstd, 3);
    }

    #[test]
    fn test_compression_level_fast() {
        let level = CompressionLevel::fast();
        assert_eq!(level.brotli, 4);
        assert_eq!(level.gzip, 4);
        assert_eq!(level.zstd, 1);
    }

    #[test]
    fn test_compression_level_best() {
        let level = CompressionLevel::best();
        assert_eq!(level.brotli, 11);
        assert_eq!(level.gzip, 9);
        assert_eq!(level.zstd, 21);
    }

    #[test]
    fn test_choose_compression() {
        let comp = Compression::new();

        assert_eq!(
            comp.choose_compression("br, gzip, deflate"),
            Some(CompressionAlgorithm::Brotli)
        );

        assert_eq!(
            comp.choose_compression("gzip, deflate"),
            Some(CompressionAlgorithm::Gzip)
        );

        assert_eq!(
            comp.choose_compression("zstd, gzip"),
            Some(CompressionAlgorithm::Zstd)
        );

        assert_eq!(comp.choose_compression("deflate"), None);
    }

    #[test]
    fn test_is_compressible() {
        let comp = Compression::new();

        assert!(comp.is_compressible(Some("text/html")));
        assert!(comp.is_compressible(Some("text/plain")));
        assert!(comp.is_compressible(Some("application/json")));
        assert!(!comp.is_compressible(Some("image/png")));
        assert!(!comp.is_compressible(Some("video/mp4")));
        assert!(!comp.is_compressible(None));
    }

    #[test]
    fn test_algorithm_encoding_name() {
        assert_eq!(CompressionAlgorithm::Brotli.encoding_name(), "br");
        assert_eq!(CompressionAlgorithm::Gzip.encoding_name(), "gzip");
        assert_eq!(CompressionAlgorithm::Zstd.encoding_name(), "zstd");
    }
}
