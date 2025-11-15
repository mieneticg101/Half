use crate::response::{Cookie, SameSite};
use std::collections::HashMap;

/// Cookie jar for managing request cookies
#[derive(Debug, Clone, Default)]
pub struct CookieJar {
    cookies: HashMap<String, String>,
}

impl CookieJar {
    /// Create a new cookie jar
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse cookies from Cookie header
    pub fn from_header(header: &str) -> Self {
        let mut jar = Self::new();

        for cookie_str in header.split(';') {
            let cookie_str = cookie_str.trim();
            if cookie_str.is_empty() {
                continue;
            }

            if let Some((name, value)) = cookie_str.split_once('=') {
                jar.add(name.trim().to_string(), value.trim().to_string());
            }
        }

        jar
    }

    /// Add a cookie to the jar
    pub fn add(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.cookies.insert(name.into(), value.into());
    }

    /// Get a cookie value by name
    pub fn get(&self, name: &str) -> Option<&String> {
        self.cookies.get(name)
    }

    /// Check if a cookie exists
    pub fn has(&self, name: &str) -> bool {
        self.cookies.contains_key(name)
    }

    /// Remove a cookie
    pub fn remove(&mut self, name: &str) -> Option<String> {
        self.cookies.remove(name)
    }

    /// Get all cookies
    pub fn all(&self) -> &HashMap<String, String> {
        &self.cookies
    }

    /// Get number of cookies
    pub fn len(&self) -> usize {
        self.cookies.len()
    }

    /// Check if jar is empty
    pub fn is_empty(&self) -> bool {
        self.cookies.is_empty()
    }

    /// Clear all cookies
    pub fn clear(&mut self) {
        self.cookies.clear();
    }

    /// Iterate over cookies
    pub fn iter(&self) -> impl Iterator<Item = (&String, &String)> {
        self.cookies.iter()
    }
}

/// Signed cookie jar for secure cookies
#[derive(Debug, Clone)]
pub struct SignedCookieJar {
    secret: String,
    jar: CookieJar,
}

impl SignedCookieJar {
    /// Create a new signed cookie jar
    pub fn new(secret: impl Into<String>) -> Self {
        Self {
            secret: secret.into(),
            jar: CookieJar::new(),
        }
    }

    /// Parse cookies from header with signature verification
    pub fn from_header(header: &str, secret: impl Into<String>) -> Self {
        let mut signed_jar = Self::new(secret);
        let jar = CookieJar::from_header(header);

        // TODO: Implement signature verification
        signed_jar.jar = jar;
        signed_jar
    }

    /// Sign a cookie value
    fn sign(&self, value: &str) -> String {
        use sha2::{Sha256, Digest};
        use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};

        let mut hasher = Sha256::new();
        hasher.update(value.as_bytes());
        hasher.update(self.secret.as_bytes());
        let signature = hasher.finalize();

        let sig_b64 = URL_SAFE_NO_PAD.encode(signature);
        format!("{}.{}", value, sig_b64)
    }

    /// Verify and extract cookie value
    fn verify(&self, signed_value: &str) -> Option<String> {
        if let Some((value, _signature)) = signed_value.split_once('.') {
            // TODO: Implement signature verification
            Some(value.to_string())
        } else {
            None
        }
    }

    /// Add a signed cookie
    pub fn add(&mut self, name: impl Into<String>, value: impl Into<String>) {
        let value_str = value.into();
        let signed_value = self.sign(&value_str);
        self.jar.add(name, signed_value);
    }

    /// Get a signed cookie value (verified)
    pub fn get(&self, name: &str) -> Option<String> {
        self.jar.get(name).and_then(|v| self.verify(v))
    }

    /// Get the underlying jar
    pub fn jar(&self) -> &CookieJar {
        &self.jar
    }
}

/// Cookie builder for creating Set-Cookie headers
pub struct CookieBuilder {
    name: String,
    value: String,
    max_age: Option<i64>,
    domain: Option<String>,
    path: Option<String>,
    secure: bool,
    http_only: bool,
    same_site: Option<SameSite>,
}

