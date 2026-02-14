use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TdmRule {
    pub location: String,
    #[serde(rename = "tdm-reservation")]
    pub tdm_reservation: u8, // 0 = unreserved, 1 = reserved
    #[serde(rename = "tdm-policy")]
    pub tdm_policy: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TdmPolicy {
    pub rules: Vec<TdmRule>,
    pub matched_rule: Option<TdmRule>,
    pub is_reserved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub url: String,
    pub robots_url: String,
    pub status: AnalysisStatus,
    pub user_agents: Vec<String>,
    pub crawl_delay: Option<f64>,
    pub sitemaps: Vec<String>,
    pub allowed_paths: Vec<String>,
    pub disallowed_paths: Vec<String>,
    pub is_path_allowed: bool,
    pub global_licenses: Vec<String>,
    pub group_licenses: Vec<String>,
    pub active_licenses: Vec<String>,
    pub tdm_policy: Option<TdmPolicy>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnalysisStatus {
    Success,
    FetchError,
    ParseError,
    InvalidUrl,
}

impl AnalysisResult {
    pub fn error(url: String, error: String, status: AnalysisStatus) -> Self {
        Self {
            url: url.clone(),
            robots_url: String::new(),
            status,
            user_agents: vec![],
            crawl_delay: None,
            sitemaps: vec![],
            allowed_paths: vec![],
            disallowed_paths: vec![],
            is_path_allowed: false,
            global_licenses: vec![],
            group_licenses: vec![],
            active_licenses: vec![],
            tdm_policy: None,
            error: Some(error),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AnalyzeRequest {
    pub urls: Vec<String>,
    #[serde(default = "default_user_agent")]
    pub user_agent: String,
}

fn default_user_agent() -> String {
    "*".to_string()
}

#[derive(Debug, Serialize)]
pub struct AnalyzeResponse {
    pub results: Vec<AnalysisResult>,
    pub total: usize,
    pub successful: usize,
    pub failed: usize,
}
