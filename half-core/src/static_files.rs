use crate::{Error, Response, Result};
use mime_guess::from_path;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncReadExt;

/// Static file serving configuration
#[derive(Debug, Clone)]
pub struct StaticConfig {
    /// Root directory for static files
    pub root: PathBuf,
    /// Index file name (e.g., "index.html")
    pub index: Option<String>,
    /// Enable directory listing
    pub dir_listing: bool,
    /// Add cache headers (max-age in seconds)
    pub cache_max_age: Option<u64>,
    /// Follow symlinks
    pub follow_symlinks: bool,
}

impl Default for StaticConfig {
    fn default() -> Self {
        Self {
            root: PathBuf::from("./public"),
            index: Some("index.html".to_string()),
            dir_listing: false,
            cache_max_age: Some(3600), // 1 hour
            follow_symlinks: false,
        }
    }
}

impl StaticConfig {
    /// Create new static file configuration
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            ..Default::default()
        }
    }

    /// Set index file
    pub fn index(mut self, index: impl Into<String>) -> Self {
        self.index = Some(index.into());
        self
    }

    /// Enable/disable directory listing
    pub fn dir_listing(mut self, enable: bool) -> Self {
        self.dir_listing = enable;
        self
    }

    /// Set cache max-age
    pub fn cache_max_age(mut self, seconds: u64) -> Self {
        self.cache_max_age = Some(seconds);
        self
    }

    /// Enable/disable symlink following
    pub fn follow_symlinks(mut self, follow: bool) -> Self {
        self.follow_symlinks = follow;
        self
    }
}

/// Static file server
pub struct StaticFileServer {
    config: StaticConfig,
}

impl StaticFileServer {
    /// Create new static file server
    pub fn new(config: StaticConfig) -> Self {
        Self { config }
    }

    /// Serve a file from the configured root directory
    pub async fn serve(&self, path: &str) -> Result<Response> {
        // Security: Prevent path traversal
        if path.contains("..") {
            return Err(Error::BadRequest("Invalid path".into()));
        }

        let requested_path = path.trim_start_matches('/');
        let file_path = self.config.root.join(requested_path);

        // Check if file exists
        if !file_path.exists() {
            return Err(Error::BadRequest("File not found".into()));
        }

        // Read file
        let mut file = fs::File::open(&file_path)
            .await
            .map_err(|_| Error::BadRequest("Cannot open file".into()))?;

        let mut contents = Vec::new();
        file.read_to_end(&mut contents)
            .await
            .map_err(|e| Error::InternalError(format!("Failed to read file: {}", e)))?;

        // Guess MIME type
        let mime_type = from_path(&file_path).first_or_octet_stream().to_string();

        // Build response
        let mut response = Response::new()
            .body(contents)
            .header_str("Content-Type", &mime_type);

        // Add cache headers if configured
        if let Some(max_age) = self.config.cache_max_age {
            response =
                response.header_str("Cache-Control", &format!("public, max-age={}", max_age));
        }

        Ok(response)
    }

    /// Get configuration
    pub fn config(&self) -> &StaticConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_static_config_default() {
        let config = StaticConfig::default();
        assert_eq!(config.root, PathBuf::from("./public"));
        assert_eq!(config.index, Some("index.html".to_string()));
        assert!(!config.dir_listing);
        assert_eq!(config.cache_max_age, Some(3600));
        assert!(!config.follow_symlinks);
    }

    #[test]
    fn test_static_config_builder() {
        let config = StaticConfig::new("/var/www")
            .index("home.html")
            .dir_listing(true)
            .cache_max_age(7200)
            .follow_symlinks(true);

        assert_eq!(config.root, PathBuf::from("/var/www"));
        assert_eq!(config.index, Some("home.html".to_string()));
        assert!(config.dir_listing);
        assert_eq!(config.cache_max_age, Some(7200));
        assert!(config.follow_symlinks);
    }

    #[test]
    fn test_static_file_server_creation() {
        let config = StaticConfig::new("./public");
        let server = StaticFileServer::new(config);
        assert_eq!(server.config().root, PathBuf::from("./public"));
    }

    #[test]
    fn test_path_traversal_prevention() {
        let path = "../etc/passwd";
        assert!(path.contains(".."));
    }

    #[tokio::test]
    async fn test_serve_nonexistent_file() {
        let config = StaticConfig::new("./public");
        let server = StaticFileServer::new(config);

        let result = server.serve("/nonexistent-file-12345.txt").await;
        assert!(result.is_err());
    }
}
