//! Helmet - Comprehensive security headers middleware
//!
//! Provides advanced security headers protection beyond Helmet.js,
//! with additional Rust-specific security features and optimizations.

use crate::{
    Request, Response,
    error::Result,
    middleware::{Middleware, Next},
};
use std::future::Future;
use std::pin::Pin;

/// Content Security Policy configuration
#[derive(Debug, Clone)]
pub struct CspConfig {
    pub default_src: Vec<String>,
    pub script_src: Vec<String>,
    pub style_src: Vec<String>,
    pub img_src: Vec<String>,
    pub font_src: Vec<String>,
    pub connect_src: Vec<String>,
    pub frame_src: Vec<String>,
    pub object_src: Vec<String>,
    pub media_src: Vec<String>,
    pub worker_src: Vec<String>,
    pub manifest_src: Vec<String>,
    pub base_uri: Vec<String>,
    pub form_action: Vec<String>,
    pub frame_ancestors: Vec<String>,
    pub upgrade_insecure_requests: bool,
    pub block_all_mixed_content: bool,
    pub require_trusted_types_for: Option<String>,
}

impl Default for CspConfig {
    fn default() -> Self {
        Self {
            default_src: vec!["'self'".to_string()],
            script_src: vec!["'self'".to_string()],
            style_src: vec!["'self'".to_string()],
            img_src: vec!["'self'".to_string(), "data:".to_string()],
            font_src: vec!["'self'".to_string()],
            connect_src: vec!["'self'".to_string()],
            frame_src: vec!["'none'".to_string()],
            object_src: vec!["'none'".to_string()],
            media_src: vec!["'self'".to_string()],
            worker_src: vec!["'self'".to_string()],
            manifest_src: vec!["'self'".to_string()],
            base_uri: vec!["'self'".to_string()],
            form_action: vec!["'self'".to_string()],
            frame_ancestors: vec!["'none'".to_string()],
            upgrade_insecure_requests: true,
            block_all_mixed_content: true,
            require_trusted_types_for: Some("'script'".to_string()),
        }
    }
}

impl CspConfig {
    /// Build CSP header value
    fn build(&self) -> String {
        let mut directives = Vec::new();

        let add_directive = |directives: &mut Vec<String>, name: &str, values: &[String]| {
            if !values.is_empty() {
                directives.push(format!("{} {}", name, values.join(" ")));
            }
        };

        add_directive(&mut directives, "default-src", &self.default_src);
        add_directive(&mut directives, "script-src", &self.script_src);
        add_directive(&mut directives, "style-src", &self.style_src);
        add_directive(&mut directives, "img-src", &self.img_src);
        add_directive(&mut directives, "font-src", &self.font_src);
        add_directive(&mut directives, "connect-src", &self.connect_src);
        add_directive(&mut directives, "frame-src", &self.frame_src);
        add_directive(&mut directives, "object-src", &self.object_src);
        add_directive(&mut directives, "media-src", &self.media_src);
        add_directive(&mut directives, "worker-src", &self.worker_src);
        add_directive(&mut directives, "manifest-src", &self.manifest_src);
        add_directive(&mut directives, "base-uri", &self.base_uri);
        add_directive(&mut directives, "form-action", &self.form_action);
        add_directive(&mut directives, "frame-ancestors", &self.frame_ancestors);

        if self.upgrade_insecure_requests {
            directives.push("upgrade-insecure-requests".to_string());
        }

        if self.block_all_mixed_content {
            directives.push("block-all-mixed-content".to_string());
        }

        if let Some(ref trusted_types) = self.require_trusted_types_for {
            directives.push(format!("require-trusted-types-for {}", trusted_types));
        }

        directives.join("; ")
    }
}

/// Permissions Policy configuration
#[derive(Debug, Clone)]
pub struct PermissionsPolicyConfig {
    pub geolocation: Vec<String>,
    pub camera: Vec<String>,
    pub microphone: Vec<String>,
    pub payment: Vec<String>,
    pub usb: Vec<String>,
    pub magnetometer: Vec<String>,
    pub gyroscope: Vec<String>,
    pub accelerometer: Vec<String>,
    pub ambient_light_sensor: Vec<String>,
    pub autoplay: Vec<String>,
    pub encrypted_media: Vec<String>,
    pub fullscreen: Vec<String>,
    pub picture_in_picture: Vec<String>,
}

impl Default for PermissionsPolicyConfig {
    fn default() -> Self {
        Self {
            geolocation: vec![],
            camera: vec![],
            microphone: vec![],
            payment: vec![],
            usb: vec![],
            magnetometer: vec![],
            gyroscope: vec![],
            accelerometer: vec![],
            ambient_light_sensor: vec![],
            autoplay: vec!["self".to_string()],
            encrypted_media: vec!["self".to_string()],
            fullscreen: vec!["self".to_string()],
            picture_in_picture: vec!["self".to_string()],
        }
    }
}

