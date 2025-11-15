use crate::{Error, Response, Result};
use handlebars::Handlebars;
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Template engine configuration
#[derive(Debug, Clone)]
pub struct TemplateConfig {
    /// Template directory
    pub dir: PathBuf,
    /// Template file extension (default: ".hbs")
    pub extension: String,
    /// Enable template caching
    pub cache: bool,
    /// Enable strict mode (error on missing variables)
    pub strict: bool,
}

impl Default for TemplateConfig {
    fn default() -> Self {
        Self {
            dir: PathBuf::from("./templates"),
            extension: ".hbs".to_string(),
            cache: true,
            strict: false,
        }
    }
}

impl TemplateConfig {
    /// Create new template configuration
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self {
            dir: dir.into(),
            ..Default::default()
        }
    }

    /// Set template extension
    pub fn extension(mut self, ext: impl Into<String>) -> Self {
        self.extension = ext.into();
        self
    }

    /// Enable/disable caching
    pub fn cache(mut self, enable: bool) -> Self {
        self.cache = enable;
        self
    }

    /// Enable/disable strict mode
    pub fn strict(mut self, enable: bool) -> Self {
        self.strict = enable;
        self
    }
}

/// Template engine
pub struct TemplateEngine {
    config: TemplateConfig,
    handlebars: Arc<RwLock<Handlebars<'static>>>,
}

impl TemplateEngine {
    /// Create new template engine
    pub fn new(config: TemplateConfig) -> Self {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(config.strict);

        Self {
            config,
            handlebars: Arc::new(RwLock::new(handlebars)),
        }
    }

    /// Register a template from string
    pub async fn register_template(
        &self,
        name: impl Into<String>,
        source: impl Into<String>,
    ) -> Result<()> {
        let name = name.into();
        let source = source.into();

        let mut hbs = self.handlebars.write().await;
        hbs.register_template_string(&name, source)
            .map_err(|e| Error::InternalError(format!("Failed to register template: {}", e)))?;

        Ok(())
    }

    /// Register a template from file
    pub async fn register_template_file(
        &self,
        name: impl Into<String>,
        path: impl AsRef<Path>,
    ) -> Result<()> {
        let name = name.into();
        let path = path.as_ref();

        let template_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.config.dir.join(path)
        };

        let mut hbs = self.handlebars.write().await;
        hbs.register_template_file(&name, template_path)
            .map_err(|e| Error::InternalError(format!("Failed to register template file: {}", e)))?;

        Ok(())
    }

    /// Register a helper function
    pub async fn register_helper<F>(&self, name: &str, helper: F)
    where
        F: handlebars::HelperDef + Send + Sync + 'static,
    {
        let mut hbs = self.handlebars.write().await;
        hbs.register_helper(name, Box::new(helper));
    }

    /// Render a template with context data
    pub async fn render<T: Serialize>(
        &self,
        name: &str,
        context: &T,
    ) -> Result<String> {
        let hbs = self.handlebars.read().await;

        hbs.render(name, context)
            .map_err(|e| {
                Error::InternalError(format!("Template render error: {}", e))
            })
    }

    /// Render a template and return HTTP response
    pub async fn render_response<T: Serialize>(
        &self,
        name: &str,
        context: &T,
    ) -> Result<Response> {
        let html = self.render(name, context).await?;
        Ok(Response::html(&html))
    }

    /// Clear all registered templates
    pub async fn clear(&self) {
        let mut hbs = self.handlebars.write().await;
        hbs.clear_templates();
    }

    /// Get configuration
    pub fn config(&self) -> &TemplateConfig {
        &self.config
    }
}

/// Template context builder
#[derive(Debug, Clone, Default)]
pub struct TemplateContext {
    data: HashMap<String, serde_json::Value>,
}

impl TemplateContext {
    /// Create new template context
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a value into the context
    pub fn insert<T: Serialize>(mut self, key: impl Into<String>, value: &T) -> Self {
        if let Ok(json_value) = serde_json::to_value(value) {
            self.data.insert(key.into(), json_value);
        }
        self
    }

    /// Get the context as a HashMap
    pub fn build(self) -> HashMap<String, serde_json::Value> {
        self.data
    }
}

impl Serialize for TemplateContext {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.data.serialize(serializer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_template_config_default() {
        let config = TemplateConfig::default();
        assert_eq!(config.dir, PathBuf::from("./templates"));
        assert_eq!(config.extension, ".hbs");
        assert!(config.cache);
        assert!(!config.strict);
    }

    #[test]
    fn test_template_config_builder() {
        let config = TemplateConfig::new("/var/templates")
            .extension(".html")
            .cache(false)
            .strict(true);

        assert_eq!(config.dir, PathBuf::from("/var/templates"));
        assert_eq!(config.extension, ".html");
        assert!(!config.cache);
        assert!(config.strict);
    }

    #[tokio::test]
    async fn test_template_engine_creation() {
        let config = TemplateConfig::new("./templates");
        let engine = TemplateEngine::new(config);
        assert_eq!(engine.config().dir, PathBuf::from("./templates"));
    }

    #[tokio::test]
    async fn test_register_template() {
        let config = TemplateConfig::new("./templates");
        let engine = TemplateEngine::new(config);

        let result = engine
            .register_template("test", "Hello {{name}}!")
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_render_template() {
        let config = TemplateConfig::new("./templates");
        let engine = TemplateEngine::new(config);

        engine
            .register_template("greeting", "Hello {{name}}!")
            .await
            .unwrap();

        let context = json!({ "name": "World" });
        let result = engine.render("greeting", &context).await.unwrap();

        assert_eq!(result, "Hello World!");
    }

    #[tokio::test]
    async fn test_template_not_found() {
        let config = TemplateConfig::new("./templates");
        let engine = TemplateEngine::new(config);

        let context = json!({});
        let result = engine.render("nonexistent", &context).await;

        assert!(result.is_err());
    }

    #[test]
    fn test_template_context_builder() {
        let context = TemplateContext::new()
            .insert("title", &"My Page")
            .insert("count", &42)
            .insert("items", &vec!["a", "b", "c"]);

        let data = context.build();
        assert_eq!(data.len(), 3);
        assert!(data.contains_key("title"));
        assert!(data.contains_key("count"));
        assert!(data.contains_key("items"));
    }

    #[tokio::test]
    async fn test_render_response() {
        let config = TemplateConfig::new("./templates");
        let engine = TemplateEngine::new(config);

        engine
            .register_template("page", "<h1>{{title}}</h1>")
            .await
            .unwrap();

        let context = json!({ "title": "Welcome" });
        let response = engine.render_response("page", &context).await.unwrap();

        assert_eq!(response.get_status().as_u16(), 200);
    }

    #[tokio::test]
    async fn test_clear_templates() {
        let config = TemplateConfig::new("./templates");
        let engine = TemplateEngine::new(config);

        engine
            .register_template("test", "Hello!")
            .await
            .unwrap();

        engine.clear().await;

        let context = json!({});
        let result = engine.render("test", &context).await;
        assert!(result.is_err());
    }
}
