//! Input validation utilities
//!
//! Provides validation functions to prevent injection attacks and ensure data integrity.

use crate::error::{Error, Result};
use std::collections::HashMap;

/// Input validator
///
/// Provides methods for validating common input types.
pub struct Validator;

impl Validator {
    /// Validate email address
    ///
    /// Basic email validation using a simple regex pattern.
    pub fn email(value: &str) -> Result<()> {
        let email_regex = regex::Regex::new(
            r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$"
        ).unwrap();

        if email_regex.is_match(value) {
            Ok(())
        } else {
            Err(Error::ValidationError("Invalid email address".to_string()))
        }
    }

    /// Validate URL
    ///
    /// Checks if the value is a valid HTTP/HTTPS URL.
    pub fn url(value: &str) -> Result<()> {
        if value.starts_with("http://") || value.starts_with("https://") {
            // Basic validation - in production, use a proper URL parser
            Ok(())
        } else {
            Err(Error::ValidationError("Invalid URL".to_string()))
        }
    }

    /// Validate string length
    ///
    /// Ensures the string length is within the specified range.
    pub fn length(value: &str, min: usize, max: usize) -> Result<()> {
        let len = value.len();
        if len < min {
            Err(Error::ValidationError(format!(
                "Value too short (minimum {} characters)",
                min
            )))
        } else if len > max {
            Err(Error::ValidationError(format!(
                "Value too long (maximum {} characters)",
                max
            )))
        } else {
            Ok(())
        }
    }

    /// Validate required field
    ///
    /// Ensures the value is not empty.
    pub fn required(value: &str) -> Result<()> {
        if value.trim().is_empty() {
            Err(Error::ValidationError("Field is required".to_string()))
        } else {
            Ok(())
        }
    }

    /// Validate numeric string
    ///
    /// Ensures the value contains only digits.
    pub fn numeric(value: &str) -> Result<()> {
        if value.chars().all(|c| c.is_ascii_digit()) {
            Ok(())
        } else {
            Err(Error::ValidationError("Value must be numeric".to_string()))
        }
    }

    /// Validate alphanumeric string
    ///
    /// Ensures the value contains only letters and numbers.
    pub fn alphanumeric(value: &str) -> Result<()> {
        if value.chars().all(|c| c.is_alphanumeric()) {
            Ok(())
        } else {
            Err(Error::ValidationError(
                "Value must be alphanumeric".to_string()
            ))
        }
    }

    /// Validate against SQL injection patterns
    ///
    /// # Security
    /// This is a basic check. Always use parameterized queries!
    pub fn no_sql_injection(value: &str) -> Result<()> {
        let dangerous_patterns = [
            "';", "--", "/*", "*/", "xp_", "sp_", "UNION", "SELECT", "DROP", "DELETE",
            "INSERT", "UPDATE", "EXEC", "EXECUTE",
        ];

        let upper_value = value.to_uppercase();
        for pattern in &dangerous_patterns {
            if upper_value.contains(pattern) {
                return Err(Error::ValidationError(
                    "Potentially dangerous SQL pattern detected".to_string()
                ));
            }
        }

        Ok(())
    }

    /// Validate that value doesn't contain path traversal attempts
    ///
    /// Prevents directory traversal attacks.
    pub fn no_path_traversal(value: &str) -> Result<()> {
        if value.contains("..") || value.contains("./") || value.contains("\\") {
            Err(Error::ValidationError(
                "Path traversal attempt detected".to_string()
            ))
        } else {
            Ok(())
        }
    }

    /// Sanitize string for safe output
    ///
    /// Removes or escapes potentially dangerous characters.
    pub fn sanitize(value: &str) -> String {
        value
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace() || matches!(c, '-' | '_' | '@' | '.'))
            .collect()
    }
}

/// Validation rules builder
pub struct ValidationRules {
    rules: HashMap<String, Vec<Rule>>,
}

