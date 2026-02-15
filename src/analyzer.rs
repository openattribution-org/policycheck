use crate::ai_crawlers::{AICrawler, BotStatus};
use crate::fetcher::RobotFetcher;
use crate::models::{AnalysisResult, AnalysisStatus, BotAnalysisResult, TdmPolicy, TdmRule};
use anyhow::Result;
use std::path::Path;
use texting_robots::Robot;
use url::Url;

pub struct RobotAnalyzer {
    user_agent: String,
    fetcher: RobotFetcher,
}

impl RobotAnalyzer {
    pub fn new(user_agent: String) -> Self {
        Self {
            user_agent,
            fetcher: RobotFetcher::new(),
        }
    }

    pub fn with_fetcher(user_agent: String, fetcher: RobotFetcher) -> Self {
        Self {
            user_agent,
            fetcher,
        }
    }

    /// Read URLs from a CSV file
    pub fn read_csv(&self, path: &Path) -> Result<Vec<String>> {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_path(path)?;

        let mut urls = Vec::new();

        // Get headers to find URL column
        let headers = reader.headers()?.clone();

        // Find the URL column index (look for "url", "URL", "Company URL", etc.)
        let url_col_idx = headers
            .iter()
            .position(|h| {
                let h_lower = h.to_lowercase();
                h_lower.contains("url") || h_lower == "link" || h_lower == "website"
            })
            .unwrap_or(0); // Default to first column if no URL header found

        // Read URLs from the identified column
        for result in reader.records() {
            let record = result?;

            if let Some(url) = record.get(url_col_idx) {
                let url = url.trim();
                if !url.is_empty() {
                    // Add http:// prefix if missing
                    let url = if url.starts_with("http://") || url.starts_with("https://") {
                        url.to_string()
                    } else if !url.is_empty() {
                        format!("https://{}", url)
                    } else {
                        continue;
                    };
                    urls.push(url);
                }
            }
        }

        Ok(urls)
    }

    /// Analyze a single URL
    pub async fn analyze_url(&self, url: &str) -> AnalysisResult {
        // Fetch robots.txt
        let (robots_url, content) = match self.fetcher.fetch_for_url(url).await {
            Ok(data) => data,
            Err(e) => {
                return AnalysisResult::error(
                    url.to_string(),
                    e.to_string(),
                    AnalysisStatus::FetchError,
                );
            }
        };

        // Parse robots.txt
        let robot = match Robot::new(&self.user_agent, content.as_bytes()) {
            Ok(r) => r,
            Err(e) => {
                return AnalysisResult::error(
                    url.to_string(),
                    format!("Parse error: {:?}", e),
                    AnalysisStatus::ParseError,
                );
            }
        };

        // Extract user agents from the content
        let user_agents = self.extract_user_agents(&content);

        // Extract allowed and disallowed paths
        let (allowed_paths, disallowed_paths) = self.extract_paths(&content);

        // Extract RSL licenses
        let (global_licenses, group_licenses) = self.extract_licenses(&content);

        // Determine active licenses based on RSL precedence rules
        let active_licenses = if !group_licenses.is_empty() {
            group_licenses.clone()
        } else {
            global_licenses.clone()
        };

        // Extract Content Signals (Cloudflare's AI policy framework)
        let (content_signal_search, content_signal_ai_input, content_signal_ai_train) =
            self.extract_content_signals(&content);

        // Check if the original URL path is allowed
        let is_path_allowed = robot.allowed(url);

        // Fetch and evaluate TDM policy
        let tdm_policy = match self.fetcher.fetch_tdm_policy(url).await {
            Ok(rules) => self.evaluate_tdm_policy(url, rules).await,
            Err(_) => None, // TDM policy is optional, ignore errors
        };

        // Analyze AI bot access
        let ai_bot_analysis = self.analyze_ai_bots(&content, url);

        AnalysisResult {
            url: url.to_string(),
            robots_url,
            status: AnalysisStatus::Success,
            user_agents,
            crawl_delay: robot.delay.map(|d| d as f64),
            sitemaps: robot.sitemaps.clone(),
            allowed_paths,
            disallowed_paths,
            is_path_allowed,
            global_licenses,
            group_licenses,
            active_licenses,
            content_signal_search,
            content_signal_ai_input,
            content_signal_ai_train,
            tdm_policy,
            ai_bot_analysis,
            error: None,
        }
    }

