use base64::{Engine, engine::general_purpose::STANDARD};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Authentication error
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthError {
    /// Invalid credentials
    InvalidCredentials,
    /// Invalid token
    InvalidToken,
    /// Expired token
    ExpiredToken,
    /// Missing authorization header
    MissingAuth,
    /// Invalid authorization header format
    InvalidAuthFormat,
    /// Permission denied
    PermissionDenied,
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::InvalidCredentials => write!(f, "Invalid credentials"),
            AuthError::InvalidToken => write!(f, "Invalid token"),
            AuthError::ExpiredToken => write!(f, "Token expired"),
            AuthError::MissingAuth => write!(f, "Missing authorization header"),
            AuthError::InvalidAuthFormat => write!(f, "Invalid authorization format"),
            AuthError::PermissionDenied => write!(f, "Permission denied"),
        }
    }
}

impl std::error::Error for AuthError {}

/// JWT Claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    /// Expiration time (Unix timestamp)
    pub exp: u64,
    /// Issued at (Unix timestamp)
    pub iat: u64,
    /// Issuer
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iss: Option<String>,
    /// Audience
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aud: Option<String>,
    /// Custom claims
    #[serde(flatten)]
    pub custom: HashMap<String, serde_json::Value>,
}

impl Claims {
    /// Create new claims
    pub fn new(sub: impl Into<String>, exp_seconds: u64) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            sub: sub.into(),
            exp: now + exp_seconds,
            iat: now,
            iss: None,
            aud: None,
            custom: HashMap::new(),
        }
    }

    /// Set issuer
    pub fn issuer(mut self, iss: impl Into<String>) -> Self {
        self.iss = Some(iss.into());
        self
    }

    /// Set audience
    pub fn audience(mut self, aud: impl Into<String>) -> Self {
        self.aud = Some(aud.into());
        self
    }

    /// Add custom claim
    pub fn claim(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.custom.insert(key.into(), value);
        self
    }

    /// Get custom claim
    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.custom.get(key)
    }
}

/// JWT Authentication
#[derive(Debug, Clone)]
pub struct JwtAuth {
    secret: Arc<String>,
    algorithm: Algorithm,
}

impl JwtAuth {
    /// Create new JWT authenticator
    pub fn new(secret: impl Into<String>) -> Self {
        Self {
            secret: Arc::new(secret.into()),
            algorithm: Algorithm::HS256,
        }
    }

    /// Set algorithm
    pub fn algorithm(mut self, algorithm: Algorithm) -> Self {
        self.algorithm = algorithm;
        self
    }

    /// Generate a JWT token
    pub fn generate(&self, claims: &Claims) -> Result<String, AuthError> {
        let key = EncodingKey::from_secret(self.secret.as_bytes());
        let header = Header {
            alg: self.algorithm,
            ..Default::default()
        };

        encode(&header, claims, &key).map_err(|_| AuthError::InvalidToken)
    }

    /// Verify a JWT token
    pub fn verify(&self, token: &str) -> Result<Claims, AuthError> {
        let key = DecodingKey::from_secret(self.secret.as_bytes());
        let mut validation = Validation::new(self.algorithm);
        validation.validate_exp = true;

        decode::<Claims>(token, &key, &validation)
            .map(|data| data.claims)
            .map_err(|e| {
                if e.to_string().contains("expired") {
                    AuthError::ExpiredToken
                } else {
                    AuthError::InvalidToken
                }
            })
    }

    /// Parse bearer token from Authorization header
    pub fn parse_bearer(header: &str) -> Result<String, AuthError> {
        if !header.starts_with("Bearer ") {
            return Err(AuthError::InvalidAuthFormat);
        }

        Ok(header[7..].trim().to_string())
    }
}

/// Basic Authentication
#[derive(Debug, Clone)]
pub struct BasicAuth {
    credentials: Arc<HashMap<String, String>>,
}

impl BasicAuth {
    /// Create new basic auth
    pub fn new() -> Self {
        Self {
            credentials: Arc::new(HashMap::new()),
        }
    }

    /// Add a user
    pub fn add_user(mut self, username: impl Into<String>, password: impl Into<String>) -> Self {
        let mut creds = (*self.credentials).clone();
        creds.insert(username.into(), password.into());
        self.credentials = Arc::new(creds);
        self
    }

    /// Verify credentials
    pub fn verify(&self, username: &str, password: &str) -> Result<(), AuthError> {
        if let Some(stored_password) = self.credentials.get(username) {
            if stored_password == password {
                return Ok(());
            }
        }
        Err(AuthError::InvalidCredentials)
    }

    /// Parse basic auth from Authorization header
    pub fn parse_basic(header: &str) -> Result<(String, String), AuthError> {
        if !header.starts_with("Basic ") {
            return Err(AuthError::InvalidAuthFormat);
        }

        let encoded = &header[6..].trim();
        let decoded = STANDARD
            .decode(encoded)
            .map_err(|_| AuthError::InvalidAuthFormat)?;

        let decoded_str = String::from_utf8(decoded).map_err(|_| AuthError::InvalidAuthFormat)?;

        let parts: Vec<&str> = decoded_str.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(AuthError::InvalidAuthFormat);
        }