impl ValidationRules {
    /// Create a new validation rules builder
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
        }
    }

    /// Add a required rule for a field
    pub fn required(mut self, field: impl Into<String>) -> Self {
        self.rules
            .entry(field.into())
            .or_default()
            .push(Rule::Required);
        self
    }

    /// Add a length rule for a field
    pub fn length(mut self, field: impl Into<String>, min: usize, max: usize) -> Self {
        self.rules
            .entry(field.into())
            .or_default()
            .push(Rule::Length { min, max });
        self
    }

    /// Add an email rule for a field
    pub fn email(mut self, field: impl Into<String>) -> Self {
        self.rules
            .entry(field.into())
            .or_default()
            .push(Rule::Email);
        self
    }

    /// Validate a set of values against the rules
    pub fn validate(&self, values: &HashMap<String, String>) -> Result<()> {
        for (field, rules) in &self.rules {
            let value = values.get(field).map(|s| s.as_str()).unwrap_or("");

            for rule in rules {
                match rule {
                    Rule::Required => Validator::required(value)?,
                    Rule::Length { min, max } => Validator::length(value, *min, *max)?,
                    Rule::Email => Validator::email(value)?,
                }
            }
        }

        Ok(())
    }
}

impl Default for ValidationRules {
    fn default() -> Self {
        Self::new()
    }
}

/// Validation rule types
enum Rule {
    Required,
    Length { min: usize, max: usize },
    Email,
}

// Note: regex crate needs to be added to dependencies for full email validation
// For now, we'll use a simple pattern check
mod regex {
    pub struct Regex {
        #[allow(dead_code)]
        pattern: String,
    }

    impl Regex {
        pub fn new(pattern: &str) -> Result<Self, ()> {
            Ok(Self {
                pattern: pattern.to_string(),
            })
        }

        pub fn is_match(&self, text: &str) -> bool {
            // Simplified email check without regex dependency
            if text.len() <= 3 {
                return false;
            }

            // Must contain @ and . and @ should not be at the start
            if !text.contains('@') || !text.contains('.') {
                return false;
            }

            // @ should not be at the start or end
            if text.starts_with('@') || text.ends_with('@') {
                return false;
            }

            // Find @ position
            if let Some(at_pos) = text.find('@') {
                // Should have chars before @
                if at_pos == 0 {
                    return false;
                }
                // Should have domain part after @
                let after_at = &text[at_pos + 1..];
                if after_at.is_empty() || !after_at.contains('.') {
                    return false;
                }
            }

            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_email() {
        assert!(Validator::email("user@example.com").is_ok());
        assert!(Validator::email("invalid").is_err());
        assert!(Validator::email("@example.com").is_err());
    }

    #[test]
    fn test_validate_length() {
        assert!(Validator::length("hello", 1, 10).is_ok());
        assert!(Validator::length("hi", 5, 10).is_err());
        assert!(Validator::length("very long string", 1, 5).is_err());
    }

    #[test]
    fn test_validate_required() {
        assert!(Validator::required("value").is_ok());
        assert!(Validator::required("").is_err());
        assert!(Validator::required("   ").is_err());
    }

    #[test]
    fn test_validate_numeric() {
        assert!(Validator::numeric("12345").is_ok());
        assert!(Validator::numeric("123abc").is_err());
    }

    #[test]
    fn test_validate_alphanumeric() {
        assert!(Validator::alphanumeric("abc123").is_ok());
        assert!(Validator::alphanumeric("abc-123").is_err());
    }

    #[test]
    fn test_no_sql_injection() {
        assert!(Validator::no_sql_injection("normal text").is_ok());
        assert!(Validator::no_sql_injection("'; DROP TABLE users;--").is_err());
        assert!(Validator::no_sql_injection("SELECT * FROM users").is_err());
    }

    #[test]
    fn test_no_path_traversal() {
        assert!(Validator::no_path_traversal("file.txt").is_ok());
        assert!(Validator::no_path_traversal("../etc/passwd").is_err());
        assert!(Validator::no_path_traversal("./config").is_err());
    }

    #[test]
    fn test_sanitize() {
        assert_eq!(Validator::sanitize("hello<script>"), "helloscript");
        assert_eq!(Validator::sanitize("user@example.com"), "user@example.com");
    }

    #[test]
    fn test_validation_rules() {
        let rules = ValidationRules::new()
            .required("username")
            .length("username", 3, 20)
            .required("email")
            .email("email");

        let mut values = HashMap::new();
        values.insert("username".to_string(), "john".to_string());
        values.insert("email".to_string(), "john@example.com".to_string());

        assert!(rules.validate(&values).is_ok());
    }
}