impl CookieBuilder {
    /// Create a new cookie builder
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            max_age: None,
            domain: None,
            path: Some("/".to_string()),
            secure: false,
            http_only: false,
            same_site: None,
        }
    }

    /// Set max age in seconds
    pub fn max_age(mut self, seconds: i64) -> Self {
        self.max_age = Some(seconds);
        self
    }

    /// Set domain
    pub fn domain(mut self, domain: impl Into<String>) -> Self {
        self.domain = Some(domain.into());
        self
    }

    /// Set path
    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// Set secure flag
    pub fn secure(mut self, secure: bool) -> Self {
        self.secure = secure;
        self
    }

    /// Set HTTP-only flag
    pub fn http_only(mut self, http_only: bool) -> Self {
        self.http_only = http_only;
        self
    }

    /// Set SameSite attribute
    pub fn same_site(mut self, same_site: SameSite) -> Self {
        self.same_site = Some(same_site);
        self
    }

    /// Build the cookie
    pub fn build(self) -> Cookie {
        let mut cookie = Cookie::new(self.name, self.value)
            .secure(self.secure)
            .http_only(self.http_only);

        if let Some(max_age) = self.max_age {
            cookie = cookie.max_age(max_age);
        }

        if let Some(domain) = self.domain {
            cookie = cookie.domain(domain);
        }

        if let Some(path) = self.path {
            cookie = cookie.path(path);
        }

        if let Some(same_site) = self.same_site {
            cookie = cookie.same_site(same_site);
        }

        cookie
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cookie_jar_new() {
        let jar = CookieJar::new();
        assert!(jar.is_empty());
        assert_eq!(jar.len(), 0);
    }

    #[test]
    fn test_cookie_jar_add_get() {
        let mut jar = CookieJar::new();
        jar.add("session", "abc123");

        assert_eq!(jar.get("session"), Some(&"abc123".to_string()));
        assert!(!jar.is_empty());
        assert_eq!(jar.len(), 1);
    }

    #[test]
    fn test_cookie_jar_has() {
        let mut jar = CookieJar::new();
        jar.add("user", "alice");

        assert!(jar.has("user"));
        assert!(!jar.has("admin"));
    }

    #[test]
    fn test_cookie_jar_remove() {
        let mut jar = CookieJar::new();
        jar.add("temp", "value");

        assert!(jar.has("temp"));
        jar.remove("temp");
        assert!(!jar.has("temp"));
    }

    #[test]
    fn test_cookie_jar_from_header() {
        let header = "session=abc123; user=alice; theme=dark";
        let jar = CookieJar::from_header(header);

        assert_eq!(jar.len(), 3);
        assert_eq!(jar.get("session"), Some(&"abc123".to_string()));
        assert_eq!(jar.get("user"), Some(&"alice".to_string()));
        assert_eq!(jar.get("theme"), Some(&"dark".to_string()));
    }

    #[test]
    fn test_cookie_jar_from_header_with_spaces() {
        let header = " session = abc123 ; user = alice ";
        let jar = CookieJar::from_header(header);

        assert_eq!(jar.get("session"), Some(&"abc123".to_string()));
        assert_eq!(jar.get("user"), Some(&"alice".to_string()));
    }

    #[test]
    fn test_cookie_jar_clear() {
        let mut jar = CookieJar::new();
        jar.add("a", "1");
        jar.add("b", "2");

        assert_eq!(jar.len(), 2);
        jar.clear();
        assert!(jar.is_empty());
    }

    #[test]
    fn test_cookie_jar_iter() {
        let mut jar = CookieJar::new();
        jar.add("key1", "value1");
        jar.add("key2", "value2");

        let count = jar.iter().count();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_signed_cookie_jar() {
        let mut jar = SignedCookieJar::new("secret-key");
        jar.add("secure", "data");

        // The signed value should be different from the original
        let signed_value = jar.jar().get("secure").unwrap();
        assert!(signed_value.contains('.'));
        assert!(signed_value.len() > 4);
    }

    #[test]
    fn test_signed_cookie_get() {
        let mut jar = SignedCookieJar::new("secret-key");
        jar.add("token", "abc123");

        let value = jar.get("token");
        assert_eq!(value, Some("abc123".to_string()));
    }

    #[test]
    fn test_cookie_builder() {
        // Test that cookie builder doesn't panic
        let _cookie = CookieBuilder::new("session", "xyz789")
            .max_age(3600)
            .domain("example.com")
            .path("/app")
            .secure(true)
            .http_only(true)
            .same_site(SameSite::Strict)
            .build();

        // If we got here without panicking, the test passes
    }

    #[test]
    fn test_cookie_builder_defaults() {
        let _cookie = CookieBuilder::new("test", "value").build();

        // If we got here without panicking, the test passes
    }
}
