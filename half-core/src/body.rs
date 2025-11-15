use bytes::Bytes;
use serde::de::DeserializeOwned;
use std::collections::HashMap;

/// Body parsing error
#[derive(Debug)]
pub enum BodyError {
    /// Body too large
    TooLarge { size: usize, max: usize },
    /// Invalid content type
    InvalidContentType(String),
    /// JSON parse error
    JsonParse(serde_json::Error),
    /// Form parse error
    FormParse(String),
    /// UTF-8 decode error
    Utf8Error(std::string::FromUtf8Error),
}

impl std::fmt::Display for BodyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BodyError::TooLarge { size, max } => {
                write!(f, "Body too large: {} bytes (max: {} bytes)", size, max)
            }
            BodyError::InvalidContentType(ct) => write!(f, "Invalid content type: {}", ct),
            BodyError::JsonParse(e) => write!(f, "JSON parse error: {}", e),
            BodyError::FormParse(e) => write!(f, "Form parse error: {}", e),
            BodyError::Utf8Error(e) => write!(f, "UTF-8 decode error: {}", e),
        }
    }
}

impl std::error::Error for BodyError {}

impl From<serde_json::Error> for BodyError {
    fn from(e: serde_json::Error) -> Self {
        BodyError::JsonParse(e)
    }
}

impl From<std::string::FromUtf8Error> for BodyError {
    fn from(e: std::string::FromUtf8Error) -> Self {
        BodyError::Utf8Error(e)
    }
}

/// Body parser configuration
#[derive(Debug, Clone)]
pub struct BodyConfig {
    /// Maximum body size in bytes (default: 1MB)
    pub max_size: usize,
    /// Allowed content types (empty = allow all)
    pub allowed_types: Vec<String>,
}

impl Default for BodyConfig {
    fn default() -> Self {
        Self {
            max_size: 1024 * 1024, // 1 MB
            allowed_types: Vec::new(),
        }
    }
}

impl BodyConfig {
    /// Create new body configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum body size
    pub fn max_size(mut self, size: usize) -> Self {
        self.max_size = size;
        self
    }

    /// Set allowed content types
    pub fn allowed_types(mut self, types: Vec<String>) -> Self {
        self.allowed_types = types;
        self
    }
}

/// Body parser
pub struct BodyParser {
    config: BodyConfig,
}

impl BodyParser {
    /// Create new body parser
    pub fn new(config: BodyConfig) -> Self {
        Self { config }
    }

    /// Parse JSON body
    pub fn parse_json<T: DeserializeOwned>(
        &self,
        body: &[u8],
        content_type: Option<&str>,
    ) -> Result<T, BodyError> {
        // Check size
        if body.len() > self.config.max_size {
            return Err(BodyError::TooLarge {
                size: body.len(),
                max: self.config.max_size,
            });
        }

        // Check content type if provided
        if let Some(ct) = content_type {
            if !ct.contains("application/json") {
                return Err(BodyError::InvalidContentType(ct.to_string()));
            }
        }

        // Parse JSON
        serde_json::from_slice(body).map_err(BodyError::JsonParse)
    }

    /// Parse form-urlencoded body
    pub fn parse_form(
        &self,
        body: &[u8],
        content_type: Option<&str>,
    ) -> Result<HashMap<String, String>, BodyError> {
        // Check size
        if body.len() > self.config.max_size {
            return Err(BodyError::TooLarge {
                size: body.len(),
                max: self.config.max_size,
            });
        }

        // Check content type if provided
        if let Some(ct) = content_type {
            if !ct.contains("application/x-www-form-urlencoded") {
                return Err(BodyError::InvalidContentType(ct.to_string()));
            }
        }

        // Convert to string
        let body_str = String::from_utf8(body.to_vec())?;

        // Parse form data
        let mut params = HashMap::new();

        for pair in body_str.split('&') {
            if pair.is_empty() {
                continue;
            }

            let parts: Vec<&str> = pair.splitn(2, '=').collect();
            if parts.len() == 2 {
                // Replace + with space before decoding (form-urlencoded spec)
                let key_raw = parts[0].replace('+', " ");
                let key = urlencoding::decode(&key_raw)
                    .map_err(|e| BodyError::FormParse(e.to_string()))?
                    .into_owned();

                let value_raw = parts[1].replace('+', " ");
                let value = urlencoding::decode(&value_raw)
                    .map_err(|e| BodyError::FormParse(e.to_string()))?
                    .into_owned();

                params.insert(key, value);
            } else if parts.len() == 1 {
                let key_raw = parts[0].replace('+', " ");
                let key = urlencoding::decode(&key_raw)
                    .map_err(|e| BodyError::FormParse(e.to_string()))?
                    .into_owned();
                params.insert(key, String::new());
            }
        }

        Ok(params)
    }

