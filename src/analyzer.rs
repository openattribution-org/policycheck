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
            let user_agent = self.user_agent.clone();

            let handle = tokio::spawn(async move {
                let analyzer = RobotAnalyzer::new(user_agent);
                analyzer.analyze_url(&url).await
            });

            handles.push(handle);
        }

        let mut results = vec![];
        for handle in handles {
            if let Ok(result) = handle.await {
                results.push(result);
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

        // Find the first matching rule (most specific wins)
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
                // Create a Robot instance specifically for this bot
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
}
