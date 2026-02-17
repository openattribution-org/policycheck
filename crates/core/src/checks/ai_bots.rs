//! AI bot access analysis.
//!
//! Checks the access status of 26 known AI crawlers against robots.txt content.
//! Each bot is individually parsed to get accurate per-bot allow/disallow results.

use crate::ai_crawlers::{AICrawler, BotStatus};
use crate::models::BotAnalysisResult;
use texting_robots::Robot;

/// Analyze each AI bot individually to determine its access status.
///
/// Re-parses robots.txt for each mentioned bot to get accurate per-bot results
/// (texting_robots bakes the user-agent into its parsed state).
pub fn analyze(content: &str, url: &str) -> Vec<BotAnalysisResult> {
    let all_bots = AICrawler::get_all();
    let mut results = Vec::new();

    // Normalize user agents for comparison (case-insensitive)
    let user_agents_lower: Vec<String> = extract_user_agents_lower(content);

    // Check if a wildcard User-agent: * rule exists (applies to all bots)
    let has_wildcard = user_agents_lower.iter().any(|ua| ua == "*");

    for bot in all_bots {
        let bot_name_lower = bot.name.to_lowercase();

        // A bot is affected by robots.txt if it's explicitly named or a wildcard rule exists
        let is_mentioned = has_wildcard
            || user_agents_lower
                .iter()
                .any(|ua| ua == &bot_name_lower || ua.contains(&bot_name_lower));

        let status = if is_mentioned {
            // Bot is mentioned (or covered by wildcard) — check actual access
            match Robot::new(&bot.name, content.as_bytes()) {
                Ok(robot) => {
                    if robot.allowed(url) {
                        BotStatus::Allowed
                    } else {
                        BotStatus::Blocked
                    }
                }
                Err(_) => BotStatus::Allowed,
            }
        } else {
            // Bot not mentioned and no wildcard — allowed by default
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

/// Extract user agents from content, lowercased for comparison.
fn extract_user_agents_lower(content: &str) -> Vec<String> {
    let mut agents = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.to_lowercase().starts_with("user-agent:") {
            if let Some(agent) = line.split(':').nth(1) {
                let agent = agent.trim().to_lowercase();
                if !agents.contains(&agent) {
                    agents.push(agent);
                }
            }
        }
    }

    agents
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_ai_bots_all_allowed() {
        let content = "User-agent: *\nAllow: /\n";
        let results = analyze(content, "https://github.com/");
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
        let results = analyze("", "https://github.com/");
        assert_eq!(results.len(), 26);
    }

    #[test]
    fn test_analyze_ai_bots_selective_blocking() {
        let content = "\
User-agent: GPTBot\nDisallow: /\n\n\
User-agent: ClaudeBot\nDisallow: /\n\n\
User-agent: *\nAllow: /\n";
        let results = analyze(content, "https://techcrunch.com/");

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
    fn test_wildcard_disallow_blocks_all_ai_bots() {
        let content = "User-agent: *\nDisallow: /\n";
        let results = analyze(content, "https://www.nytimes.com/");
        assert_eq!(results.len(), 26);
        for bot in &results {
            assert!(
                matches!(bot.status, BotStatus::Blocked),
                "{} should be blocked by wildcard Disallow: /",
                bot.bot_name
            );
        }
    }
}
