use crate::ai_crawlers::BotStatus;
use crate::checks::markdown_agents::MarkdownAgentsResult;
use crate::checks::robots_meta::RobotsMetaResult;
use crate::checks::well_known_oa::WellKnownOaResult;
use serde::{Deserialize, Serialize};

/// Input data for robots meta analysis (HTML body + X-Robots-Tag headers).
#[derive(Debug, Clone)]
pub struct RobotsMetaInput {
    pub html: String,
    pub x_robots_headers: Vec<String>,
}

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
pub struct BotAnalysisResult {
    pub bot_name: String,
    pub company: String,
    pub category: String,
    pub status: BotStatus,
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
    pub content_signal_search: Option<String>,
    pub content_signal_ai_input: Option<String>,
    pub content_signal_ai_train: Option<String>,
    pub tdm_policy: Option<TdmPolicy>,
    pub ai_bot_analysis: Vec<BotAnalysisResult>,
    pub robots_meta: Option<RobotsMetaResult>,
    pub markdown_agents: Option<MarkdownAgentsResult>,
    pub well_known_oa: Option<WellKnownOaResult>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnalysisStatus {
    Success,
    FetchError,
    ParseError,
}

impl AnalysisResult {
    pub fn error(url: String, error: String, status: AnalysisStatus) -> Self {
        Self {
            url,
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
            content_signal_search: None,
            content_signal_ai_input: None,
            content_signal_ai_train: None,
            tdm_policy: None,
            ai_bot_analysis: vec![],
            robots_meta: None,
            markdown_agents: None,
            well_known_oa: None,
            error: Some(error),
        }
    }
}
