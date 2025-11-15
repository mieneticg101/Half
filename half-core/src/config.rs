//! Configuration management for Half framework
//!
//! Provides utilities for loading and managing application configuration
//! from multiple sources (environment variables, config files, defaults).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;

/// Application environment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Environment {
    /// Development environment
    #[default]
    Development,
    /// Staging environment
    Staging,
    /// Production environment
    Production,
    /// Test environment
    Test,
}

impl Environment {
    /// Get current environment from environment variable
    ///
    /// Reads from `APP_ENV`, `ENVIRONMENT`, or `ENV` variables.
    /// Defaults to Development if not set.
    pub fn current() -> Self {
        let env_str = env::var("APP_ENV")
            .or_else(|_| env::var("ENVIRONMENT"))
            .or_else(|_| env::var("ENV"))
            .unwrap_or_else(|_| "development".to_string());

        match env_str.to_lowercase().as_str() {
            "production" | "prod" => Environment::Production,
            "staging" | "stage" => Environment::Staging,
            "test" | "testing" => Environment::Test,
            _ => Environment::Development,
        }
    }

    /// Check if running in production
    pub fn is_production(&self) -> bool {
        matches!(self, Environment::Production)
    }

    /// Check if running in development
    pub fn is_development(&self) -> bool {
        matches!(self, Environment::Development)
    }

    /// Check if running in test
    pub fn is_test(&self) -> bool {
        matches!(self, Environment::Test)
    }

    /// Get environment as string
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Development => "development",
            Environment::Staging => "staging",
            Environment::Production => "production",
            Environment::Test => "test",
        }
    }
}

impl std::fmt::Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Configuration builder
pub struct ConfigBuilder {
    env_prefix: String,
    config_path: Option<String>,
    defaults: HashMap<String, String>,
    environment: Environment,
}

impl ConfigBuilder {
    /// Create a new configuration builder
    pub fn new() -> Self {
        Self {
            env_prefix: String::new(),
            config_path: None,
            defaults: HashMap::new(),
            environment: Environment::current(),
        }
    }

