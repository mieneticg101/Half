use bytes::Bytes;
use futures_util::stream;
use multer::Multipart;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

/// File upload error
#[derive(Debug)]
pub enum UploadError {
    /// File too large
    FileTooLarge { size: u64, max: u64 },
    /// Invalid content type
    InvalidContentType,
    /// Invalid field name
    InvalidFieldName,
    /// IO error
    Io(std::io::Error),
    /// Multipart error
    Multipart(multer::Error),
    /// Missing field
    MissingField(String),
}

impl std::fmt::Display for UploadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UploadError::FileTooLarge { size, max } => {
                write!(f, "File too large: {} bytes (max: {} bytes)", size, max)
            }
            UploadError::InvalidContentType => write!(f, "Invalid content type"),
            UploadError::InvalidFieldName => write!(f, "Invalid field name"),
            UploadError::Io(e) => write!(f, "IO error: {}", e),
            UploadError::Multipart(e) => write!(f, "Multipart error: {}", e),
            UploadError::MissingField(name) => write!(f, "Missing field: {}", name),
        }
    }
}

impl std::error::Error for UploadError {}

impl From<std::io::Error> for UploadError {
    fn from(e: std::io::Error) -> Self {
        UploadError::Io(e)
    }
}

impl From<multer::Error> for UploadError {
    fn from(e: multer::Error) -> Self {
        UploadError::Multipart(e)
    }
}

/// Uploaded file
#[derive(Debug, Clone)]
pub struct UploadedFile {
    /// Field name
    pub field_name: String,
    /// File name
    pub file_name: Option<String>,
    /// Content type
    pub content_type: Option<String>,
    /// File data
    pub data: Bytes,
    /// File size
    pub size: u64,
}

impl UploadedFile {
    /// Save file to disk
    pub async fn save(&self, path: impl AsRef<Path>) -> Result<PathBuf, UploadError> {
        let path = path.as_ref();
        let mut file = File::create(path).await?;
        file.write_all(&self.data).await?;
        Ok(path.to_path_buf())
    }

    /// Get file extension
    pub fn extension(&self) -> Option<&str> {
        self.file_name
            .as_ref()
            .and_then(|name| Path::new(name).extension())
            .and_then(|ext| ext.to_str())
    }

    /// Check if file is of a specific type
    pub fn is_type(&self, types: &[&str]) -> bool {
        if let Some(content_type) = &self.content_type {
            types.iter().any(|t| content_type.contains(t))
        } else {
            false
        }
    }

    /// Check if file is an image
    pub fn is_image(&self) -> bool {
        self.is_type(&["image/"])
    }

    /// Check if file is a document
    pub fn is_document(&self) -> bool {
        self.is_type(&["application/pdf", "application/msword", "text/"])
    }
}

/// Upload configuration
#[derive(Debug, Clone)]
pub struct UploadConfig {
    /// Maximum file size in bytes
    pub max_file_size: u64,
    /// Maximum total upload size in bytes
    pub max_total_size: u64,
    /// Allowed content types (empty = allow all)
    pub allowed_types: Vec<String>,
    /// Maximum number of files
    pub max_files: usize,
}

impl Default for UploadConfig {
    fn default() -> Self {
        Self {
            max_file_size: 10 * 1024 * 1024,   // 10 MB
            max_total_size: 100 * 1024 * 1024, // 100 MB
            allowed_types: Vec::new(),
            max_files: 10,
        }
    }
}

impl UploadConfig {
    /// Create new upload configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum file size
    pub fn max_file_size(mut self, size: u64) -> Self {
        self.max_file_size = size;
        self
    }

    /// Set maximum total size
    pub fn max_total_size(mut self, size: u64) -> Self {
        self.max_total_size = size;
        self
    }

    /// Set allowed content types
    pub fn allowed_types(mut self, types: Vec<String>) -> Self {
        self.allowed_types = types;
        self
    }

    /// Set maximum number of files
    pub fn max_files(mut self, max: usize) -> Self {
        self.max_files = max;
        self
    }

    /// Validate file
    pub fn validate(&self, file: &UploadedFile) -> Result<(), UploadError> {
        // Check file size
        if file.size > self.max_file_size {
            return Err(UploadError::FileTooLarge {
                size: file.size,
                max: self.max_file_size,
            });
        }

        // Check content type
        if !self.allowed_types.is_empty() {
            if let Some(content_type) = &file.content_type {
                let allowed = self.allowed_types.iter().any(|t| content_type.contains(t));
                if !allowed {
                    return Err(UploadError::InvalidContentType);
                }
            } else {
                return Err(UploadError::InvalidContentType);
            }
        }

        Ok(())
    }
}

/// File upload handler
pub struct FileUpload {
    config: UploadConfig,
}

impl FileUpload {
    /// Create new file upload handler
    pub fn new(config: UploadConfig) -> Self {
        Self { config }
    }

