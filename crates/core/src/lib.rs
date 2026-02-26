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

use checks::markdown_agents::MarkdownProbeData;
use models::{AnalysisResult, AnalysisStatus, RobotsMetaInput, TdmRule};

/// Core policy analyzer. Takes raw content (no fetching) and produces analysis results.
///
/// ```
/// use policycheck_core::PolicyAnalyzer;
///
/// let analyzer = PolicyAnalyzer::new("GPTBot".to_string());
/// let result = analyzer.analyze("https://example.com", "User-agent: *\nDisallow: /\n", None, None, None);
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
    ///
    /// `robots_meta_input` is optional — pass fetched HTML + X-Robots-Tag headers
    /// to evaluate page-level robots directives, or `None` to skip.
    ///
    /// `markdown_probe` is optional — pass pre-fetched Markdown for Agents probe
    /// data if available, or `None` to skip that check.
    pub fn analyze(
        &self,
        url: &str,
        robots_txt: &str,
        tdm_rules: Option<Vec<TdmRule>>,
        robots_meta_input: Option<RobotsMetaInput>,
        markdown_probe: Option<MarkdownProbeData>,
    ) -> AnalysisResult {
        // Run each compliance check module
        let robots = checks::robots::analyze(robots_txt, &self.user_agent, url);
        let rsl = checks::rsl::extract(robots_txt, &self.user_agent);
        let signals = checks::content_signals::extract(robots_txt, &self.user_agent);
        let tdm_policy = tdm_rules.and_then(|rules| checks::tdm::evaluate(url, rules));
        let ai_bot_analysis = checks::ai_bots::analyze(robots_txt, url);
        let markdown_agents =
            markdown_probe.map(|probe| checks::markdown_agents::evaluate(&probe));

        let robots_meta = robots_meta_input.map(|input| {
            checks::robots_meta::analyze(&input.html, &input.x_robots_headers, &self.user_agent)
        });

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
            robots_meta,
            markdown_agents,
            error: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use checks::markdown_agents::MarkdownProbeData;

    #[test]
    fn test_analyze_basic() {
        let analyzer = PolicyAnalyzer::new("*".to_string());
        let result = analyzer.analyze(
            "https://example.com",
            "User-agent: *\nAllow: /\n",
            None,
            None,
            None,
        );

        assert!(matches!(result.status, AnalysisStatus::Success));
        assert!(result.is_path_allowed);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_analyze_blocked() {
        let analyzer = PolicyAnalyzer::new("GPTBot".to_string());
        let content = "User-agent: GPTBot\nDisallow: /\n";
        let result = analyzer.analyze("https://example.com", content, None, None, None);

        assert!(!result.is_path_allowed);
    }

    #[test]
    fn test_analyze_with_rsl() {
        let analyzer = PolicyAnalyzer::new("*".to_string());
        let content = "License: https://example.com/license.xml\nUser-agent: *\nAllow: /\n";
        let result = analyzer.analyze("https://example.com", content, None, None, None);

        assert_eq!(result.global_licenses.len(), 1);
        assert_eq!(result.active_licenses.len(), 1);
    }

    #[test]
    fn test_analyze_with_content_signals() {
        let analyzer = PolicyAnalyzer::new("*".to_string());
        let content = "User-agent: *\nContent-Signal: search=yes, ai-train=no\nAllow: /\n";
        let result = analyzer.analyze("https://example.com", content, None, None, None);

        assert_eq!(result.content_signal_search, Some("yes".to_string()));
        assert_eq!(result.content_signal_ai_train, Some("no".to_string()));
    }

    #[test]
    fn test_analyze_empty_robots_txt_returns_success_with_path_allowed() {
        // GIVEN empty robots.txt content (site has no restrictions)
        let analyzer = PolicyAnalyzer::new("GPTBot".to_string());

        // WHEN we analyze
        let result = analyzer.analyze("https://www.nytimes.com", "", None, None, None);

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
            None,
            None,
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
        let result = analyzer.analyze(
            "https://www.nytimes.com/search?q=test",
            content,
            None,
            None,
            None,
        );

        // SHOULD be disallowed (path starts with /search)
        assert!(!result.is_path_allowed);
    }

    #[test]
    fn test_analyze_with_markdown_probe_supported() {
        // GIVEN robots.txt and a markdown probe indicating support
        let analyzer = PolicyAnalyzer::new("*".to_string());
        let probe = MarkdownProbeData {
            status_code: 200,
            content_type: Some("text/markdown; charset=utf-8".to_string()),
            markdown_tokens: Some("3500".to_string()),
            content_signal: Some("search=allow, ai-input=allow, ai-train=disallow".to_string()),
        };

        // WHEN we analyze with the probe
        let result = analyzer.analyze(
            "https://example.com",
            "User-agent: *\nAllow: /\n",
            None,
            None,
            Some(probe),
        );

        // SHOULD include markdown agents result
        let md = result.markdown_agents.unwrap();
        assert!(md.supported);
        assert_eq!(md.token_count, Some(3500));
        assert_eq!(md.http_content_signal_search, Some("allow".to_string()));
        assert_eq!(md.http_content_signal_ai_input, Some("allow".to_string()));
        assert_eq!(
            md.http_content_signal_ai_train,
            Some("disallow".to_string())
        );
    }

    #[test]
    fn test_analyze_without_markdown_probe_returns_none() {
        // GIVEN no markdown probe data
        let analyzer = PolicyAnalyzer::new("*".to_string());

        // WHEN we analyze without probe
        let result = analyzer.analyze(
            "https://example.com",
            "User-agent: *\nAllow: /\n",
            None,
            None,
            None,
        );

        // SHOULD have no markdown agents result
        assert!(result.markdown_agents.is_none());
    }
}