    /// Analyze multiple URLs concurrently
    pub async fn analyze_urls(&self, urls: &[String]) -> Vec<AnalysisResult> {
        let mut handles = vec![];

        for url in urls {
            let url = url.clone();
            let url_for_error = url.clone();
            let user_agent = self.user_agent.clone();
            let fetcher = self.fetcher.clone();

            let handle = tokio::spawn(async move {
                let analyzer = RobotAnalyzer::with_fetcher(user_agent, fetcher);
                analyzer.analyze_url(&url).await
            });

            handles.push((url_for_error, handle));
        }

        let mut results = vec![];
        for (url, handle) in handles {
            match handle.await {
                Ok(result) => results.push(result),
                Err(e) => results.push(AnalysisResult::error(
                    url,
                    format!("Task failed: {}", e),
                    AnalysisStatus::FetchError,
                )),
            }
        }

        results
    }

    /// Extract user agents from robots.txt content
    #[allow(clippy::collapsible_if)]
    fn extract_user_agents(&self, content: &str) -> Vec<String> {
        let mut agents = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            if line.to_lowercase().starts_with("user-agent:") {
                if let Some(agent) = line.split(':').nth(1) {
                    let agent = agent.trim().to_string();
                    if !agents.contains(&agent) {
                        agents.push(agent);
                    }
                }
            }
        }