    /// Parse multipart form data
    pub async fn parse_multipart(
        &self,
        boundary: &str,
        body: impl Into<Bytes>,
    ) -> Result<MultipartData, UploadError> {
        let body: Bytes = body.into();
        // Convert Bytes to a Stream that multer expects
        let stream = stream::once(async move { Ok::<Bytes, std::io::Error>(body) });
        let mut multipart = Multipart::new(stream, boundary);

        let mut files = Vec::new();
        let mut fields = HashMap::new();
        let mut total_size = 0u64;

        while let Some(field) = multipart.next_field().await? {
            let field_name = field
                .name()
                .ok_or(UploadError::InvalidFieldName)?
                .to_string();

            let file_name = field.file_name().map(|s| s.to_string());
            let content_type = field.content_type().map(|s| s.to_string());

            let data = field.bytes().await?;
            let size = data.len() as u64;

            total_size += size;

            // Check total size
            if total_size > self.config.max_total_size {
                return Err(UploadError::FileTooLarge {
                    size: total_size,
                    max: self.config.max_total_size,
                });
            }

            // Check max files
            if files.len() >= self.config.max_files {
                return Err(UploadError::FileTooLarge {
                    size: files.len() as u64,
                    max: self.config.max_files as u64,
                });
            }

            if file_name.is_some() {
                // It's a file
                let file = UploadedFile {
                    field_name: field_name.clone(),
                    file_name,
                    content_type,
                    data,
                    size,
                };

                // Validate file
                self.config.validate(&file)?;

                files.push(file);
            } else {
                // It's a regular field
                let value = String::from_utf8_lossy(&data).to_string();
                fields.insert(field_name, value);
            }
        }

        Ok(MultipartData { files, fields })
    }

    /// Get configuration
    pub fn config(&self) -> &UploadConfig {
        &self.config
    }
}

/// Multipart form data
#[derive(Debug)]
pub struct MultipartData {
    /// Uploaded files
    pub files: Vec<UploadedFile>,
    /// Form fields
    pub fields: HashMap<String, String>,
}

impl MultipartData {
    /// Get file by field name
    pub fn get_file(&self, field_name: &str) -> Option<&UploadedFile> {
        self.files.iter().find(|f| f.field_name == field_name)
    }

    /// Get all files for a field name
    pub fn get_files(&self, field_name: &str) -> Vec<&UploadedFile> {
        self.files
            .iter()
            .filter(|f| f.field_name == field_name)
            .collect()
    }

    /// Get form field value
    pub fn get_field(&self, name: &str) -> Option<&str> {
        self.fields.get(name).map(|s| s.as_str())
    }

    /// Get required field
    pub fn require_field(&self, name: &str) -> Result<&str, UploadError> {
        self.get_field(name)
            .ok_or_else(|| UploadError::MissingField(name.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upload_config_default() {
        let config = UploadConfig::default();
        assert_eq!(config.max_file_size, 10 * 1024 * 1024);
        assert_eq!(config.max_total_size, 100 * 1024 * 1024);
        assert_eq!(config.max_files, 10);
        assert!(config.allowed_types.is_empty());
    }

    #[test]
    fn test_upload_config_builder() {
        let config = UploadConfig::new()
            .max_file_size(5 * 1024 * 1024)
            .max_total_size(50 * 1024 * 1024)
            .max_files(5)
            .allowed_types(vec!["image/".to_string()]);

        assert_eq!(config.max_file_size, 5 * 1024 * 1024);
        assert_eq!(config.max_total_size, 50 * 1024 * 1024);
        assert_eq!(config.max_files, 5);
        assert_eq!(config.allowed_types.len(), 1);
    }

    #[test]
    fn test_uploaded_file_extension() {
        let file = UploadedFile {
            field_name: "file".to_string(),
            file_name: Some("test.jpg".to_string()),
            content_type: Some("image/jpeg".to_string()),
            data: Bytes::from("test"),
            size: 4,
        };

        assert_eq!(file.extension(), Some("jpg"));
    }

    #[test]
    fn test_uploaded_file_is_image() {
        let file = UploadedFile {
            field_name: "file".to_string(),
            file_name: Some("test.jpg".to_string()),
            content_type: Some("image/jpeg".to_string()),
            data: Bytes::from("test"),
            size: 4,
        };

        assert!(file.is_image());
        assert!(!file.is_document());
    }

    #[test]
    fn test_uploaded_file_is_document() {
        let file = UploadedFile {
            field_name: "file".to_string(),
            file_name: Some("test.pdf".to_string()),
            content_type: Some("application/pdf".to_string()),
            data: Bytes::from("test"),
            size: 4,
        };

        assert!(file.is_document());
        assert!(!file.is_image());
    }

    #[test]
    fn test_validate_file_size() {
        let config = UploadConfig::new().max_file_size(100);

        let file = UploadedFile {
            field_name: "file".to_string(),
            file_name: Some("test.txt".to_string()),
            content_type: Some("text/plain".to_string()),
            data: Bytes::from(vec![0u8; 200]),
            size: 200,
        };

        assert!(matches!(
            config.validate(&file),
            Err(UploadError::FileTooLarge { .. })
        ));
    }

    #[test]
    fn test_validate_content_type() {
        let config = UploadConfig::new().allowed_types(vec!["image/".to_string()]);

        let file = UploadedFile {
            field_name: "file".to_string(),
            file_name: Some("test.txt".to_string()),
            content_type: Some("text/plain".to_string()),
            data: Bytes::from("test"),
            size: 4,
        };

        assert!(matches!(
            config.validate(&file),
            Err(UploadError::InvalidContentType)
        ));
    }

    #[test]
    fn test_multipart_data() {
        let mut data = MultipartData {
            files: vec![UploadedFile {
                field_name: "file1".to_string(),
                file_name: Some("test.jpg".to_string()),
                content_type: Some("image/jpeg".to_string()),
                data: Bytes::from("test"),
                size: 4,
            }],
            fields: HashMap::new(),
        };

        data.fields.insert("name".to_string(), "John".to_string());

        assert!(data.get_file("file1").is_some());
        assert_eq!(data.get_field("name"), Some("John"));
        assert!(data.require_field("name").is_ok());
        assert!(data.require_field("missing").is_err());
    }
}