impl PermissionsPolicyConfig {
    /// Build Permissions-Policy header value
    fn build(&self) -> String {
        let mut policies = Vec::new();

        let add_policy = |policies: &mut Vec<String>, name: &str, values: &[String]| {
            let value = if values.is_empty() {
                format!("{}=()", name)
            } else {
                format!("{}=({})", name, values.join(" "))
            };
            policies.push(value);
        };

        add_policy(&mut policies, "geolocation", &self.geolocation);
        add_policy(&mut policies, "camera", &self.camera);
        add_policy(&mut policies, "microphone", &self.microphone);
        add_policy(&mut policies, "payment", &self.payment);
        add_policy(&mut policies, "usb", &self.usb);
        add_policy(&mut policies, "magnetometer", &self.magnetometer);
        add_policy(&mut policies, "gyroscope", &self.gyroscope);
        add_policy(&mut policies, "accelerometer", &self.accelerometer);
        add_policy(
            &mut policies,
            "ambient-light-sensor",
            &self.ambient_light_sensor,
        );
        add_policy(&mut policies, "autoplay", &self.autoplay);
        add_policy(&mut policies, "encrypted-media", &self.encrypted_media);
        add_policy(&mut policies, "fullscreen", &self.fullscreen);
        add_policy(
            &mut policies,
            "picture-in-picture",
            &self.picture_in_picture,
        );

        policies.join(", ")
    }
}

/// Helmet middleware configuration
///
/// Provides comprehensive security headers beyond Helmet.js with:
/// - Advanced Content Security Policy
/// - Strict Transport Security (HSTS)
/// - Permissions Policy
/// - Cross-Origin policies
/// - DNS Prefetch Control
/// - Certificate Transparency
/// - And more...
#[derive(Debug, Clone)]
pub struct Helmet {
    // CSP
    pub csp: Option<CspConfig>,
    pub csp_report_only: bool,

    // HSTS
    pub hsts_enabled: bool,
    pub hsts_max_age: u32,
    pub hsts_include_subdomains: bool,
    pub hsts_preload: bool,

    // X-Frame-Options
    pub x_frame_options: Option<String>,

    // X-Content-Type-Options
    pub x_content_type_options: bool,

    // Referrer-Policy
    pub referrer_policy: Option<String>,

    // Permissions-Policy
    pub permissions_policy: Option<PermissionsPolicyConfig>,

    // X-DNS-Prefetch-Control
    pub dns_prefetch_control: bool,

    // Expect-CT
    pub expect_ct_enabled: bool,
    pub expect_ct_max_age: u32,
    pub expect_ct_enforce: bool,
    pub expect_ct_report_uri: Option<String>,

    // Cross-Origin policies
    pub cross_origin_embedder_policy: Option<String>,
    pub cross_origin_opener_policy: Option<String>,
    pub cross_origin_resource_policy: Option<String>,

    // X-Permitted-Cross-Domain-Policies
    pub x_permitted_cross_domain_policies: Option<String>,

    // Origin-Agent-Cluster
    pub origin_agent_cluster: bool,

    // X-Download-Options (IE8+)
    pub x_download_options: bool,

    // Additional Rust-specific security features
    pub strict_sniffing_protection: bool,
    pub cache_control_sensitive: bool,
}

impl Default for Helmet {
    fn default() -> Self {
        Self::new()
    }
}

impl Helmet {
    /// Create a new Helmet middleware with secure defaults
    pub fn new() -> Self {
        Self {
            csp: Some(CspConfig::default()),
            csp_report_only: false,
            hsts_enabled: true,
            hsts_max_age: 31536000, // 1 year
            hsts_include_subdomains: true,
            hsts_preload: true,
            x_frame_options: Some("DENY".to_string()),
            x_content_type_options: true,
            referrer_policy: Some("strict-origin-when-cross-origin".to_string()),
            permissions_policy: Some(PermissionsPolicyConfig::default()),
            dns_prefetch_control: false,
            expect_ct_enabled: true,
            expect_ct_max_age: 86400,
            expect_ct_enforce: true,
            expect_ct_report_uri: None,
            cross_origin_embedder_policy: Some("require-corp".to_string()),
            cross_origin_opener_policy: Some("same-origin".to_string()),
            cross_origin_resource_policy: Some("same-origin".to_string()),
            x_permitted_cross_domain_policies: Some("none".to_string()),
            origin_agent_cluster: true,
            x_download_options: true,
            strict_sniffing_protection: true,
            cache_control_sensitive: true,
        }
    }

    /// Set Content Security Policy
    pub fn csp(mut self, csp: CspConfig) -> Self {
        self.csp = Some(csp);
        self
    }

    /// Disable CSP
    pub fn disable_csp(mut self) -> Self {
        self.csp = None;
        self
    }

    /// Enable CSP report-only mode
    pub fn csp_report_only(mut self, enabled: bool) -> Self {
        self.csp_report_only = enabled;
        self
    }

    /// Configure HSTS
    pub fn hsts(mut self, max_age: u32, include_subdomains: bool, preload: bool) -> Self {
        self.hsts_enabled = true;
        self.hsts_max_age = max_age;
        self.hsts_include_subdomains = include_subdomains;
        self.hsts_preload = preload;
        self
    }

