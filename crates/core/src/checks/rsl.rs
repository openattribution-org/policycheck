//! RSL (Responsible Sourcing License) extraction from robots.txt.
//!
//! Detects `License:` directives per the RSL standard. Supports both
//! global licenses (outside any User-agent group) and group-scoped
//! licenses. Group-scoped licenses take precedence over global ones.
//!
//! See: <https://rslstandard.org/rsl>

/// RSL license extraction result.
pub struct RslResult {
    pub global_licenses: Vec<String>,
    pub group_licenses: Vec<String>,
    pub active_licenses: Vec<String>,
}

/// Extract RSL licenses from robots.txt content.
///
/// Returns global, group-scoped, and active (effective) licenses.
/// Precedence: group-scoped licenses override global licenses.
pub fn extract(content: &str, user_agent: &str) -> RslResult {
    let mut global_licenses = Vec::new();
    let mut group_licenses = Vec::new();
    let mut current_user_agents: Vec<String> = Vec::new();
    let mut in_matching_group = false;

    for line in content.lines() {
        let line = line.trim();

        // Skip comments and empty lines
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let line_lower = line.to_lowercase();

        if line_lower.starts_with("user-agent:") {
            // Extract user agent
            if let Some(agent) = line.split(':').nth(1) {
                let agent = agent.trim();
                current_user_agents.push(agent.to_string());

                // Check if this matches our target user agent
                if agent == user_agent || agent == "*" {
                    in_matching_group = true;
                }
            }
        } else if line_lower.starts_with("license:") {
            // Extract license URI
            if let Some(license_uri) = line
                .split(':')
                .skip(1)
                .collect::<Vec<_>>()
                .join(":")
                .split_whitespace()
                .next()
            {
                let license = license_uri.trim().to_string();

                if !license.is_empty() {
                    // Validate it's an absolute URI (basic check)
                    if license.starts_with("http://") || license.starts_with("https://") {
                        if current_user_agents.is_empty() {
                            // Global license (outside any user-agent group)
                            if !global_licenses.contains(&license) {
                                global_licenses.push(license);
                            }
                        } else if in_matching_group {
                            // Group-scoped license for our user agent
                            if !group_licenses.contains(&license) {
                                group_licenses.push(license);
                            }
                        }
                    }
                }
            }
        } else if !line_lower.starts_with("allow:")
            && !line_lower.starts_with("disallow:")
            && !line_lower.starts_with("sitemap:")
            && !line_lower.starts_with("crawl-delay:")
            && !current_user_agents.is_empty()
        {
            // Reset group context on unrecognized directive (new group likely starting)
            if line.contains(':') {
                current_user_agents.clear();
                in_matching_group = false;
            }
        }
    }

    // Determine active licenses based on RSL precedence rules
    let active_licenses = if !group_licenses.is_empty() {
        group_licenses.clone()
    } else {
        global_licenses.clone()
    };

    RslResult {
        global_licenses,
        group_licenses,
        active_licenses,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_global_licenses() {
        let content = "License: https://example.com/license.xml\nUser-agent: *\nDisallow: /private";
        let result = extract(content, "*");

        assert_eq!(result.global_licenses.len(), 1);
        assert_eq!(result.global_licenses[0], "https://example.com/license.xml");
        assert_eq!(result.group_licenses.len(), 0);
    }

    #[test]
    fn test_extract_group_scoped_licenses() {
        let content = r#"
License: https://example.com/global.xml
User-agent: GPTBot
License: https://example.com/gptbot.xml
Disallow: /
        "#;
        let result = extract(content, "GPTBot");

        assert_eq!(result.global_licenses.len(), 1);
        assert_eq!(result.global_licenses[0], "https://example.com/global.xml");
        assert_eq!(result.group_licenses.len(), 1);
        assert_eq!(result.group_licenses[0], "https://example.com/gptbot.xml");
    }

    #[test]
    fn test_license_precedence_group_overrides_global() {
        let content = r#"
License: https://example.com/global.xml
User-agent: GPTBot
License: https://example.com/gptbot.xml
        "#;
        let result = extract(content, "GPTBot");

        assert_eq!(result.active_licenses.len(), 1);
        assert_eq!(result.active_licenses[0], "https://example.com/gptbot.xml");
    }

    #[test]
    fn test_license_requires_absolute_uri() {
        let content = "License: /relative/path.xml\nLicense: https://example.com/absolute.xml";
        let result = extract(content, "*");

        assert_eq!(result.global_licenses.len(), 1);
        assert_eq!(result.global_licenses[0], "https://example.com/absolute.xml");
    }

    #[test]
    fn test_license_ignores_comments() {
        let content = r#"
# This is a comment with License: https://fake.com/license.xml
License: https://example.com/real.xml
        "#;
        let result = extract(content, "*");

        assert_eq!(result.global_licenses.len(), 1);
        assert_eq!(result.global_licenses[0], "https://example.com/real.xml");
    }

    #[test]
    fn test_wildcard_user_agent_matches() {
        let content = r#"
User-agent: *
License: https://example.com/wildcard.xml
        "#;
        let result = extract(content, "MyBot");

        assert_eq!(result.group_licenses.len(), 1);
        assert_eq!(result.group_licenses[0], "https://example.com/wildcard.xml");
    }

    #[test]
    fn test_separate_user_agent_groups_dont_mix_licenses() {
        let content = r#"
User-agent: Googlebot
License: https://example.com/google-only.xml
Disallow: /admin

User-agent: GPTBot
License: https://example.com/gpt-only.xml
Disallow: /
        "#;
        let result = extract(content, "GPTBot");

        assert_eq!(result.global_licenses.len(), 0, "Should have no global licenses");
        assert_eq!(result.group_licenses.len(), 1, "Should have exactly 1 group license");
        assert_eq!(result.group_licenses[0], "https://example.com/gpt-only.xml");
        assert!(
            !result.group_licenses.contains(&"https://example.com/google-only.xml".to_string()),
            "Should NOT include Googlebot's license"
        );
    }

    #[test]
    fn test_rsl_real_world_rslstandard_org() {
        let content = r#"
License: https://rslcollective.org/royalty.xml

User-agent: *
Disallow:
        "#;
        let result = extract(content, "*");

        assert_eq!(result.global_licenses.len(), 1);
        assert_eq!(result.global_licenses[0], "https://rslcollective.org/royalty.xml");
        assert_eq!(result.group_licenses.len(), 0, "No group-scoped licenses");
        assert_eq!(result.active_licenses[0], "https://rslcollective.org/royalty.xml");
    }

    #[test]
    fn test_rsl_real_world_medium_com() {
        let content = r#"
User-agent: *
Allow: /about

User-agent: GPTBot
User-agent: ClaudeBot
User-agent: FacebookBot
License: https://medium.com/license.xml
Disallow: /
        "#;
        let result = extract(content, "GPTBot");

        assert_eq!(result.global_licenses.len(), 0, "No global licenses");
        assert_eq!(result.group_licenses.len(), 1, "Should have group license");
        assert_eq!(result.group_licenses[0], "https://medium.com/license.xml");
        assert_eq!(result.active_licenses[0], "https://medium.com/license.xml");
    }
}
