//! TDM (Text & Data Mining) policy evaluation.
//!
//! Evaluates rules from `/.well-known/tdmrep.json` against a URL path.
//! Supports `*` wildcards and `$` end-of-pattern markers per W3C TDMRep.
//!
//! See: <https://www.w3.org/community/reports/tdmrep/CG-FINAL-tdmrep-20240510/>

use crate::models::{TdmPolicy, TdmRule};
use url::Url;

/// Evaluate TDM rules against a URL and return the matching policy.
///
/// Returns `None` if the URL cannot be parsed.
/// Rules are evaluated in order (first-match wins per W3C TDMRep).
pub fn evaluate(url: &str, rules: Vec<TdmRule>) -> Option<TdmPolicy> {
    let parsed = Url::parse(url).ok()?;
    let path = parsed.path();

    // Find the first matching rule (first-match wins per W3C TDMRep)
    let matched_rule = rules
        .iter()
        .find(|rule| match_pattern(&rule.location, path))
        .cloned();

    let is_reserved = matched_rule
        .as_ref()
        .map(|r| r.tdm_reservation == 1)
        .unwrap_or(false);

    Some(TdmPolicy {
        rules,
        matched_rule,
        is_reserved,
    })
}

/// Match a path against a TDM location pattern.
///
/// Supports `*` wildcard and `$` end-of-pattern marker.
pub fn match_pattern(pattern: &str, path: &str) -> bool {
    // Remove $ end marker if present for processing
    let (pattern, must_end) = if let Some(stripped) = pattern.strip_suffix('$') {
        (stripped, true)
    } else {
        (pattern, false)
    };

    // Simple wildcard matching
    let parts: Vec<&str> = pattern.split('*').collect();

    if parts.len() == 1 {
        // No wildcards - exact match (or prefix if no $)
        if must_end {
            return path == pattern;
        } else {
            return path.starts_with(pattern);
        }
    }

    // Check if path matches the pattern with wildcards
    let mut path_pos = 0;
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }

        if i == 0 {
            // First part must match from the start
            if !path[path_pos..].starts_with(part) {
                return false;
            }
            path_pos += part.len();
        } else {
            // Find the next occurrence
            if let Some(pos) = path[path_pos..].find(part) {
                path_pos += pos + part.len();
            } else {
                return false;
            }
        }
    }

    // If must_end is true, ensure we've consumed the entire path
    !must_end || path_pos == path.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_exact_match() {
        assert!(match_pattern("/", "/"));
        assert!(match_pattern("/docs", "/docs"));
        assert!(match_pattern("/docs", "/docs/page"));
        assert!(!match_pattern("/docs$", "/docs/page"));
    }

    #[test]
    fn test_pattern_wildcard() {
        assert!(match_pattern("/docs/*", "/docs/page"));
        assert!(match_pattern("/docs/*", "/docs/page/sub"));
        assert!(match_pattern("*.pdf", "/file.pdf"));
        assert!(match_pattern("*.pdf", "/docs/file.pdf"));
        assert!(!match_pattern("/docs/*", "/other/page"));
    }

    #[test]
    fn test_pattern_end_marker() {
        assert!(match_pattern("/docs$", "/docs"));
        assert!(!match_pattern("/docs$", "/docs/"));
        assert!(!match_pattern("/docs$", "/docs/page"));
        assert!(match_pattern("/docs/page$", "/docs/page"));
    }

    #[test]
    fn test_pattern_complex() {
        assert!(match_pattern("/*/public/*", "/docs/public/file"));
        assert!(match_pattern("/docs/*.pdf$", "/docs/file.pdf"));
        assert!(!match_pattern("/docs/*.pdf$", "/docs/file.pdf.bak"));
    }

    #[test]
    fn test_evaluate_rule_matching() {
        let rules = vec![
            TdmRule {
                location: "/".to_string(),
                tdm_reservation: 1,
                tdm_policy: Some("https://example.com/policy.html".to_string()),
            },
            TdmRule {
                location: "/public/*".to_string(),
                tdm_reservation: 0,
                tdm_policy: None,
            },
        ];

        // Test root path matches first rule
        let policy = evaluate("https://example.com/", rules.clone());
        assert!(policy.is_some());
        let policy = policy.unwrap();
        assert!(policy.is_reserved);
        assert_eq!(policy.matched_rule.as_ref().unwrap().location, "/");

        // Test public path — first rule still wins (first-match)
        let rules2 = vec![
            TdmRule {
                location: "/public/*".to_string(),
                tdm_reservation: 0,
                tdm_policy: None,
            },
            TdmRule {
                location: "/".to_string(),
                tdm_reservation: 1,
                tdm_policy: Some("https://example.com/policy.html".to_string()),
            },
        ];

        let policy2 = evaluate("https://example.com/public/data", rules2);
        assert!(policy2.is_some());
        let policy2 = policy2.unwrap();
        assert!(!policy2.is_reserved);
        assert_eq!(policy2.matched_rule.as_ref().unwrap().location, "/public/*");
    }

    #[test]
    fn test_evaluate_empty_rules_returns_unreserved_with_no_match() {
        // GIVEN a valid URL but no TDM rules
        let policy = evaluate("https://www.nytimes.com/article", vec![]);

        // SHOULD return a policy with no matched rule and not reserved
        assert!(policy.is_some());
        let policy = policy.unwrap();
        assert!(!policy.is_reserved);
        assert!(policy.matched_rule.is_none());
    }

    #[test]
    fn test_evaluate_invalid_url_returns_none() {
        let policy = evaluate("not-a-url", vec![]);
        assert!(policy.is_none());
    }
}
