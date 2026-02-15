use anyhow::{Context, Result};
use std::time::Duration;
use url::Url;

use crate::models::TdmRule;

#[derive(Clone)]
pub struct RobotFetcher {
    client: reqwest::Client,
}

impl RobotFetcher {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent(format!("Mozilla/5.0 (compatible; PolicyCheck/{}; +https://github.com/openattribution-org/policycheck)", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    /// Get the robots.txt URL for a given base URL
    pub fn get_robots_url(base_url: &str) -> Result<String> {
        let parsed = Url::parse(base_url).context("Invalid URL")?;
        let host = parsed.host_str().context("URL has no host")?;

        let robots_url = match parsed.port() {
            Some(port) => format!("{}://{}:{}/robots.txt", parsed.scheme(), host, port),
            None => format!("{}://{}/robots.txt", parsed.scheme(), host),
        };

        Ok(robots_url)
    }

    /// Fetch robots.txt content from a URL
    pub async fn fetch(&self, robots_url: &str) -> Result<String> {
        let response = self
            .client
            .get(robots_url)
            .send()
            .await
            .context("Failed to send request")?;

        if !response.status().is_success() {
            anyhow::bail!(
                "HTTP {} - {}",
                response.status(),
                response.status().canonical_reason().unwrap_or("Unknown")
            );
        }

        let content = response
            .text()
            .await
            .context("Failed to read response body")?;

        // Limit to 500KB as recommended by Google
        if content.len() > 512_000 {
            anyhow::bail!("robots.txt too large (>500KB)");
        }

        Ok(content)
    }

    /// Fetch robots.txt for a base URL (convenience method)
    pub async fn fetch_for_url(&self, base_url: &str) -> Result<(String, String)> {
        let robots_url = Self::get_robots_url(base_url)?;
        let content = self.fetch(&robots_url).await?;
        Ok((robots_url, content))
    }

    /// Get the TDM policy URL for a given base URL
    pub fn get_tdm_url(base_url: &str) -> Result<String> {
        let parsed = Url::parse(base_url).context("Invalid URL")?;
        let host = parsed.host_str().context("URL has no host")?;

        let tdm_url = match parsed.port() {
            Some(port) => format!(
                "{}://{}:{}/.well-known/tdmrep.json",
                parsed.scheme(),
                host,
                port
            ),
            None => format!("{}://{}/.well-known/tdmrep.json", parsed.scheme(), host),
        };

        Ok(tdm_url)
    }

    /// Fetch and parse TDM policy from /.well-known/tdmrep.json
    pub async fn fetch_tdm_policy(&self, base_url: &str) -> Result<Vec<TdmRule>> {
        let tdm_url = Self::get_tdm_url(base_url)?;

        let response = self
            .client
            .get(&tdm_url)
            .send()
            .await
            .context("Failed to send TDM request")?;

        if !response.status().is_success() {
            anyhow::bail!(
                "TDM HTTP {} - {}",
                response.status(),
                response.status().canonical_reason().unwrap_or("Unknown")
            );
        }

        let content = response
            .text()
            .await
            .context("Failed to read TDM response body")?;

        // Parse JSON array of TDM rules
        let rules: Vec<TdmRule> =
            serde_json::from_str(&content).context("Failed to parse tdmrep.json")?;

        Ok(rules)
    }
}

impl Default for RobotFetcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_robots_url_standard() {
        let url = RobotFetcher::get_robots_url("https://www.nytimes.com").unwrap();
        assert_eq!(url, "https://www.nytimes.com/robots.txt");
    }

    #[test]
    fn test_get_robots_url_with_port() {
        let url = RobotFetcher::get_robots_url("https://localhost:8080/page").unwrap();
        assert_eq!(url, "https://localhost:8080/robots.txt");
    }

    #[test]
    fn test_get_robots_url_http_scheme() {
        let url = RobotFetcher::get_robots_url("http://example.com/path").unwrap();
        assert_eq!(url, "http://example.com/robots.txt");
    }

    #[test]
    fn test_get_robots_url_with_path_stripped() {
        let url = RobotFetcher::get_robots_url("https://github.com/some/path").unwrap();
        assert_eq!(url, "https://github.com/robots.txt");
    }

    #[test]
    fn test_get_robots_url_invalid() {
        assert!(RobotFetcher::get_robots_url("not-a-url").is_err());
    }

    #[test]
    fn test_get_tdm_url_standard() {
        let url = RobotFetcher::get_tdm_url("https://www.nytimes.com").unwrap();
        assert_eq!(url, "https://www.nytimes.com/.well-known/tdmrep.json");
    }

    #[test]
    fn test_get_tdm_url_with_port() {
        let url = RobotFetcher::get_tdm_url("https://localhost:3000/page").unwrap();
        assert_eq!(url, "https://localhost:3000/.well-known/tdmrep.json");
    }

    #[test]
    fn test_get_tdm_url_invalid() {
        assert!(RobotFetcher::get_tdm_url("not-a-url").is_err());
    }
}
