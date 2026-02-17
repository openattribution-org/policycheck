//! PolicyCheck core library.
//!
//! Pure parsing and analysis logic for web compliance checking.
//! No network I/O — callers provide raw content, this library parses it.
//!
//! ## Architecture
//!
//! Each compliance standard lives in its own module under `checks/`:
//! - `checks::robots` — Robots Exclusion Protocol (RFC 9309)
//! - `checks::rsl` — Responsible Sourcing License
//! - `checks::content_signals` — Cloudflare Content Signals
//! - `checks::tdm` — W3C Text & Data Mining Reservation Protocol
//! - `checks::ai_bots` — AI crawler access analysis
//!
//! The `PolicyAnalyzer` orchestrates all checks into a unified `AnalysisResult`.

pub mod ai_crawlers;
pub mod checks;
pub mod models;

use models::{AnalysisResult, AnalysisStatus, TdmRule};

/// Core policy analyzer. Takes raw content (no fetching) and produces analysis results.
///
/// ```
/// use policycheck_core::PolicyAnalyzer;
///
/// let analyzer = PolicyAnalyzer::new("GPTBot".to_string());
/// let result = analyzer.analyze("https://example.com", "User-agent: *\nDisallow: /\n", None);
/// assert!(!result.is_path_allowed);
/// ```
pub struct PolicyAnalyzer {
    user_agent: String,
}

impl PolicyAnalyzer {
    pub fn new(user_agent: String) -> Self {
        Self { user_agent }
    }

    /// Get the configured user agent.
    pub fn user_agent(&self) -> &str {
        &self.user_agent
    }

    /// Analyze raw robots.txt content for a given URL.
    ///
    /// `tdm_rules` is optional — pass pre-fetched `/.well-known/tdmrep.json` data
    /// if available, or `None` to skip TDM evaluation.
    pub fn analyze(
        &self,
        url: &str,
        robots_txt: &str,
        tdm_rules: Option<Vec<TdmRule>>,
    ) -> AnalysisResult {
        // Run each compliance check module
        let robots = checks::robots::analyze(robots_txt, &self.user_agent, url);
        let rsl = checks::rsl::extract(robots_txt, &self.user_agent);
        let signals = checks::content_signals::extract(robots_txt, &self.user_agent);
        let tdm_policy = tdm_rules.and_then(|rules| checks::tdm::evaluate(url, rules));
        let ai_bot_analysis = checks::ai_bots::analyze(robots_txt, url);

        AnalysisResult {
            url: url.to_string(),
            robots_url: String::new(), // Caller sets this (they know the actual URL fetched)
            status: AnalysisStatus::Success,
            user_agents: robots.user_agents,
            crawl_delay: robots.crawl_delay,
            sitemaps: robots.sitemaps,
            allowed_paths: robots.allowed_paths,
            disallowed_paths: robots.disallowed_paths,
            is_path_allowed: robots.is_path_allowed,
            global_licenses: rsl.global_licenses,
            group_licenses: rsl.group_licenses,
            active_licenses: rsl.active_licenses,
            content_signal_search: signals.search,
            content_signal_ai_input: signals.ai_input,
            content_signal_ai_train: signals.ai_train,
            tdm_policy,
            ai_bot_analysis,
            error: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_basic() {
        let analyzer = PolicyAnalyzer::new("*".to_string());
        let result = analyzer.analyze("https://example.com", "User-agent: *\nAllow: /\n", None);

        assert!(matches!(result.status, AnalysisStatus::Success));
        assert!(result.is_path_allowed);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_analyze_blocked() {
        let analyzer = PolicyAnalyzer::new("GPTBot".to_string());
        let content = "User-agent: GPTBot\nDisallow: /\n";
        let result = analyzer.analyze("https://example.com", content, None);

        assert!(!result.is_path_allowed);
    }

    #[test]
    fn test_analyze_with_rsl() {
        let analyzer = PolicyAnalyzer::new("*".to_string());
        let content = "License: https://example.com/license.xml\nUser-agent: *\nAllow: /\n";
        let result = analyzer.analyze("https://example.com", content, None);

        assert_eq!(result.global_licenses.len(), 1);
        assert_eq!(result.active_licenses.len(), 1);
    }

    #[test]
    fn test_analyze_with_content_signals() {
        let analyzer = PolicyAnalyzer::new("*".to_string());
        let content = "User-agent: *\nContent-Signal: search=yes, ai-train=no\nAllow: /\n";
        let result = analyzer.analyze("https://example.com", content, None);

        assert_eq!(result.content_signal_search, Some("yes".to_string()));
        assert_eq!(result.content_signal_ai_train, Some("no".to_string()));
    }

    #[test]
    fn test_analyze_empty_robots_txt_returns_success_with_path_allowed() {
        // GIVEN empty robots.txt content (site has no restrictions)
        let analyzer = PolicyAnalyzer::new("GPTBot".to_string());

        // WHEN we analyze
        let result = analyzer.analyze("https://www.nytimes.com", "", None);

        // SHOULD succeed with path allowed (empty robots.txt = no restrictions)
        assert!(matches!(result.status, AnalysisStatus::Success));
        assert!(result.is_path_allowed);
        assert!(result.user_agents.is_empty());
        assert!(result.error.is_none());
    }

    #[test]
    fn test_analyze_with_tdm_rules_sets_reservation_status() {
        // GIVEN robots.txt allowing access and TDM rules reserving all content
        let analyzer = PolicyAnalyzer::new("*".to_string());
        let tdm_rules = vec![models::TdmRule {
            location: "/".to_string(),
            tdm_reservation: 1,
            tdm_policy: Some("https://www.nytimes.com/tdm-policy".to_string()),
        }];

        // WHEN we analyze with TDM rules
        let result = analyzer.analyze(
            "https://www.nytimes.com/article",
            "User-agent: *\nAllow: /\n",
            Some(tdm_rules),
        );

        // SHOULD report TDM reserved
        let tdm = result.tdm_policy.unwrap();
        assert!(tdm.is_reserved);
        assert_eq!(tdm.matched_rule.unwrap().location, "/");
    }

    #[test]
    fn test_analyze_url_with_query_params_checks_path_correctly() {
        // GIVEN robots.txt blocking /search
        let analyzer = PolicyAnalyzer::new("*".to_string());
        let content = "User-agent: *\nDisallow: /search\n";

        // WHEN checking a URL with query params under /search
        let result = analyzer.analyze("https://www.nytimes.com/search?q=test", content, None);

        // SHOULD be disallowed (path starts with /search)
        assert!(!result.is_path_allowed);
    }
}
