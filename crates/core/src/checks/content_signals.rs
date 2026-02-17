//! Cloudflare Content Signals extraction from robots.txt.
//!
//! Detects `Content-Signal:` directives from Cloudflare's AI policy framework.
//! Three signals: search, ai-input, ai-train (values: "yes" or "no").
//!
//! See: <https://blog.cloudflare.com/content-signals-policy>

/// Content Signals extraction result.
pub struct ContentSignalsResult {
    pub search: Option<String>,
    pub ai_input: Option<String>,
    pub ai_train: Option<String>,
}

/// Extract Content Signals from robots.txt content.
///
/// Parses `Content-Signal: search=yes, ai-train=no, ai-input=yes` directives.
/// Respects user-agent group scoping.
pub fn extract(content: &str, user_agent: &str) -> ContentSignalsResult {
    let mut search_signal = None;
    let mut ai_input_signal = None;
    let mut ai_train_signal = None;
    let mut in_matching_group = false;
    let mut current_user_agents: Vec<String> = Vec::new();

    for line in content.lines() {
        let line = line.trim();

        // Skip comments and empty lines
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let line_lower = line.to_lowercase();

        // Track user-agent groups
        if line_lower.starts_with("user-agent:") {
            if let Some(agent) = line.split(':').nth(1) {
                let agent = agent.trim();
                current_user_agents.push(agent.to_string());

                // Check if this matches our target user agent
                if agent == user_agent || agent == "*" {
                    in_matching_group = true;
                }
            }
        } else if line_lower.starts_with("content-signal:") {
            // Only process if we're in the matching group or no group (global)
            if current_user_agents.is_empty() || in_matching_group {
                // Extract the value part after "Content-Signal:"
                if let Some(signals_str) = line.split(':').nth(1) {
                    // Parse comma-separated key=value pairs
                    for pair in signals_str.split(',') {
                        let pair = pair.trim();
                        if let Some((key, value)) = pair.split_once('=') {
                            let key = key.trim().to_lowercase();
                            let value = value.trim().to_lowercase();

                            // Only accept "yes" or "no" values
                            if value == "yes" || value == "no" {
                                match key.as_str() {
                                    "search" => search_signal = Some(value),
                                    "ai-input" => ai_input_signal = Some(value),
                                    "ai-train" => ai_train_signal = Some(value),
                                    _ => {} // Ignore unknown signals
                                }
                            }
                        }
                    }
                }
            }
        } else if !line_lower.starts_with("allow:")
            && !line_lower.starts_with("disallow:")
            && !line_lower.starts_with("sitemap:")
            && !line_lower.starts_with("crawl-delay:")
            && !line_lower.starts_with("license:")
            && !current_user_agents.is_empty()
        {
            // Reset group context on unrecognized directive
            if line.contains(':') {
                current_user_agents.clear();
                in_matching_group = false;
            }
        }
    }

    ContentSignalsResult {
        search: search_signal,
        ai_input: ai_input_signal,
        ai_train: ai_train_signal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_signals_basic() {
        let content = r#"
User-agent: *
Content-Signal: search=yes, ai-train=no, ai-input=yes
Allow: /
        "#;
        let result = extract(content, "*");

        assert_eq!(result.search, Some("yes".to_string()));
        assert_eq!(result.ai_input, Some("yes".to_string()));
        assert_eq!(result.ai_train, Some("no".to_string()));
    }

    #[test]
    fn test_content_signals_partial() {
        let content = r#"
User-agent: *
Content-Signal: search=yes, ai-train=no
Allow: /
        "#;
        let result = extract(content, "*");

        assert_eq!(result.search, Some("yes".to_string()));
        assert_eq!(result.ai_input, None);
        assert_eq!(result.ai_train, Some("no".to_string()));
    }

    #[test]
    fn test_content_signals_group_scoped() {
        let content = r#"
User-agent: Googlebot
Content-Signal: search=yes, ai-train=yes

User-agent: GPTBot
Content-Signal: search=yes, ai-train=no, ai-input=no
Disallow: /
        "#;
        let result = extract(content, "GPTBot");

        assert_eq!(result.search, Some("yes".to_string()));
        assert_eq!(result.ai_input, Some("no".to_string()));
        assert_eq!(result.ai_train, Some("no".to_string()));
    }

    #[test]
    fn test_content_signals_cloudflare_format() {
        let content = r#"
User-Agent: *
Content-Signal: search=yes, ai-train=no
Allow: /
        "#;
        let result = extract(content, "*");

        assert_eq!(result.search, Some("yes".to_string()));
        assert_eq!(result.ai_input, None, "ai-input not specified");
        assert_eq!(result.ai_train, Some("no".to_string()));
    }
}
