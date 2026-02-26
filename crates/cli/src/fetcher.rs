use anyhow::{Context, Result};
use policycheck_core::checks::markdown_agents::MarkdownProbeData;
use policycheck_core::models::TdmRule;
use std::time::Duration;
use url::Url;

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

    /// Fetch a page's HTML and X-Robots-Tag headers for robots meta analysis.
    ///
    /// Returns (html_body, x_robots_tag_headers). Body is limited to 256KB.
    pub async fn fetch_page_meta(&self, url: &str) -> Result<(String, Vec<String>)> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .context("Failed to fetch page for robots meta")?;

        if !response.status().is_success() {
            anyhow::bail!(
                "Page HTTP {} - {}",
                response.status(),
                response.status().canonical_reason().unwrap_or("Unknown")
            );
        }

        // Collect X-Robots-Tag headers
        let x_robots_headers: Vec<String> = response
            .headers()
            .get_all("x-robots-tag")
            .iter()
            .filter_map(|v| v.to_str().ok())
            .map(|s| s.to_string())
            .collect();

        let body = response.text().await.context("Failed to read page body")?;

        // Limit to 256KB
        let html = if body.len() > 262_144 {
            body[..262_144].to_string()
        } else {
            body
        };

        Ok((html, x_robots_headers))
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

        let rules: Vec<TdmRule> =
            serde_json::from_str(&content).context("Failed to parse tdmrep.json")?;

        Ok(rules)
    }

    /// Probe a URL for Cloudflare "Markdown for Agents" support.
    ///
    /// Sends a HEAD request with `Accept: text/markdown` and extracts
    /// `Content-Type`, `x-markdown-tokens`, and `Content-Signal` headers.
    /// Returns `None` on any network or parsing failure.
    pub async fn fetch_markdown_probe(&self, base_url: &str) -> Option<MarkdownProbeData> {
        let markdown_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .user_agent(format!(
                "Mozilla/5.0 (compatible; PolicyCheck/{}; +https://github.com/openattribution-org/policycheck)",
                env!("CARGO_PKG_VERSION")
            ))
            .build()
            .ok()?;

        let response = markdown_client
            .head(base_url)
            .header("Accept", "text/markdown")
            .send()
            .await
            .ok()?;

        let status_code = response.status().as_u16();

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .map(String::from);

        let markdown_tokens = response
            .headers()
            .get("x-markdown-tokens")
            .and_then(|v| v.to_str().ok())
            .map(String::from);

        let content_signal = response
            .headers()
            .get("content-signal")
            .and_then(|v| v.to_str().ok())
            .map(String::from);

        Some(MarkdownProbeData {
            status_code,
            content_type,
            markdown_tokens,
            content_signal,
        })
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