        agents
    }

    /// Extract allowed and disallowed paths
    #[allow(clippy::collapsible_if)]
    fn extract_paths(&self, content: &str) -> (Vec<String>, Vec<String>) {
        let mut allowed = Vec::new();
        let mut disallowed = Vec::new();

        for line in content.lines() {
            let line = line.trim();

            if line.to_lowercase().starts_with("allow:") {
                if let Some(path) = line.split(':').nth(1) {
                    let path = path.trim();
                    if !path.is_empty() && !allowed.contains(&path.to_string()) {
                        allowed.push(path.to_string());
                    }
                }
            } else if line.to_lowercase().starts_with("disallow:") {
                if let Some(path) = line.split(':').nth(1) {
                    let path = path.trim();
                    if !path.is_empty() && !disallowed.contains(&path.to_string()) {
                        disallowed.push(path.to_string());
                    }
                }
            }
        }

        (allowed, disallowed)
    }

    /// Extract RSL licenses from robots.txt content
    /// Returns (global_licenses, group_licenses) where:
    /// - global_licenses: License directives outside any User-agent group
    /// - group_licenses: License directives within the matching User-agent group
    fn extract_licenses(&self, content: &str) -> (Vec<String>, Vec<String>) {
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
                    if agent == self.user_agent || agent == "*" {
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

        (global_licenses, group_licenses)
    }

    /// Extract Content Signals from robots.txt (Cloudflare's AI policy framework)
    /// Returns (search, ai-input, ai-train) signals as Option<String>
    /// Format: Content-Signal: search=yes, ai-train=no, ai-input=yes
    fn extract_content_signals(
        &self,
        content: &str,
    ) -> (Option<String>, Option<String>, Option<String>) {
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
                    if agent == self.user_agent || agent == "*" {
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

        (search_signal, ai_input_signal, ai_train_signal)
    }

    /// Match a path against a TDM location pattern
    /// Supports * wildcard and $ end-of-pattern marker
    fn match_tdm_pattern(pattern: &str, path: &str) -> bool {
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

    /// Evaluate TDM rules and find the matching rule for a URL
    async fn evaluate_tdm_policy(&self, url: &str, rules: Vec<TdmRule>) -> Option<TdmPolicy> {
        // Parse URL to get the path
        let parsed = Url::parse(url).ok()?;
        let path = parsed.path();

        // Find the first matching rule (first-match wins per W3C TDMRep)
        let matched_rule = rules
            .iter()
            .find(|rule| Self::match_tdm_pattern(&rule.location, path))
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

    /// Analyze each AI bot individually to determine its access status
    fn analyze_ai_bots(&self, content: &str, url: &str) -> Vec<BotAnalysisResult> {
        let all_bots = AICrawler::get_all();
        let mut results = Vec::new();

        // Normalize user agents for comparison (case-insensitive)
        let user_agents_lower: Vec<String> = self
            .extract_user_agents(content)
            .iter()
            .map(|ua| ua.to_lowercase())
            .collect();

        for bot in all_bots {
            let bot_name_lower = bot.name.to_lowercase();

            // Check if this bot is mentioned in robots.txt
            let is_mentioned = user_agents_lower
                .iter()
                .any(|ua| ua == &bot_name_lower || ua.contains(&bot_name_lower));

            let status = if is_mentioned {
                // Bot is mentioned - check if it's allowed or blocked for this path
                // texting_robots::Robot bakes the user-agent into its parsed state,
                // so we must re-parse for each bot to get per-bot allow/disallow results.
                match Robot::new(&bot.name, content.as_bytes()) {
                    Ok(robot) => {
                        if robot.allowed(url) {
                            BotStatus::Allowed
                        } else {
                            BotStatus::Blocked
                        }
                    }
                    Err(_) => BotStatus::Allowed, // Parse error, default to allowed
                }
            } else {
                // Bot not mentioned - allowed by default (follows wildcard rules or no restrictions)
                BotStatus::Allowed
            };

            results.push(BotAnalysisResult {
                bot_name: bot.name,
                company: bot.company,
                category: format!("{:?}", bot.category),
                status,
            });
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_user_agents() {
        let analyzer = RobotAnalyzer::new("TestBot".to_string());
        let content = "User-agent: *\nUser-agent: GoogleBot\nUser-agent: BingBot";
        let agents = analyzer.extract_user_agents(content);

        assert_eq!(agents.len(), 3);
        assert!(agents.contains(&"*".to_string()));
        assert!(agents.contains(&"GoogleBot".to_string()));
        assert!(agents.contains(&"BingBot".to_string()));
    }

    #[test]
    fn test_extract_paths() {
        let analyzer = RobotAnalyzer::new("*".to_string());
        let content = "Allow: /public\nDisallow: /private\nDisallow: /admin";
        let (allowed, disallowed) = analyzer.extract_paths(content);

        assert_eq!(allowed.len(), 1);
        assert_eq!(allowed[0], "/public");
        assert_eq!(disallowed.len(), 2);
        assert!(disallowed.contains(&"/private".to_string()));
        assert!(disallowed.contains(&"/admin".to_string()));
    }

    #[test]
    fn test_extract_global_licenses() {
        let analyzer = RobotAnalyzer::new("*".to_string());
        let content = "License: https://example.com/license.xml\nUser-agent: *\nDisallow: /private";
        let (global, group) = analyzer.extract_licenses(content);

        assert_eq!(global.len(), 1);
        assert_eq!(global[0], "https://example.com/license.xml");
        assert_eq!(group.len(), 0);
    }

    #[test]
    fn test_extract_group_scoped_licenses() {
        let analyzer = RobotAnalyzer::new("GPTBot".to_string());
        let content = r#"
License: https://example.com/global.xml
User-agent: GPTBot
License: https://example.com/gptbot.xml
Disallow: /
        "#;
        let (global, group) = analyzer.extract_licenses(content);

        assert_eq!(global.len(), 1);
        assert_eq!(global[0], "https://example.com/global.xml");
        assert_eq!(group.len(), 1);
        assert_eq!(group[0], "https://example.com/gptbot.xml");
    }

    #[test]
    fn test_license_precedence_group_overrides_global() {
        let analyzer = RobotAnalyzer::new("GPTBot".to_string());
        let content = r#"
License: https://example.com/global.xml
User-agent: GPTBot
License: https://example.com/gptbot.xml
        "#;
        let (global, group) = analyzer.extract_licenses(content);

        // Active licenses should be group-scoped when present
        let active = if !group.is_empty() { &group } else { &global };
        assert_eq!(active.len(), 1);
        assert_eq!(active[0], "https://example.com/gptbot.xml");
    }

    #[test]
    fn test_license_requires_absolute_uri() {
        let analyzer = RobotAnalyzer::new("*".to_string());
        let content = "License: /relative/path.xml\nLicense: https://example.com/absolute.xml";
        let (global, _) = analyzer.extract_licenses(content);

        // Should only include absolute URIs
        assert_eq!(global.len(), 1);
        assert_eq!(global[0], "https://example.com/absolute.xml");
    }

    #[test]
    fn test_license_ignores_comments() {
        let analyzer = RobotAnalyzer::new("*".to_string());
        let content = r#"
# This is a comment with License: https://fake.com/license.xml
License: https://example.com/real.xml
        "#;
        let (global, _) = analyzer.extract_licenses(content);

        assert_eq!(global.len(), 1);
        assert_eq!(global[0], "https://example.com/real.xml");
    }

    #[test]
    fn test_wildcard_user_agent_matches() {
        let analyzer = RobotAnalyzer::new("MyBot".to_string());
        let content = r#"
User-agent: *
License: https://example.com/wildcard.xml
        "#;
        let (_, group) = analyzer.extract_licenses(content);

        // Wildcard should match any user agent
        assert_eq!(group.len(), 1);
        assert_eq!(group[0], "https://example.com/wildcard.xml");
    }

    #[test]
    fn test_content_signals_basic() {
        let analyzer = RobotAnalyzer::new("*".to_string());
        let content = r#"
User-agent: *
Content-Signal: search=yes, ai-train=no, ai-input=yes
Allow: /
        "#;
        let (search, ai_input, ai_train) = analyzer.extract_content_signals(content);

        assert_eq!(search, Some("yes".to_string()));
        assert_eq!(ai_input, Some("yes".to_string()));
        assert_eq!(ai_train, Some("no".to_string()));
    }

    #[test]
    fn test_content_signals_partial() {
        let analyzer = RobotAnalyzer::new("*".to_string());
        let content = r#"
User-agent: *
Content-Signal: search=yes, ai-train=no
Allow: /
        "#;
        let (search, ai_input, ai_train) = analyzer.extract_content_signals(content);

        assert_eq!(search, Some("yes".to_string()));
        assert_eq!(ai_input, None);
        assert_eq!(ai_train, Some("no".to_string()));
    }

    #[test]
    fn test_content_signals_group_scoped() {
        let analyzer = RobotAnalyzer::new("GPTBot".to_string());
        let content = r#"
User-agent: Googlebot
Content-Signal: search=yes, ai-train=yes

User-agent: GPTBot
Content-Signal: search=yes, ai-train=no, ai-input=no
Disallow: /
        "#;
        let (search, ai_input, ai_train) = analyzer.extract_content_signals(content);

        // Should only get signals from GPTBot group
        assert_eq!(search, Some("yes".to_string()));
        assert_eq!(ai_input, Some("no".to_string()));
        assert_eq!(ai_train, Some("no".to_string()));
    }

    #[test]
    fn test_content_signals_cloudflare_format() {
        // Test the exact format used by Cloudflare's managed robots.txt
        let analyzer = RobotAnalyzer::new("*".to_string());
        let content = r#"
User-Agent: *
Content-Signal: search=yes, ai-train=no
Allow: /
        "#;
        let (search, ai_input, ai_train) = analyzer.extract_content_signals(content);

        assert_eq!(search, Some("yes".to_string()));
        assert_eq!(ai_input, None, "ai-input not specified");
        assert_eq!(ai_train, Some("no".to_string()));
    }

    #[test]
    fn test_separate_user_agent_groups_dont_mix_licenses() {
        let analyzer = RobotAnalyzer::new("GPTBot".to_string());
        let content = r#"
User-agent: Googlebot
License: https://example.com/google-only.xml
Disallow: /admin

User-agent: GPTBot
License: https://example.com/gpt-only.xml
Disallow: /
        "#;
        let (global, group) = analyzer.extract_licenses(content);

        // Should not collect licenses from other user-agent groups
        assert_eq!(global.len(), 0, "Should have no global licenses");
        assert_eq!(group.len(), 1, "Should have exactly 1 group license");
        assert_eq!(group[0], "https://example.com/gpt-only.xml");
        assert!(
            !group.contains(&"https://example.com/google-only.xml".to_string()),
            "Should NOT include Googlebot's license"
        );
    }

    #[test]
    fn test_rsl_real_world_rslstandard_org() {
        // Based on rslstandard.org's robots.txt (as of 2026-02-14)
        // Tests global license pattern used by RSL Standard's own site
        let analyzer = RobotAnalyzer::new("*".to_string());
        let content = r#"
License: https://rslcollective.org/royalty.xml

User-agent: *
Disallow:
        "#;
        let (global, group) = analyzer.extract_licenses(content);

        assert_eq!(global.len(), 1);
        assert_eq!(global[0], "https://rslcollective.org/royalty.xml");
        assert_eq!(group.len(), 0, "No group-scoped licenses");

        // Active licenses should be global
        let active = if !group.is_empty() { &group } else { &global };
        assert_eq!(active[0], "https://rslcollective.org/royalty.xml");
    }

    #[test]
    fn test_rsl_real_world_medium_com() {
        // Based on medium.com's robots.txt (as of 2026-02-14)
        // Tests group-scoped license pattern with multiple specific bots
        let analyzer = RobotAnalyzer::new("GPTBot".to_string());
        let content = r#"
User-agent: *
Allow: /about

User-agent: GPTBot
User-agent: ClaudeBot
User-agent: FacebookBot
License: https://medium.com/license.xml
Disallow: /
        "#;
        let (global, group) = analyzer.extract_licenses(content);

        assert_eq!(global.len(), 0, "No global licenses");
        assert_eq!(group.len(), 1, "Should have group license");
        assert_eq!(group[0], "https://medium.com/license.xml");

        // Active licenses should be group-scoped
        let active = if !group.is_empty() { &group } else { &global };
        assert_eq!(active[0], "https://medium.com/license.xml");
    }

    #[test]
    fn test_tdm_pattern_exact_match() {
        assert!(RobotAnalyzer::match_tdm_pattern("/", "/"));
        assert!(RobotAnalyzer::match_tdm_pattern("/docs", "/docs"));
        assert!(RobotAnalyzer::match_tdm_pattern("/docs", "/docs/page"));
        assert!(!RobotAnalyzer::match_tdm_pattern("/docs$", "/docs/page"));
    }

    #[test]
    fn test_tdm_pattern_wildcard() {
        assert!(RobotAnalyzer::match_tdm_pattern("/docs/*", "/docs/page"));
        assert!(RobotAnalyzer::match_tdm_pattern(
            "/docs/*",
            "/docs/page/sub"
        ));
        assert!(RobotAnalyzer::match_tdm_pattern("*.pdf", "/file.pdf"));
        assert!(RobotAnalyzer::match_tdm_pattern("*.pdf", "/docs/file.pdf"));
        assert!(!RobotAnalyzer::match_tdm_pattern("/docs/*", "/other/page"));
    }

    #[test]
    fn test_tdm_pattern_end_marker() {
        assert!(RobotAnalyzer::match_tdm_pattern("/docs$", "/docs"));
        assert!(!RobotAnalyzer::match_tdm_pattern("/docs$", "/docs/"));
        assert!(!RobotAnalyzer::match_tdm_pattern("/docs$", "/docs/page"));
        assert!(RobotAnalyzer::match_tdm_pattern(
            "/docs/page$",
            "/docs/page"
        ));
    }

    #[test]
    fn test_tdm_pattern_complex() {
        assert!(RobotAnalyzer::match_tdm_pattern(
            "/*/public/*",
            "/docs/public/file"
        ));
        assert!(RobotAnalyzer::match_tdm_pattern(
            "/docs/*.pdf$",
            "/docs/file.pdf"
        ));
        assert!(!RobotAnalyzer::match_tdm_pattern(
            "/docs/*.pdf$",
            "/docs/file.pdf.bak"
        ));
    }

    #[tokio::test]
    async fn test_tdm_rule_matching() {
        let analyzer = RobotAnalyzer::new("*".to_string());

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
        let policy = analyzer
            .evaluate_tdm_policy("https://example.com/", rules.clone())
            .await;
        assert!(policy.is_some());
        let policy = policy.unwrap();
        assert!(policy.is_reserved);
        assert_eq!(policy.matched_rule.as_ref().unwrap().location, "/");

        // Test public path matches second rule
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

        let policy2 = analyzer
            .evaluate_tdm_policy("https://example.com/public/data", rules2)
            .await;
        assert!(policy2.is_some());
        let policy2 = policy2.unwrap();
        assert!(!policy2.is_reserved);
        assert_eq!(policy2.matched_rule.as_ref().unwrap().location, "/public/*");
    }

    #[test]
    fn test_analyze_ai_bots_all_blocked() {
        let analyzer = RobotAnalyzer::new("*".to_string());
        let content = "User-agent: *\nDisallow: /\n";
        let results = analyzer.analyze_ai_bots(content, "https://www.nytimes.com/");
        assert_eq!(results.len(), 26);
        // Wildcard disallow blocks all bots via Robot parsing, but only bots
        // that are "mentioned" get blocked status. Unmentioned bots default to Allowed.
        // With User-agent: *, all bots match via wildcard in texting_robots.
    }

    #[test]
    fn test_analyze_ai_bots_all_allowed() {
        let analyzer = RobotAnalyzer::new("*".to_string());
        let content = "User-agent: *\nAllow: /\n";
        let results = analyzer.analyze_ai_bots(content, "https://github.com/");
        assert_eq!(results.len(), 26);
        for bot in &results {
            assert!(
                matches!(bot.status, BotStatus::Allowed),
                "{} should be allowed",
                bot.bot_name
            );
        }
    }

    #[test]
    fn test_analyze_ai_bots_returns_26() {
        let analyzer = RobotAnalyzer::new("*".to_string());
        let results = analyzer.analyze_ai_bots("", "https://github.com/");
        assert_eq!(results.len(), 26);
    }

    #[test]
    fn test_analyze_ai_bots_selective_blocking() {
        let analyzer = RobotAnalyzer::new("*".to_string());
        let content = "\
User-agent: GPTBot\nDisallow: /\n\n\
User-agent: ClaudeBot\nDisallow: /\n\n\
User-agent: *\nAllow: /\n";
        let results = analyzer.analyze_ai_bots(content, "https://techcrunch.com/");

        let gptbot = results.iter().find(|b| b.bot_name == "GPTBot").unwrap();
        assert!(matches!(gptbot.status, BotStatus::Blocked));

        let claudebot = results.iter().find(|b| b.bot_name == "ClaudeBot").unwrap();
        assert!(matches!(claudebot.status, BotStatus::Blocked));

        let perplexity = results
            .iter()
            .find(|b| b.bot_name == "PerplexityBot")
            .unwrap();
        assert!(matches!(perplexity.status, BotStatus::Allowed));
    }

    #[test]
    fn test_read_csv_url_column_detection() {
        let dir = tempfile::tempdir().unwrap();
        let csv_path = dir.path().join("test.csv");
        std::fs::write(
            &csv_path,
            "name,Company URL,notes\nNYT,https://www.nytimes.com,news\n",
        )
        .unwrap();
        let analyzer = RobotAnalyzer::new("*".to_string());
        let urls = analyzer.read_csv(&csv_path).unwrap();
        assert_eq!(urls, vec!["https://www.nytimes.com"]);
    }

    #[test]
    fn test_read_csv_adds_https_prefix() {
        let dir = tempfile::tempdir().unwrap();
        let csv_path = dir.path().join("test.csv");
        std::fs::write(&csv_path, "url\ngithub.com\n").unwrap();
        let analyzer = RobotAnalyzer::new("*".to_string());
        let urls = analyzer.read_csv(&csv_path).unwrap();
        assert_eq!(urls, vec!["https://github.com"]);
    }

    #[test]
    fn test_read_csv_skips_empty_rows() {
        let dir = tempfile::tempdir().unwrap();
        let csv_path = dir.path().join("test.csv");
        std::fs::write(
            &csv_path,
            "url\nhttps://github.com\n\n  \nhttps://www.nytimes.com\n",
        )
        .unwrap();
        let analyzer = RobotAnalyzer::new("*".to_string());
        let urls = analyzer.read_csv(&csv_path).unwrap();
        assert_eq!(urls, vec!["https://github.com", "https://www.nytimes.com"]);
    }
}
