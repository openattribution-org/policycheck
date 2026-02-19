//! Meta robots tag and X-Robots-Tag header parsing.
//!
//! Analyses `<meta name="robots">` HTML tags and `X-Robots-Tag` HTTP headers
//! to determine page-level indexing directives. Bot-specific entries override
//! generic `robots` entries.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RobotsMetaSource {
    MetaTag,
    HttpHeader,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RobotsDirective {
    NoIndex,
    NoFollow,
    None,
    All,
    NoArchive,
    NoSnippet,
    NoImageIndex,
    MaxSnippet(i32),
    MaxImagePreview(String),
    MaxVideoPreview(i32),
    Unknown(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobotsMetaEntry {
    pub source: RobotsMetaSource,
    pub bot_name: Option<String>,
    pub directives: Vec<RobotsDirective>,
    pub raw: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobotsMetaResult {
    pub entries: Vec<RobotsMetaEntry>,
    pub is_noindex: bool,
    pub is_nofollow: bool,
}

/// Parse a comma-separated directive string into a list of `RobotsDirective`.
fn parse_directives(content: &str) -> Vec<RobotsDirective> {
    content
        .split(',')
        .map(|d| d.trim())
        .filter(|d| !d.is_empty())
        .map(|d| {
            let lower = d.to_lowercase();
            if lower == "noindex" {
                RobotsDirective::NoIndex
            } else if lower == "nofollow" {
                RobotsDirective::NoFollow
            } else if lower == "none" {
                RobotsDirective::None
            } else if lower == "all" {
                RobotsDirective::All
            } else if lower == "noarchive" {
                RobotsDirective::NoArchive
            } else if lower == "nosnippet" {
                RobotsDirective::NoSnippet
            } else if lower == "noimageindex" {
                RobotsDirective::NoImageIndex
            } else if let Some(val) = lower.strip_prefix("max-snippet:") {
                val.trim()
                    .parse::<i32>()
                    .map(RobotsDirective::MaxSnippet)
                    .unwrap_or(RobotsDirective::Unknown(d.to_string()))
            } else if let Some(val) = lower.strip_prefix("max-image-preview:") {
                RobotsDirective::MaxImagePreview(val.trim().to_string())
            } else if let Some(val) = lower.strip_prefix("max-video-preview:") {
                val.trim()
                    .parse::<i32>()
                    .map(RobotsDirective::MaxVideoPreview)
                    .unwrap_or(RobotsDirective::Unknown(d.to_string()))
            } else {
                RobotsDirective::Unknown(d.to_string())
            }
        })
        .collect()
}

/// Extract meta robots tags from HTML, scanning only within `<head>` or the first 64KB.
fn parse_meta_tags(html: &str) -> Vec<RobotsMetaEntry> {
    let mut entries = Vec::new();

    // Limit scanning to <head> section or first 64KB
    let scan_region = if let Some(end_pos) = html
        .as_bytes()
        .windows(7)
        .position(|w| w.eq_ignore_ascii_case(b"</head>"))
    {
        &html[..end_pos]
    } else {
        let limit = html.len().min(65_536);
        &html[..limit]
    };

    // Find all <meta tags
    let lower = scan_region.to_lowercase();
    let mut search_from = 0;

    while let Some(meta_start) = lower[search_from..].find("<meta") {
        let abs_start = search_from + meta_start;
        let tag_region = &lower[abs_start..];
        let orig_region = &scan_region[abs_start..];

        let tag_end = match tag_region.find('>') {
            Some(pos) => pos + 1,
            Option::None => {
                search_from = abs_start + 5;
                continue;
            }
        };

        let tag_lower = &tag_region[..tag_end];
        let tag_orig = &orig_region[..tag_end];

        // Extract name attribute
        let name_val = extract_attr(tag_lower, "name");

        if let Some(ref name) = name_val {
            // Must be "robots" or a bot-specific name
            if name == "robots" || is_bot_name(name) {
                if let Some(content) = extract_attr(tag_lower, "content") {
                    let directives = parse_directives(&content);
                    let bot_name = if name == "robots" {
                        Option::None
                    } else {
                        // Use original case from the tag
                        let orig_name = extract_attr(tag_orig, "name");
                        Some(orig_name.unwrap_or_else(|| name.clone()))
                    };

                    entries.push(RobotsMetaEntry {
                        source: RobotsMetaSource::MetaTag,
                        bot_name,
                        directives,
                        raw: tag_orig.to_string(),
                    });
                }
            }
        }

        search_from = abs_start + tag_end;
    }

    entries
}

/// Extract an attribute value from a lowercased tag string.
fn extract_attr(tag: &str, attr_name: &str) -> Option<String> {
    // Match attr_name= followed by quoted or unquoted value
    let patterns = [
        format!("{}=\"", attr_name),
        format!("{}='", attr_name),
        format!("{}=", attr_name),
    ];

    for (i, pattern) in patterns.iter().enumerate() {
        if let Some(start) = tag.find(pattern.as_str()) {
            let val_start = start + pattern.len();
            let remaining = &tag[val_start..];

            let val = if i < 2 {
                // Quoted value
                let quote = if i == 0 { '"' } else { '\'' };
                remaining
                    .find(quote)
                    .map(|end| remaining[..end].to_string())
            } else {
                // Unquoted value — ends at whitespace or >
                let end = remaining
                    .find(|c: char| c.is_whitespace() || c == '>' || c == '/')
                    .unwrap_or(remaining.len());
                Some(remaining[..end].to_string())
            };

            if let Some(v) = val {
                return Some(v);
            }
        }
    }

    Option::None
}

/// Known bot-specific meta names (lowercase).
fn is_bot_name(name: &str) -> bool {
    matches!(
        name,
        "googlebot"
            | "bingbot"
            | "gptbot"
            | "claudebot"
            | "anthropic-ai"
            | "google-extended"
            | "googlebot-news"
            | "googlebot-image"
            | "googlebot-video"
    )
}

/// Parse `X-Robots-Tag` HTTP header values.
///
/// Supports two forms:
/// - `directive, directive` (applies to all bots)
/// - `botname: directive, directive` (bot-specific)
fn parse_x_robots_headers(headers: &[String]) -> Vec<RobotsMetaEntry> {
    headers
        .iter()
        .map(|header| {
            let trimmed = header.trim();

            // Check for bot-specific form: "botname: directives"
            let (bot_name, directives_str) = if let Some(colon_pos) = trimmed.find(':') {
                let candidate = trimmed[..colon_pos].trim();
                // Only treat as bot name if the candidate is a single token (no spaces)
                // and doesn't look like a directive itself
                if !candidate.contains(' ') && !candidate.contains(',') {
                    let lower = candidate.to_lowercase();
                    if is_known_x_robots_bot(&lower) {
                        (Some(candidate.to_string()), &trimmed[colon_pos + 1..])
                    } else {
                        // Could be a directive like max-snippet:5
                        (Option::None, trimmed)
                    }
                } else {
                    (Option::None, trimmed)
                }
            } else {
                (Option::None, trimmed)
            };

            let directives = parse_directives(directives_str);

            RobotsMetaEntry {
                source: RobotsMetaSource::HttpHeader,
                bot_name,
                directives,
                raw: header.clone(),
            }
        })
        .collect()
}

/// Check if a name (lowercase) is a known bot for X-Robots-Tag purposes.
fn is_known_x_robots_bot(name: &str) -> bool {
    is_bot_name(name)
        || matches!(
            name,
            "otherbot"
                | "yandexbot"
                | "duckduckbot"
                | "slurp"
                | "baiduspider"
                | "perplexitybot"
                | "oai-searchbot"
        )
}

/// Analyse HTML content and X-Robots-Tag headers for page-level robots directives.
///
/// `user_agent` is used to determine which bot-specific entries apply.
/// Bot-specific entries override generic `robots` entries.
pub fn analyze(html: &str, x_robots_headers: &[String], user_agent: &str) -> RobotsMetaResult {
    let mut entries = parse_meta_tags(html);
    entries.extend(parse_x_robots_headers(x_robots_headers));

    let ua_lower = user_agent.to_lowercase();

    // Find bot-specific entries matching our user agent
    let bot_specific: Vec<&RobotsMetaEntry> = entries
        .iter()
        .filter(|e| {
            e.bot_name
                .as_ref()
                .is_some_and(|name| name.to_lowercase() == ua_lower)
        })
        .collect();

    // If there are bot-specific entries, use those; otherwise use generic ones
    let effective: Vec<&RobotsMetaEntry> = if !bot_specific.is_empty() {
        bot_specific
    } else {
        entries.iter().filter(|e| e.bot_name.is_none()).collect()
    };

    let mut is_noindex = false;
    let mut is_nofollow = false;

    for entry in &effective {
        for directive in &entry.directives {
            match directive {
                RobotsDirective::NoIndex => is_noindex = true,
                RobotsDirective::NoFollow => is_nofollow = true,
                RobotsDirective::None => {
                    is_noindex = true;
                    is_nofollow = true;
                }
                RobotsDirective::All => {
                    // all = index + follow (no restrictions)
                }
                _ => {}
            }
        }
    }

    RobotsMetaResult {
        entries,
        is_noindex,
        is_nofollow,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_noindex_nofollow() {
        let html = r#"<html><head><meta name="robots" content="noindex, nofollow"></head></html>"#;
        let result = analyze(html, &[], "*");
        assert!(result.is_noindex);
        assert!(result.is_nofollow);
        assert_eq!(result.entries.len(), 1);
    }

    #[test]
    fn test_none_expands_to_noindex_nofollow() {
        let html = r#"<html><head><meta name="robots" content="none"></head></html>"#;
        let result = analyze(html, &[], "*");
        assert!(result.is_noindex);
        assert!(result.is_nofollow);
    }

    #[test]
    fn test_all_means_no_restrictions() {
        let html = r#"<html><head><meta name="robots" content="all"></head></html>"#;
        let result = analyze(html, &[], "*");
        assert!(!result.is_noindex);
        assert!(!result.is_nofollow);
    }

    #[test]
    fn test_bot_specific_tag() {
        let html = r#"<html><head><meta name="googlebot" content="noindex"></head></html>"#;
        let result = analyze(html, &[], "googlebot");
        assert!(result.is_noindex);
        assert!(!result.is_nofollow);
    }

    #[test]
    fn test_bot_specific_overrides_generic() {
        let html = r#"<html><head>
            <meta name="robots" content="noindex, nofollow">
            <meta name="googlebot" content="all">
        </head></html>"#;
        let result = analyze(html, &[], "googlebot");
        // Bot-specific "all" overrides generic "noindex, nofollow"
        assert!(!result.is_noindex);
        assert!(!result.is_nofollow);
    }

    #[test]
    fn test_case_insensitive_directives() {
        let html = r#"<html><head><meta name="ROBOTS" content="NOINDEX, NOFOLLOW"></head></html>"#;
        let result = analyze(html, &[], "*");
        assert!(result.is_noindex);
        assert!(result.is_nofollow);
    }

    #[test]
    fn test_reversed_attribute_order() {
        let html = r#"<html><head><meta content="noindex" name="robots"></head></html>"#;
        let result = analyze(html, &[], "*");
        assert!(result.is_noindex);
        assert!(!result.is_nofollow);
    }

    #[test]
    fn test_x_robots_tag_parsing() {
        let headers = vec!["noindex, nofollow".to_string()];
        let result = analyze("", &headers, "*");
        assert!(result.is_noindex);
        assert!(result.is_nofollow);
        assert_eq!(result.entries.len(), 1);
        assert_eq!(result.entries[0].source, RobotsMetaSource::HttpHeader);
    }

    #[test]
    fn test_x_robots_tag_bot_specific() {
        let headers = vec!["googlebot: noindex".to_string()];
        let result = analyze("", &headers, "googlebot");
        assert!(result.is_noindex);
        assert!(!result.is_nofollow);
    }

    #[test]
    fn test_max_snippet_directive() {
        let html =
            r#"<html><head><meta name="robots" content="max-snippet:50, noarchive"></head></html>"#;
        let result = analyze(html, &[], "*");
        assert!(!result.is_noindex);
        assert!(!result.is_nofollow);
        let directives = &result.entries[0].directives;
        assert!(matches!(directives[0], RobotsDirective::MaxSnippet(50)));
        assert!(matches!(directives[1], RobotsDirective::NoArchive));
    }

    #[test]
    fn test_empty_input() {
        let result = analyze("", &[], "*");
        assert!(!result.is_noindex);
        assert!(!result.is_nofollow);
        assert!(result.entries.is_empty());
    }

    #[test]
    fn test_head_only_scanning() {
        // Meta tag after </head> should be ignored
        let html = r#"<html><head><title>Test</title></head><body><meta name="robots" content="noindex"></body></html>"#;
        let result = analyze(html, &[], "*");
        assert!(!result.is_noindex);
        assert!(result.entries.is_empty());
    }

    #[test]
    fn test_multiple_meta_tags() {
        let html = r#"<html><head>
            <meta name="robots" content="noindex">
            <meta name="robots" content="nofollow">
        </head></html>"#;
        let result = analyze(html, &[], "*");
        assert!(result.is_noindex);
        assert!(result.is_nofollow);
        assert_eq!(result.entries.len(), 2);
    }

    #[test]
    fn test_noindex_only() {
        let html = r#"<html><head><meta name="robots" content="noindex"></head></html>"#;
        let result = analyze(html, &[], "*");
        assert!(result.is_noindex);
        assert!(!result.is_nofollow);
    }

    #[test]
    fn test_generic_not_applied_when_bot_specific_exists() {
        let html = r#"<html><head>
            <meta name="robots" content="noindex, nofollow">
            <meta name="gptbot" content="noindex">
        </head></html>"#;
        // When querying as gptbot, only the gptbot entry applies
        let result = analyze(html, &[], "gptbot");
        assert!(result.is_noindex);
        assert!(!result.is_nofollow); // nofollow was only on generic
    }
}