        Ok((parts[0].to_string(), parts[1].to_string()))
    }

    /// Authenticate from Authorization header
    pub fn authenticate(&self, header: &str) -> Result<String, AuthError> {
        let (username, password) = Self::parse_basic(header)?;
        self.verify(&username, &password)?;
        Ok(username)
    }
}

impl Default for BasicAuth {
    fn default() -> Self {
        Self::new()
    }
}

/// API Key Authentication
#[derive(Debug, Clone)]
pub struct ApiKeyAuth {
    keys: Arc<HashMap<String, String>>,
    header_name: String,
}

impl ApiKeyAuth {
    /// Create new API key auth
    pub fn new() -> Self {
        Self {
            keys: Arc::new(HashMap::new()),
            header_name: "X-API-Key".to_string(),
        }
    }

    /// Set header name
    pub fn header_name(mut self, name: impl Into<String>) -> Self {
        self.header_name = name.into();
        self
    }

    /// Add an API key
    pub fn add_key(mut self, key: impl Into<String>, user_id: impl Into<String>) -> Self {
        let mut keys = (*self.keys).clone();
        keys.insert(key.into(), user_id.into());
        self.keys = Arc::new(keys);
        self
    }

    /// Verify an API key
    pub fn verify(&self, key: &str) -> Result<String, AuthError> {
        self.keys
            .get(key)
            .cloned()
            .ok_or(AuthError::InvalidCredentials)
    }

    /// Get header name
    pub fn get_header_name(&self) -> &str {
        &self.header_name
    }
}

impl Default for ApiKeyAuth {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claims_creation() {
        let claims = Claims::new("user123", 3600);
        assert_eq!(claims.sub, "user123");
        assert!(claims.exp > claims.iat);
    }

    #[test]
    fn test_claims_builder() {
        let claims = Claims::new("user123", 3600)
            .issuer("half-framework")
            .audience("api")
            .claim("role", serde_json::json!("admin"));

        assert_eq!(claims.iss.as_ref().unwrap(), "half-framework");
        assert_eq!(claims.aud.as_ref().unwrap(), "api");
        assert_eq!(claims.get("role").unwrap(), &serde_json::json!("admin"));
    }

    #[test]
    fn test_jwt_generate_and_verify() {
        let jwt = JwtAuth::new("secret");
        let claims = Claims::new("user123", 3600);

        let token = jwt.generate(&claims).unwrap();
        let verified = jwt.verify(&token).unwrap();

        assert_eq!(verified.sub, "user123");
    }

    #[test]
    fn test_jwt_invalid_token() {
        let jwt = JwtAuth::new("secret");
        let result = jwt.verify("invalid.token.here");
        assert_eq!(result.unwrap_err(), AuthError::InvalidToken);
    }

    #[test]
    fn test_jwt_parse_bearer() {
        let token = JwtAuth::parse_bearer("Bearer abc123").unwrap();
        assert_eq!(token, "abc123");

        let err = JwtAuth::parse_bearer("Invalid abc123").unwrap_err();
        assert_eq!(err, AuthError::InvalidAuthFormat);
    }

    #[test]
    fn test_basic_auth() {
        let auth = BasicAuth::new()
            .add_user("alice", "password123")
            .add_user("bob", "secret456");

        assert!(auth.verify("alice", "password123").is_ok());
        assert!(auth.verify("bob", "secret456").is_ok());
        assert_eq!(
            auth.verify("alice", "wrong").unwrap_err(),
            AuthError::InvalidCredentials
        );
    }

    #[test]
    fn test_basic_auth_parse() {
        let encoded = STANDARD.encode("alice:password123");
        let header = format!("Basic {}", encoded);

        let (username, password) = BasicAuth::parse_basic(&header).unwrap();
        assert_eq!(username, "alice");
        assert_eq!(password, "password123");
    }

    #[test]
    fn test_basic_auth_authenticate() {
        let auth = BasicAuth::new().add_user("alice", "password123");

        let encoded = STANDARD.encode("alice:password123");
        let header = format!("Basic {}", encoded);

        let username = auth.authenticate(&header).unwrap();
        assert_eq!(username, "alice");
    }

    #[test]
    fn test_api_key_auth() {
        let auth = ApiKeyAuth::new()
            .add_key("key123", "user1")
            .add_key("key456", "user2");

        assert_eq!(auth.verify("key123").unwrap(), "user1");
        assert_eq!(auth.verify("key456").unwrap(), "user2");
        assert_eq!(
            auth.verify("invalid").unwrap_err(),
            AuthError::InvalidCredentials
        );
    }

    #[test]
    fn test_api_key_auth_custom_header() {
        let auth = ApiKeyAuth::new().header_name("Authorization");
        assert_eq!(auth.get_header_name(), "Authorization");
    }
}
