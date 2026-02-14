use anyhow::{Context, Result};
use std::time::Duration;
use url::Url;

pub struct RobotFetcher {
    client: reqwest::Client,
}

impl RobotFetcher {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("robotxt/0.1.0 (robots.txt analyzer)")
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    /// Get the robots.txt URL for a given base URL
    pub fn get_robots_url(base_url: &str) -> Result<String> {
        // Use texting_robots helper if available, otherwise construct manually
        let parsed = Url::parse(base_url).context("Invalid URL")?;

        let robots_url = format!(
            "{}://{}/robots.txt",
            parsed.scheme(),
            parsed.host_str().context("URL has no host")?
        );

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
}

impl Default for RobotFetcher {
    fn default() -> Self {
        Self::new()
    }
}
