//! OpenAttribution `.well-known/openattribution.json` check.
//!
//! Parses the JSON file that content owners publish to declare their
//! telemetry endpoint and verification token.

use serde::{Deserialize, Serialize};

/// Parsed content of `/.well-known/openattribution.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WellKnownOaResult {
    pub found: bool,
    pub version: Option<String>,
    pub telemetry_endpoint: Option<String>,
    pub has_verification: bool,
}

/// Raw JSON structure matching the spec.
#[derive(Debug, Deserialize)]
struct OaFile {
    openattribution: Option<OaBlock>,
}

#[derive(Debug, Deserialize)]
struct OaBlock {
    version: Option<String>,
    telemetry_endpoint: Option<String>,
    verification: Option<String>,
}

/// Evaluate raw JSON content from `/.well-known/openattribution.json`.
///
/// Returns `found = false` for missing or malformed files.
pub fn evaluate(content: &str) -> WellKnownOaResult {
    let parsed: Result<OaFile, _> = serde_json::from_str(content);

    match parsed {
        Ok(file) => match file.openattribution {
            Some(block) => WellKnownOaResult {
                found: true,
                version: block.version,
                telemetry_endpoint: block.telemetry_endpoint,
                has_verification: block.verification.is_some(),
            },
            None => WellKnownOaResult {
                found: false,
                version: None,
                telemetry_endpoint: None,
                has_verification: false,
            },
        },
        Err(_) => WellKnownOaResult {
            found: false,
            version: None,
            telemetry_endpoint: None,
            has_verification: false,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_json_with_all_fields() {
        let content = r#"{
            "openattribution": {
                "version": "0.4",
                "telemetry_endpoint": "https://api.openattribution.org/api/v1/telemetry",
                "verification": "oa-verify=7ddfbe00-1d6c-489e-bf82-c44ad2f27e2a"
            }
        }"#;

        let result = evaluate(content);
        assert!(result.found);
        assert_eq!(result.version, Some("0.4".to_string()));
        assert_eq!(
            result.telemetry_endpoint,
            Some("https://api.openattribution.org/api/v1/telemetry".to_string())
        );
        assert!(result.has_verification);
    }

    #[test]
    fn test_valid_json_without_verification() {
        let content = r#"{
            "openattribution": {
                "version": "0.4",
                "telemetry_endpoint": "https://telemetry.example.com"
            }
        }"#;

        let result = evaluate(content);
        assert!(result.found);
        assert_eq!(result.version, Some("0.4".to_string()));
        assert!(!result.has_verification);
    }

    #[test]
    fn test_valid_json_minimal() {
        let content = r#"{ "openattribution": {} }"#;

        let result = evaluate(content);
        assert!(result.found);
        assert_eq!(result.version, None);
        assert_eq!(result.telemetry_endpoint, None);
        assert!(!result.has_verification);
    }

    #[test]
    fn test_missing_openattribution_key() {
        let content = r#"{ "other": "data" }"#;

        let result = evaluate(content);
        assert!(!result.found);
    }

    #[test]
    fn test_malformed_json() {
        let result = evaluate("not json at all");
        assert!(!result.found);
    }

    #[test]
    fn test_empty_string() {
        let result = evaluate("");
        assert!(!result.found);
    }

    #[test]
    fn test_html_404_page() {
        let result = evaluate("<html><body>404 Not Found</body></html>");
        assert!(!result.found);
    }
}
