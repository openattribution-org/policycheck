//! CDN and CMS infrastructure detection from HTTP response headers.
//!
//! Detects the CDN and CMS running behind a domain by inspecting
//! standard and vendor-specific HTTP response headers. Used to personalise
//! onboarding — e.g. recommending the WordPress plugin vs a Cloudflare Worker.

use serde::{Deserialize, Serialize};

/// Raw HTTP headers extracted by the CLI fetcher for infrastructure detection.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InfraProbeData {
    pub server: Option<String>,
    pub x_powered_by: Option<String>,
    /// Cloudflare: present on every response
    pub cf_ray: Option<String>,
    /// Vercel: present on every response
    pub x_vercel_id: Option<String>,
    /// Netlify: present on every response
    pub x_nf_request_id: Option<String>,
    /// Fastly: present on cache hits
    pub x_served_by: Option<String>,
    /// Akamai: present on transformed responses
    pub x_akamai_transformed: Option<String>,
    /// CloudFront: present on every response
    pub x_amz_cf_id: Option<String>,
    /// WordPress: Link header containing wp-json API
    pub link: Option<String>,
    /// WordPress: x-pingback header pointing at xmlrpc.php
    pub x_pingback: Option<String>,
}

/// Detected infrastructure for a domain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfraDetection {
    /// Detected CDN vendor, if any.
    pub cdn: Option<String>,
    /// Detected CMS, if any.
    pub cms: Option<String>,
}

/// Evaluate infrastructure from pre-fetched HTTP headers.
pub fn evaluate(probe: &InfraProbeData) -> InfraDetection {
    InfraDetection {
        cdn: detect_cdn(probe),
        cms: detect_cms(probe),
    }
}

fn detect_cdn(probe: &InfraProbeData) -> Option<String> {
    // Cloudflare: cf-ray header is definitive
    if probe.cf_ray.is_some() {
        return Some("cloudflare".to_string());
    }

    // Vercel
    if probe.x_vercel_id.is_some() {
        return Some("vercel".to_string());
    }

    // Netlify
    if probe.x_nf_request_id.is_some() {
        return Some("netlify".to_string());
    }

    // CloudFront
    if probe.x_amz_cf_id.is_some() {
        return Some("cloudfront".to_string());
    }

    // Fastly: x-served-by contains cache node names
    if let Some(served_by) = &probe.x_served_by {
        if served_by.contains("cache-") {
            return Some("fastly".to_string());
        }
    }

    // Akamai
    if probe.x_akamai_transformed.is_some() {
        return Some("akamai".to_string());
    }

    // Server header fallback
    if let Some(server) = &probe.server {
        let s = server.to_lowercase();
        if s.contains("cloudflare") {
            return Some("cloudflare".to_string());
        }
        if s.contains("cloudfront") {
            return Some("cloudfront".to_string());
        }
        if s.contains("netlify") {
            return Some("netlify".to_string());
        }
        if s.starts_with("vercel") {
            return Some("vercel".to_string());
        }
        if s.contains("akamaighost") {
            return Some("akamai".to_string());
        }
    }

    None
}

