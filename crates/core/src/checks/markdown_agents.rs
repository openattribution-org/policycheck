//! Cloudflare "Markdown for Agents" detection.
//!
//! Detects whether a site supports Cloudflare's Markdown for Agents feature.
//! When enabled, sites return `Content-Type: text/markdown` and
//! `x-markdown-tokens` headers in response to `Accept: text/markdown` requests.
//! May also include a `Content-Signal` HTTP header with licence signals.
//!
//! See: <https://blog.cloudflare.com/markdown-for-ai-agents>

use serde::{Deserialize, Serialize};

/// Pre-fetched probe data from the CLI layer (no I/O in core).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownProbeData {
    pub status_code: u16,
    pub content_type: Option<String>,
    pub markdown_tokens: Option<String>,
    pub content_signal: Option<String>,
}

/// Result of Markdown for Agents evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownAgentsResult {
    pub supported: bool,
    pub token_count: Option<u64>,
    pub http_content_signal_search: Option<String>,
    pub http_content_signal_ai_input: Option<String>,
    pub http_content_signal_ai_train: Option<String>,
}

/// Evaluate Markdown for Agents support from pre-fetched probe data.
///
/// - `supported` is true if the response `Content-Type` starts with `text/markdown`.
/// - `token_count` is parsed from the `x-markdown-tokens` header value.
/// - `Content-Signal` header is parsed as comma-separated `key=value` pairs.
pub fn evaluate(probe: &MarkdownProbeData) -> MarkdownAgentsResult {
    let supported = probe
        .content_type
        .as_deref()
        .map(|ct| ct.starts_with("text/markdown"))
        .unwrap_or(false);

    let token_count = probe
        .markdown_tokens
        .as_deref()
        .and_then(|v| v.trim().parse::<u64>().ok());

    let (search, ai_input, ai_train) = parse_content_signal(probe.content_signal.as_deref());

    MarkdownAgentsResult {
        supported,
        token_count,
        http_content_signal_search: search,
        http_content_signal_ai_input: ai_input,
        http_content_signal_ai_train: ai_train,
    }
}

/// Parse a `Content-Signal` HTTP header value.
///
/// Format: `key=value, key=value` (e.g. `ai-train=disallow, search=allow, ai-input=allow`).
fn parse_content_signal(
    header: Option<&str>,
) -> (Option<String>, Option<String>, Option<String>) {
    let Some(header) = header else {
        return (None, None, None);
    };

    let mut search = None;
    let mut ai_input = None;
    let mut ai_train = None;

    for pair in header.split(',') {
        let pair = pair.trim();
        if let Some((key, value)) = pair.split_once('=') {
            let key = key.trim().to_lowercase();
            let value = value.trim().to_lowercase();

            match key.as_str() {
                "search" => search = Some(value),
                "ai-input" => ai_input = Some(value),
                "ai-train" => ai_train = Some(value),
                _ => {} // Ignore unknown signals
            }
        }
    }

    (search, ai_input, ai_train)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supported_when_content_type_is_text_markdown() {
        // GIVEN a probe response with text/markdown content type
        let probe = MarkdownProbeData {
            status_code: 200,
            content_type: Some("text/markdown; charset=utf-8".to_string()),
            markdown_tokens: Some("4521".to_string()),
            content_signal: None,
        };

        // WHEN we evaluate
        let result = evaluate(&probe);

        // SHOULD detect as supported with correct token count
        assert!(result.supported);
        assert_eq!(result.token_count, Some(4521));
    }

    #[test]
    fn test_not_supported_when_content_type_is_html() {
        // GIVEN a probe response with text/html content type
        let probe = MarkdownProbeData {
            status_code: 200,
            content_type: Some("text/html; charset=utf-8".to_string()),
            markdown_tokens: None,
            content_signal: None,
        };

        // WHEN we evaluate
        let result = evaluate(&probe);

        // SHOULD not be supported
        assert!(!result.supported);
        assert_eq!(result.token_count, None);
    }

    #[test]
    fn test_malformed_token_count_returns_none() {
        // GIVEN a probe with a non-numeric token count
        let probe = MarkdownProbeData {
            status_code: 200,
            content_type: Some("text/markdown".to_string()),
            markdown_tokens: Some("not-a-number".to_string()),
            content_signal: None,
        };

        // WHEN we evaluate
        let result = evaluate(&probe);

        // SHOULD be supported but token count is None
        assert!(result.supported);
        assert_eq!(result.token_count, None);
    }

    #[test]
    fn test_partial_content_signal() {
        // GIVEN a probe with only some Content-Signal values
        let probe = MarkdownProbeData {
            status_code: 200,
            content_type: Some("text/markdown".to_string()),
            markdown_tokens: Some("1000".to_string()),
            content_signal: Some("search=allow, ai-train=disallow".to_string()),
        };

        // WHEN we evaluate
        let result = evaluate(&probe);

        // SHOULD parse present signals and leave missing ones as None
        assert!(result.supported);
        assert_eq!(
            result.http_content_signal_search,
            Some("allow".to_string())
        );
        assert_eq!(result.http_content_signal_ai_input, None);
        assert_eq!(
            result.http_content_signal_ai_train,
            Some("disallow".to_string())
        );
    }

    #[test]
    fn test_full_content_signal() {
        // GIVEN a probe with all three Content-Signal values
        let probe = MarkdownProbeData {
            status_code: 200,
            content_type: Some("text/markdown".to_string()),
            markdown_tokens: Some("2500".to_string()),
            content_signal: Some(
                "ai-train=disallow, search=allow, ai-input=allow".to_string(),
            ),
        };

        // WHEN we evaluate
        let result = evaluate(&probe);

        // SHOULD parse all three signals correctly
        assert!(result.supported);
        assert_eq!(result.token_count, Some(2500));
        assert_eq!(
            result.http_content_signal_search,
            Some("allow".to_string())
        );
        assert_eq!(
            result.http_content_signal_ai_input,
            Some("allow".to_string())
        );
        assert_eq!(
            result.http_content_signal_ai_train,
            Some("disallow".to_string())
        );
    }
}
