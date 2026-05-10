//! OpenAttribution `.well-known/openattribution.json` check.
//!
//! Parses the manifest published by content owners, agents, and platforms to
//! declare their identity and telemetry endpoint. Section 8 of the
//! OpenAttribution Telemetry specification is the normative reference.
//!
//! Supports the v0.1 manifest shape (current) and the legacy wrapper shape
//! (`{ "openattribution": { ... } }`) that predates the unified spec. The
//! legacy fallback exists only so previously published files continue to be
//! recognised; new manifests should use v0.1.

use serde::{Deserialize, Serialize};

/// Parsed content of `/.well-known/openattribution.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WellKnownOaResult {
    pub found: bool,
    /// v0.1 manifest field. `None` for legacy manifests.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_version: Option<String>,
    /// Roles declared by the manifest. v0.1 only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<String>>,
    /// Operator display name (`operator.name`). v0.1 only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator_name: Option<String>,
    /// Telemetry endpoint URL. Reads `telemetry.endpoint` (v0.1) or
    /// `telemetry_endpoint` (legacy).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub telemetry_endpoint: Option<String>,
    /// Legacy wrapper-shape `version` field. `None` for v0.1 manifests.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Legacy wrapper-shape only - whether a `verification` field was present.
    /// Always `false` for v0.1 manifests.
    pub has_verification: bool,
}

/// v0.1 manifest top level.
#[derive(Debug, Deserialize)]
struct V01Manifest {
    schema_version: String,
    #[serde(default)]
    roles: Option<Vec<String>>,
    #[serde(default)]
    operator: Option<V01Operator>,
    #[serde(default)]
    telemetry: Option<V01Telemetry>,
}

#[derive(Debug, Deserialize)]
struct V01Operator {
    #[serde(default)]
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct V01Telemetry {
    #[serde(default)]
    endpoint: Option<String>,
}

/// Legacy wrapper shape: `{ "openattribution": { ... } }`.
#[derive(Debug, Deserialize)]
struct LegacyFile {
    openattribution: Option<LegacyBlock>,
}

#[derive(Debug, Deserialize)]
struct LegacyBlock {
    version: Option<String>,
    telemetry_endpoint: Option<String>,
    verification: Option<String>,
}

fn empty() -> WellKnownOaResult {
    WellKnownOaResult {
        found: false,
        schema_version: None,
        roles: None,
        operator_name: None,
        telemetry_endpoint: None,
        version: None,
        has_verification: false,
    }
}

/// Evaluate raw JSON content from `/.well-known/openattribution.json`.
///
/// Returns `found = false` for missing or malformed files. v0.1 manifests are
/// preferred; legacy `{ "openattribution": { ... } }` shape is also accepted.
pub fn evaluate(content: &str) -> WellKnownOaResult {
    if content.is_empty() {
        return empty();
    }

    // v0.1 manifest: discriminated by a top-level `schema_version` string.
    if let Ok(manifest) = serde_json::from_str::<V01Manifest>(content) {
        return WellKnownOaResult {
            found: true,
            schema_version: Some(manifest.schema_version),
            roles: manifest.roles,
            operator_name: manifest.operator.and_then(|o| o.name),
            telemetry_endpoint: manifest.telemetry.and_then(|t| t.endpoint),
            version: None,
            has_verification: false,
        };
    }

    // Legacy fallback: wrapper object with `openattribution` key.
    if let Ok(file) = serde_json::from_str::<LegacyFile>(content) {
        if let Some(block) = file.openattribution {
            return WellKnownOaResult {
                found: true,
                schema_version: None,
                roles: None,
                operator_name: None,
                telemetry_endpoint: block.telemetry_endpoint,
                version: block.version,
                has_verification: block.verification.is_some(),
            };
        }
    }

    empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_v01_manifest_full() {
        let content = r#"{
            "schema_version": "0.1",
            "id": "https://example.com/.well-known/openattribution.json",
            "roles": ["content_owner"],
            "operator": { "name": "Example Media" },
            "telemetry": {
                "endpoint": "https://api.openattribution.org/v1/events",
                "conformance_level": "retrieval"
            },
            "domains": ["example.com", "*.example.com"]
        }"#;

        let result = evaluate(content);
        assert!(result.found);
        assert_eq!(result.schema_version, Some("0.1".to_string()));
        assert_eq!(result.roles, Some(vec!["content_owner".to_string()]));
        assert_eq!(result.operator_name, Some("Example Media".to_string()));
        assert_eq!(
            result.telemetry_endpoint,
            Some("https://api.openattribution.org/v1/events".to_string())
        );
        assert_eq!(result.version, None);
        assert!(!result.has_verification);
    }

    #[test]
    fn test_v01_manifest_minimal() {
        let content = r#"{
            "schema_version": "0.1",
            "id": "https://example.com/.well-known/openattribution.json",
            "roles": ["agent"],
            "operator": { "name": "Example" }
        }"#;

        let result = evaluate(content);
        assert!(result.found);
        assert_eq!(result.schema_version, Some("0.1".to_string()));
        assert_eq!(result.roles, Some(vec!["agent".to_string()]));
        assert_eq!(result.telemetry_endpoint, None);
        assert!(!result.has_verification);
    }

    #[test]
    fn test_v01_agent_with_keys_and_telemetry() {
        let content = r#"{
            "schema_version": "0.1",
            "id": "https://searchco.com/agents/web-search/.well-known/openattribution.json",
            "roles": ["agent"],
            "operator": { "name": "SearchCo" },
            "keys": [
                { "id": "key-1", "type": "Ed25519", "publicKey": "z6Mk..." }
            ],
            "telemetry": {
                "endpoint": "https://api.openattribution.org/v1/events",
                "conformance_level": "grounding"
            }
        }"#;

        let result = evaluate(content);
        assert!(result.found);
        assert_eq!(result.roles, Some(vec!["agent".to_string()]));
        assert_eq!(
            result.telemetry_endpoint,
            Some("https://api.openattribution.org/v1/events".to_string())
        );
    }

    #[test]
    fn test_legacy_with_all_fields() {
        let content = r#"{
            "openattribution": {
                "version": "0.1",
                "telemetry_endpoint": "https://api.openattribution.org/api/v1/telemetry",
                "verification": "oa-verify=7ddfbe00-1d6c-489e-bf82-c44ad2f27e2a"
            }
        }"#;

        let result = evaluate(content);
        assert!(result.found);
        assert_eq!(result.schema_version, None);
        assert_eq!(result.version, Some("0.1".to_string()));
        assert_eq!(
            result.telemetry_endpoint,
            Some("https://api.openattribution.org/api/v1/telemetry".to_string())
        );
        assert!(result.has_verification);
    }

    #[test]
    fn test_legacy_without_verification() {
        let content = r#"{
            "openattribution": {
                "version": "0.1",
                "telemetry_endpoint": "https://telemetry.example.com"
            }
        }"#;

        let result = evaluate(content);
        assert!(result.found);
        assert_eq!(result.version, Some("0.1".to_string()));
        assert!(!result.has_verification);
    }

    #[test]
    fn test_legacy_minimal() {
        let content = r#"{ "openattribution": {} }"#;

        let result = evaluate(content);
        assert!(result.found);
        assert_eq!(result.schema_version, None);
        assert_eq!(result.version, None);
        assert_eq!(result.telemetry_endpoint, None);
        assert!(!result.has_verification);
    }

    #[test]
    fn test_unrelated_json() {
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
