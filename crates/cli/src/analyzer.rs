use crate::fetcher::RobotFetcher;
use anyhow::Result;
use policycheck_core::models::{AnalysisResult, AnalysisStatus, RobotsMetaInput};
use policycheck_core::PolicyAnalyzer;

/// Network-aware analyzer that wraps the core `PolicyAnalyzer` with HTTP fetching.
pub struct RobotAnalyzer {
    core: PolicyAnalyzer,
    fetcher: RobotFetcher,
    check_robots_meta: bool,
}

impl RobotAnalyzer {
    pub fn new(user_agent: String) -> Self {
        Self {
            core: PolicyAnalyzer::new(user_agent),
            fetcher: RobotFetcher::new(),
            check_robots_meta: false,
        }
    }

    pub fn with_fetcher(user_agent: String, fetcher: RobotFetcher) -> Self {
        Self {
            core: PolicyAnalyzer::new(user_agent),
            fetcher,
            check_robots_meta: false,
        }
    }

    pub fn with_robots_meta(mut self, enabled: bool) -> Self {
        self.check_robots_meta = enabled;
        self
    }

    /// Read URLs from a CSV file.
    ///
    /// Detects the URL column by header name (url, link, website) and adds
    /// an `https://` prefix to bare domains.
    pub fn read_csv(&self, path: &std::path::Path) -> Result<Vec<String>> {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_path(path)?;

        let mut urls = Vec::new();

        let headers = reader.headers()?.clone();

        let url_col_idx = headers
            .iter()
            .position(|h| {
                let h_lower = h.to_lowercase();
                h_lower.contains("url") || h_lower == "link" || h_lower == "website"
            })
            .unwrap_or(0);

        for result in reader.records() {
            let record = result?;

            if let Some(url) = record.get(url_col_idx) {
                let url = url.trim();
                if !url.is_empty() {
                    let url = if url.starts_with("http://") || url.starts_with("https://") {
                        url.to_string()
                    } else {
                        format!("https://{}", url)
                    };
                    urls.push(url);
                }
            }
        }

        Ok(urls)
    }

    /// Fetch and analyze a single URL.
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

        // Fetch optional checks concurrently
        let (tdm_rules, markdown_probe, well_known_oa) = tokio::join!(
            self.fetcher.fetch_tdm_policy(url),
            self.fetcher.fetch_markdown_probe(url),
            self.fetcher.fetch_well_known_oa(url),
        );

        // Fetch page-level robots meta (optional — don't fail if missing)
        let robots_meta_input = if self.check_robots_meta {
            self.fetcher
                .fetch_page_meta(url)
                .await
                .ok()
                .map(|(html, headers)| RobotsMetaInput {
                    html,
                    x_robots_headers: headers,
                })
        } else {
            None
        };

        // Delegate to core analyzer
        let mut result = self.core.analyze(
            url,
            &content,
            tdm_rules.ok(),
            robots_meta_input,
            markdown_probe,
            well_known_oa.as_deref(),
        );
        result.robots_url = robots_url;

        result
    }

    /// Analyze multiple URLs concurrently.
    pub async fn analyze_urls(&self, urls: &[String]) -> Vec<AnalysisResult> {
        let mut handles = vec![];

        for url in urls {
            let url = url.clone();
            let url_for_error = url.clone();
            let fetcher = self.fetcher.clone();
            let core_user_agent = self.core_user_agent();
            let check_robots_meta = self.check_robots_meta;

            let handle = tokio::spawn(async move {
                let analyzer = RobotAnalyzer::with_fetcher(core_user_agent, fetcher)
                    .with_robots_meta(check_robots_meta);
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

    fn core_user_agent(&self) -> String {
        self.core.user_agent().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_read_csv_bare_domain_adds_https_prefix() {
        let dir = tempfile::tempdir().unwrap();
        let csv_path = dir.path().join("test.csv");
        std::fs::write(&csv_path, "url\ngithub.com\n").unwrap();
        let analyzer = RobotAnalyzer::new("*".to_string());
        let urls = analyzer.read_csv(&csv_path).unwrap();
        assert_eq!(urls, vec!["https://github.com"]);
    }

    #[test]
    fn test_read_csv_empty_rows_skipped() {
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