    /// Disable HSTS
    pub fn disable_hsts(mut self) -> Self {
        self.hsts_enabled = false;
        self
    }

    /// Set X-Frame-Options
    pub fn frame_options(mut self, value: impl Into<String>) -> Self {
        self.x_frame_options = Some(value.into());
        self
    }

    /// Set Referrer-Policy
    pub fn referrer_policy(mut self, policy: impl Into<String>) -> Self {
        self.referrer_policy = Some(policy.into());
        self
    }

    /// Set Permissions-Policy
    pub fn permissions_policy(mut self, policy: PermissionsPolicyConfig) -> Self {
        self.permissions_policy = Some(policy);
        self
    }

    /// Apply security headers to response
    fn apply_headers(&self, mut response: Response) -> Response {
        // Content Security Policy
        if let Some(ref csp) = self.csp {
            let header_name = if self.csp_report_only {
                "content-security-policy-report-only"
            } else {
                "content-security-policy"
            };
            response = response.header_str(header_name, &csp.build());
        }

        // HSTS
        if self.hsts_enabled {
            let mut hsts = format!("max-age={}", self.hsts_max_age);
            if self.hsts_include_subdomains {
                hsts.push_str("; includeSubDomains");
            }
            if self.hsts_preload {
                hsts.push_str("; preload");
            }
            response = response.header_str("strict-transport-security", &hsts);
        }

        // X-Frame-Options
        if let Some(ref value) = self.x_frame_options {
            response = response.header_str("x-frame-options", value);
        }

        // X-Content-Type-Options
        if self.x_content_type_options {
            response = response.header_str("x-content-type-options", "nosniff");
        }

        // Referrer-Policy
        if let Some(ref policy) = self.referrer_policy {
            response = response.header_str("referrer-policy", policy);
        }

        // Permissions-Policy
        if let Some(ref policy) = self.permissions_policy {
            response = response.header_str("permissions-policy", &policy.build());
        }

        // DNS Prefetch Control
        response = response.header_str(
            "x-dns-prefetch-control",
            if self.dns_prefetch_control {
                "on"
            } else {
                "off"
            },
        );

        // Expect-CT
        if self.expect_ct_enabled {
            let mut expect_ct = format!("max-age={}", self.expect_ct_max_age);
            if self.expect_ct_enforce {
                expect_ct.push_str(", enforce");
            }
            if let Some(ref report_uri) = self.expect_ct_report_uri {
                expect_ct.push_str(&format!(", report-uri=\"{}\"", report_uri));
            }
            response = response.header_str("expect-ct", &expect_ct);
        }

        // Cross-Origin policies
        if let Some(ref policy) = self.cross_origin_embedder_policy {
            response = response.header_str("cross-origin-embedder-policy", policy);
        }
        if let Some(ref policy) = self.cross_origin_opener_policy {
            response = response.header_str("cross-origin-opener-policy", policy);
        }
        if let Some(ref policy) = self.cross_origin_resource_policy {
            response = response.header_str("cross-origin-resource-policy", policy);
        }

        // X-Permitted-Cross-Domain-Policies
        if let Some(ref policy) = self.x_permitted_cross_domain_policies {
            response = response.header_str("x-permitted-cross-domain-policies", policy);
        }

        // Origin-Agent-Cluster
        if self.origin_agent_cluster {
            response = response.header_str("origin-agent-cluster", "?1");
        }

        // X-Download-Options
        if self.x_download_options {
            response = response.header_str("x-download-options", "noopen");
        }

        // Strict sniffing protection (Rust-specific enhancement)
        if self.strict_sniffing_protection {
            response = response.header_str("x-content-type-options", "nosniff");
            response = response.header_str("x-permitted-cross-domain-policies", "none");
        }

        // Cache control for sensitive data
        if self.cache_control_sensitive {
            response = response.header_str(
                "cache-control",
                "no-store, no-cache, must-revalidate, private",
            );
            response = response.header_str("pragma", "no-cache");
        }

        response
    }
}

impl Middleware for Helmet {
    fn handle(
        &self,
        req: Request,
        next: Next,
    ) -> Pin<Box<dyn Future<Output = Result<Response>> + Send + '_>> {
        Box::pin(async move {
            let response = next(req).await?;
            Ok(self.apply_headers(response))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csp_build() {
        let csp = CspConfig::default();
        let header = csp.build();

        assert!(header.contains("default-src 'self'"));
        assert!(header.contains("script-src 'self'"));
        assert!(header.contains("upgrade-insecure-requests"));
    }

    #[test]
    fn test_permissions_policy_build() {
        let policy = PermissionsPolicyConfig::default();
        let header = policy.build();

        assert!(header.contains("geolocation=()"));
        assert!(header.contains("camera=()"));
        assert!(header.contains("microphone=()"));
    }

    #[test]
    fn test_helmet_defaults() {
        let helmet = Helmet::new();

        assert!(helmet.csp.is_some());
        assert!(helmet.hsts_enabled);
        assert_eq!(helmet.hsts_max_age, 31536000);
        assert!(helmet.x_content_type_options);
    }
}