    /// Set environment variable prefix
    ///
    /// When set, only environment variables with this prefix will be loaded.
    /// The prefix is stripped from the variable name.
    ///
    /// # Example
    /// ```ignore
    /// let config = ConfigBuilder::new()
    ///     .env_prefix("APP_")
    ///     .build();
    /// // Will load APP_PORT as "port"
    /// ```
    pub fn env_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.env_prefix = prefix.into();
        self
    }

    /// Set configuration file path
    ///
    /// Supports JSON and TOML formats based on file extension.
    pub fn config_file(mut self, path: impl Into<String>) -> Self {
        self.config_path = Some(path.into());
        self
    }

    /// Set a default value
    pub fn default(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.defaults.insert(key.into(), value.into());
        self
    }

    /// Set the environment
    pub fn environment(mut self, env: Environment) -> Self {
        self.environment = env;
        self
    }

    /// Build the configuration
    pub fn build(self) -> Config {
        let mut values = HashMap::new();

        // 1. Load defaults
        values.extend(self.defaults);

        // 2. Load from config file if specified
        if let Some(path) = &self.config_path {
            if let Ok(file_values) = Self::load_file(path) {
                values.extend(file_values);
            }
        }

        // 3. Load from environment variables (highest priority)
        let env_values = Self::load_env(&self.env_prefix);
        values.extend(env_values);

        Config {
            values,
            environment: self.environment,
        }
    }

    /// Load configuration from file
    fn load_file(path: &str) -> Result<HashMap<String, String>, std::io::Error> {
        let content = std::fs::read_to_string(path)?;

        // Parse based on file extension
        if path.ends_with(".json") {
            Self::parse_json(&content)
        } else if path.ends_with(".toml") {
            Self::parse_toml(&content)
        } else {
            // Try JSON first, then TOML
            Self::parse_json(&content).or_else(|_| Self::parse_toml(&content))
        }
    }

    /// Parse JSON configuration
    fn parse_json(content: &str) -> Result<HashMap<String, String>, std::io::Error> {
        let json: serde_json::Value = serde_json::from_str(content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        Ok(Self::flatten_json("", &json))
    }

    /// Flatten JSON into dot-notation keys
    fn flatten_json(prefix: &str, value: &serde_json::Value) -> HashMap<String, String> {
        let mut result = HashMap::new();

        match value {
            serde_json::Value::Object(map) => {
                for (key, val) in map {
                    let new_prefix = if prefix.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", prefix, key)
                    };
                    result.extend(Self::flatten_json(&new_prefix, val));
                }
            }
            serde_json::Value::Array(arr) => {
                for (i, val) in arr.iter().enumerate() {
                    let new_prefix = format!("{}.{}", prefix, i);
                    result.extend(Self::flatten_json(&new_prefix, val));
                }
            }
            _ => {
                if !prefix.is_empty() {
                    result.insert(
                        prefix.to_string(),
                        value.to_string().trim_matches('"').to_string(),
                    );
                }
            }
        }

        result
    }

    /// Parse TOML configuration (simplified - only supports flat structure)
    fn parse_toml(content: &str) -> Result<HashMap<String, String>, std::io::Error> {
        let mut result = HashMap::new();

        for line in content.lines() {
            let line = line.trim();

            // Skip comments and empty lines
            if line.starts_with('#') || line.is_empty() || line.starts_with('[') {
                continue;
            }

            // Parse key = value
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim().to_string();
                let value = value
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string();
                result.insert(key, value);
            }
        }

        Ok(result)
    }

    /// Load environment variables
    fn load_env(prefix: &str) -> HashMap<String, String> {
        env::vars()
            .filter_map(|(key, value)| {
                if prefix.is_empty() {
                    Some((key.to_lowercase(), value))
                } else if let Some(stripped) = key.strip_prefix(prefix) {
                    let stripped_key = stripped.to_lowercase();
                    Some((stripped_key, value))
                } else {
                    None
                }
            })
            .collect()
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Application configuration
#[derive(Debug, Clone)]
pub struct Config {
    values: HashMap<String, String>,
    environment: Environment,
}

impl Config {
    /// Create a new empty configuration
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            environment: Environment::current(),
        }
    }

    /// Get environment
    pub fn environment(&self) -> Environment {
        self.environment
    }

    /// Get a configuration value as string
    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }

    /// Get a configuration value as string with default
    pub fn get_or(&self, key: &str, default: &str) -> String {
        self.get(key)
            .cloned()
            .unwrap_or_else(|| default.to_string())
    }

    /// Get a configuration value and parse it
    pub fn get_parse<T: std::str::FromStr>(&self, key: &str) -> Option<T> {
        self.get(key).and_then(|v| v.parse().ok())
    }

    /// Get a configuration value and parse it with default
    pub fn get_parse_or<T: std::str::FromStr>(&self, key: &str, default: T) -> T {
        self.get_parse(key).unwrap_or(default)
    }

    /// Get a boolean value
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.get(key).and_then(|v| match v.to_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => Some(true),
            "false" | "0" | "no" | "off" => Some(false),
            _ => v.parse().ok(),
        })
    }

    /// Get an integer value
    pub fn get_int(&self, key: &str) -> Option<i64> {
        self.get_parse(key)
    }

    /// Get a float value
    pub fn get_float(&self, key: &str) -> Option<f64> {
        self.get_parse(key)
    }

    /// Check if a key exists
    pub fn has(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }

    /// Get all keys
    pub fn keys(&self) -> Vec<&String> {
        self.values.keys().collect()
    }

    /// Get all values as a map
    pub fn all(&self) -> &HashMap<String, String> {
        &self.values
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_detection() {
        unsafe {
            env::set_var("APP_ENV", "production");
        }
        assert_eq!(Environment::current(), Environment::Production);

        unsafe {
            env::set_var("APP_ENV", "development");
        }
        assert_eq!(Environment::current(), Environment::Development);

        unsafe {
            env::remove_var("APP_ENV");
            env::remove_var("ENVIRONMENT");
            env::remove_var("ENV");
        }
        assert_eq!(Environment::current(), Environment::Development);
    }

    #[test]
    fn test_environment_methods() {
        let prod = Environment::Production;
        assert!(prod.is_production());
        assert!(!prod.is_development());
        assert_eq!(prod.as_str(), "production");

        let dev = Environment::Development;
        assert!(dev.is_development());
        assert!(!dev.is_production());
    }

    #[test]
    fn test_config_builder() {
        let config = ConfigBuilder::new()
            .default("port", "3000")
            .default("host", "127.0.0.1")
            .build();

        assert_eq!(config.get("port"), Some(&"3000".to_string()));
        assert_eq!(config.get("host"), Some(&"127.0.0.1".to_string()));
    }

    #[test]
    fn test_config_parsing() {
        let config = ConfigBuilder::new()
            .default("port", "8080")
            .default("debug", "true")
            .default("rate", "1.5")
            .build();

        assert_eq!(config.get_int("port"), Some(8080));
        assert_eq!(config.get_bool("debug"), Some(true));
        assert_eq!(config.get_float("rate"), Some(1.5));
    }

    #[test]
    fn test_config_bool_values() {
        let config = ConfigBuilder::new()
            .default("enabled1", "true")
            .default("enabled2", "1")
            .default("enabled3", "yes")
            .default("disabled1", "false")
            .default("disabled2", "0")
            .default("disabled3", "no")
            .build();

        assert_eq!(config.get_bool("enabled1"), Some(true));
        assert_eq!(config.get_bool("enabled2"), Some(true));
        assert_eq!(config.get_bool("enabled3"), Some(true));
        assert_eq!(config.get_bool("disabled1"), Some(false));
        assert_eq!(config.get_bool("disabled2"), Some(false));
        assert_eq!(config.get_bool("disabled3"), Some(false));
    }

    #[test]
    fn test_json_parsing() {
        let json = r#"{"port": 8080, "host": "localhost"}"#;
        let values = ConfigBuilder::parse_json(json).unwrap();

        assert_eq!(values.get("port"), Some(&"8080".to_string()));
        assert_eq!(values.get("host"), Some(&"localhost".to_string()));
    }

    #[test]
    fn test_toml_parsing() {
        let toml = r#"
            port = 8080
            host = "localhost"
            # This is a comment
            debug = true
        "#;
        let values = ConfigBuilder::parse_toml(toml).unwrap();

        assert_eq!(values.get("port"), Some(&"8080".to_string()));
        assert_eq!(values.get("host"), Some(&"localhost".to_string()));
        assert_eq!(values.get("debug"), Some(&"true".to_string()));
    }
}
