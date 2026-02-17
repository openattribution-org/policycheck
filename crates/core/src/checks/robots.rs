//! Robots Exclusion Protocol (REP/RFC 9309) parsing.
//!
//! Extracts user agents, allowed/disallowed paths, crawl delay,
//! and sitemaps from robots.txt content.

use texting_robots::Robot;

/// Parsed robots.txt data for a specific user agent.
pub struct RobotsResult {
    pub user_agents: Vec<String>,
    pub allowed_paths: Vec<String>,
    pub disallowed_paths: Vec<String>,
    pub is_path_allowed: bool,
    pub crawl_delay: Option<f64>,
    pub sitemaps: Vec<String>,
}

/// Parse robots.txt content and check path access for the given user agent.
pub fn analyze(content: &str, user_agent: &str, url: &str) -> RobotsResult {
    let user_agents = extract_user_agents(content);
    let (allowed_paths, disallowed_paths) = extract_paths(content);

    let (is_path_allowed, crawl_delay, sitemaps) = match Robot::new(user_agent, content.as_bytes())
    {
        Ok(robot) => (
            robot.allowed(url),
            robot.delay.map(|d| d as f64),
            robot.sitemaps.clone(),
        ),
        Err(_) => (false, None, vec![]),
    };

    RobotsResult {
        user_agents,
        allowed_paths,
        disallowed_paths,
        is_path_allowed,
        crawl_delay,
        sitemaps,
    }
}

/// Extract all User-agent directives from robots.txt content.
#[allow(clippy::collapsible_if)]
fn extract_user_agents(content: &str) -> Vec<String> {
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

/// Extract Allow and Disallow paths from robots.txt content.
#[allow(clippy::collapsible_if)]
fn extract_paths(content: &str) -> (Vec<String>, Vec<String>) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_user_agents() {
        let content = "User-agent: *\nUser-agent: GoogleBot\nUser-agent: BingBot";
        let agents = extract_user_agents(content);

        assert_eq!(agents.len(), 3);
        assert!(agents.contains(&"*".to_string()));
        assert!(agents.contains(&"GoogleBot".to_string()));
        assert!(agents.contains(&"BingBot".to_string()));
    }

    #[test]
    fn test_extract_paths() {
        let content = "Allow: /public\nDisallow: /private\nDisallow: /admin";
        let (allowed, disallowed) = extract_paths(content);

        assert_eq!(allowed.len(), 1);
        assert_eq!(allowed[0], "/public");
        assert_eq!(disallowed.len(), 2);
        assert!(disallowed.contains(&"/private".to_string()));
        assert!(disallowed.contains(&"/admin".to_string()));
    }

    #[test]
    fn test_analyze_allowed_path() {
        let content = "User-agent: *\nAllow: /\n";
        let result = analyze(content, "*", "https://example.com/page");
        assert!(result.is_path_allowed);
    }

    #[test]
    fn test_analyze_disallowed_path() {
        let content = "User-agent: *\nDisallow: /\n";
        let result = analyze(content, "*", "https://example.com/page");
        assert!(!result.is_path_allowed);
    }
}