fn detect_cms(probe: &InfraProbeData) -> Option<String> {
    // WordPress: Link header with wp-json API is definitive
    if let Some(link) = &probe.link {
        if link.contains("wp-json") || link.contains("api.w.org") {
            return Some("wordpress".to_string());
        }
    }

    // WordPress: x-pingback pointing at xmlrpc.php
    if let Some(pingback) = &probe.x_pingback {
        if pingback.contains("xmlrpc.php") {
            return Some("wordpress".to_string());
        }
    }

    // Shopify
    if let Some(server) = &probe.server {
        if server.to_lowercase().contains("shopify") {
            return Some("shopify".to_string());
        }
    }
    if let Some(powered) = &probe.x_powered_by {
        let p = powered.to_lowercase();
        if p.contains("shopify") {
            return Some("shopify".to_string());
        }
    }

    // Squarespace
    if let Some(server) = &probe.server {
        if server.to_lowercase().contains("squarespace") {
            return Some("squarespace".to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cloudflare_detected_from_cf_ray() {
        let probe = InfraProbeData {
            cf_ray: Some("8f1a2b3c4d5e6f-LHR".to_string()),
            ..Default::default()
        };

        let result = evaluate(&probe);
        assert_eq!(result.cdn, Some("cloudflare".to_string()));
    }

    #[test]
    fn test_vercel_detected_from_x_vercel_id() {
        let probe = InfraProbeData {
            x_vercel_id: Some("iad1::iad1::abc123".to_string()),
            ..Default::default()
        };

        let result = evaluate(&probe);
        assert_eq!(result.cdn, Some("vercel".to_string()));
    }

    #[test]
    fn test_netlify_detected_from_x_nf_request_id() {
        let probe = InfraProbeData {
            x_nf_request_id: Some("01234567-89ab-cdef".to_string()),
            ..Default::default()
        };

        let result = evaluate(&probe);
        assert_eq!(result.cdn, Some("netlify".to_string()));
    }

    #[test]
    fn test_fastly_detected_from_x_served_by() {
        let probe = InfraProbeData {
            x_served_by: Some("cache-lhr7324-LHR".to_string()),
            ..Default::default()
        };

        let result = evaluate(&probe);
        assert_eq!(result.cdn, Some("fastly".to_string()));
    }

    #[test]
    fn test_cloudfront_detected_from_x_amz_cf_id() {
        let probe = InfraProbeData {
            x_amz_cf_id: Some("abc123==".to_string()),
            ..Default::default()
        };

        let result = evaluate(&probe);
        assert_eq!(result.cdn, Some("cloudfront".to_string()));
    }

    #[test]
    fn test_akamai_detected_from_header() {
        let probe = InfraProbeData {
            x_akamai_transformed: Some("9 - 0".to_string()),
            ..Default::default()
        };

        let result = evaluate(&probe);
        assert_eq!(result.cdn, Some("akamai".to_string()));
    }

    #[test]
    fn test_cloudflare_from_server_header_fallback() {
        let probe = InfraProbeData {
            server: Some("cloudflare".to_string()),
            ..Default::default()
        };

        let result = evaluate(&probe);
        assert_eq!(result.cdn, Some("cloudflare".to_string()));
    }

    #[test]
    fn test_wordpress_detected_from_link_header() {
        let probe = InfraProbeData {
            link: Some(r#"<https://example.com/wp-json/>; rel="https://api.w.org/""#.to_string()),
            ..Default::default()
        };

        let result = evaluate(&probe);
        assert_eq!(result.cms, Some("wordpress".to_string()));
    }

    #[test]
    fn test_wordpress_detected_from_x_pingback() {
        let probe = InfraProbeData {
            x_pingback: Some("https://example.com/xmlrpc.php".to_string()),
            ..Default::default()
        };

        let result = evaluate(&probe);
        assert_eq!(result.cms, Some("wordpress".to_string()));
    }

    #[test]
    fn test_shopify_detected_from_server() {
        let probe = InfraProbeData {
            server: Some("Shopify".to_string()),
            ..Default::default()
        };

        let result = evaluate(&probe);
        assert_eq!(result.cms, Some("shopify".to_string()));
    }

    #[test]
    fn test_wordpress_behind_cloudflare() {
        let probe = InfraProbeData {
            cf_ray: Some("8f1a2b3c4d5e6f-LHR".to_string()),
            link: Some(r#"<https://example.com/wp-json/>; rel="https://api.w.org/""#.to_string()),
            ..Default::default()
        };

        let result = evaluate(&probe);
        assert_eq!(result.cdn, Some("cloudflare".to_string()));
        assert_eq!(result.cms, Some("wordpress".to_string()));
    }

    #[test]
    fn test_no_infrastructure_detected() {
        let probe = InfraProbeData::default();

        let result = evaluate(&probe);
        assert_eq!(result.cdn, None);
        assert_eq!(result.cms, None);
    }

    #[test]
    fn test_squarespace_detected_from_server() {
        let probe = InfraProbeData {
            server: Some("Squarespace".to_string()),
            ..Default::default()
        };

        let result = evaluate(&probe);
        assert_eq!(result.cms, Some("squarespace".to_string()));
    }
}