    /// Parse body based on content type
    pub fn parse(&self, body: &[u8], content_type: Option<&str>) -> Result<BodyData, BodyError> {
        if let Some(ct) = content_type {
            if ct.contains("application/json") {
                let value: serde_json::Value = self.parse_json(body, Some(ct))?;
                return Ok(BodyData::Json(value));
            } else if ct.contains("application/x-www-form-urlencoded") {
                let params = self.parse_form(body, Some(ct))?;
                return Ok(BodyData::Form(params));
            } else if ct.contains("text/plain") {
                let text = String::from_utf8(body.to_vec())?;
                return Ok(BodyData::Text(text));
            }
        }

        // Try to parse as JSON first
        if let Ok(value) = serde_json::from_slice::<serde_json::Value>(body) {
            return Ok(BodyData::Json(value));
        }

        // Fallback to raw bytes
        Ok(BodyData::Raw(Bytes::copy_from_slice(body)))
    }

    /// Get configuration
    pub fn config(&self) -> &BodyConfig {
        &self.config
    }
}

impl Default for BodyParser {
    fn default() -> Self {
        Self::new(BodyConfig::default())
    }
}

/// Parsed body data
#[derive(Debug, Clone)]
pub enum BodyData {
    /// JSON data
    Json(serde_json::Value),
    /// Form-urlencoded data
    Form(HashMap<String, String>),
    /// Plain text
    Text(String),
    /// Raw bytes
    Raw(Bytes),
}

impl BodyData {
    /// Try to get as JSON
    pub fn as_json(&self) -> Option<&serde_json::Value> {
        match self {
            BodyData::Json(v) => Some(v),
            _ => None,
        }
    }

    /// Try to get as form data
    pub fn as_form(&self) -> Option<&HashMap<String, String>> {
        match self {
            BodyData::Form(f) => Some(f),
            _ => None,
        }
    }

    /// Try to get as text
    pub fn as_text(&self) -> Option<&str> {
        match self {
            BodyData::Text(t) => Some(t),
            _ => None,
        }
    }

    /// Try to get as raw bytes
    pub fn as_raw(&self) -> Option<&Bytes> {
        match self {
            BodyData::Raw(b) => Some(b),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_body_config_default() {
        let config = BodyConfig::default();
        assert_eq!(config.max_size, 1024 * 1024);
        assert!(config.allowed_types.is_empty());
    }

    #[test]
    fn test_body_config_builder() {
        let config = BodyConfig::new()
            .max_size(2048)
            .allowed_types(vec!["application/json".to_string()]);

        assert_eq!(config.max_size, 2048);
        assert_eq!(config.allowed_types.len(), 1);
    }

    #[test]
    fn test_parse_json() {
        let parser = BodyParser::default();
        let body = r#"{"name":"John","age":30}"#.as_bytes();

        let result: serde_json::Value = parser.parse_json(body, Some("application/json")).unwrap();

        assert_eq!(result["name"], "John");
        assert_eq!(result["age"], 30);
    }

    #[test]
    fn test_parse_json_invalid_content_type() {
        let parser = BodyParser::default();
        let body = r#"{"name":"John"}"#.as_bytes();

        let result: Result<serde_json::Value, _> = parser.parse_json(body, Some("text/plain"));

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_form() {
        let parser = BodyParser::default();
        let body = b"name=John&age=30&city=New+York";

        let result = parser
            .parse_form(body, Some("application/x-www-form-urlencoded"))
            .unwrap();

        assert_eq!(result.get("name"), Some(&"John".to_string()));
        assert_eq!(result.get("age"), Some(&"30".to_string()));
        assert_eq!(result.get("city"), Some(&"New York".to_string()));
    }

    #[test]
    fn test_parse_form_empty_value() {
        let parser = BodyParser::default();
        let body = b"key1=&key2=value";

        let result = parser
            .parse_form(body, Some("application/x-www-form-urlencoded"))
            .unwrap();

        assert_eq!(result.get("key1"), Some(&String::new()));
        assert_eq!(result.get("key2"), Some(&"value".to_string()));
    }

    #[test]
    fn test_parse_body_too_large() {
        let config = BodyConfig::new().max_size(10);
        let parser = BodyParser::new(config);
        let body = b"This is a very long body that exceeds the limit";

        let result: Result<serde_json::Value, _> =
            parser.parse_json(body, Some("application/json"));

        assert!(matches!(result, Err(BodyError::TooLarge { .. })));
    }

    #[test]
    fn test_body_data_as_json() {
        let data = BodyData::Json(json!({"key": "value"}));
        assert!(data.as_json().is_some());
        assert!(data.as_form().is_none());
    }

    #[test]
    fn test_body_data_as_form() {
        let mut form = HashMap::new();
        form.insert("key".to_string(), "value".to_string());
        let data = BodyData::Form(form);

        assert!(data.as_form().is_some());
        assert!(data.as_json().is_none());
    }

    #[test]
    fn test_parse_auto_detect_json() {
        let parser = BodyParser::default();
        let body = r#"{"test": true}"#.as_bytes();

        let result = parser.parse(body, None).unwrap();
        assert!(result.as_json().is_some());
    }
}
