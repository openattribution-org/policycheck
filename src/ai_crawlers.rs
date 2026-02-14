use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AICrawler {
    pub name: String,
    pub company: String,
    pub category: CrawlerCategory,
    pub purpose: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CrawlerCategory {
    Training,
    Search,
    UserTriggered,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BotStatus {
    Blocked,  // Disallowed in robots.txt
    Allowed,  // Allowed or not mentioned in robots.txt (can access per robots.txt)
}

impl AICrawler {
    /// Returns the canonical list of known AI crawlers
    pub fn get_all() -> Vec<AICrawler> {
        vec![
            // Training Crawlers
            AICrawler {
                name: "GPTBot".to_string(),
                company: "OpenAI".to_string(),
                category: CrawlerCategory::Training,
                purpose: "ChatGPT model training".to_string(),
            },
            AICrawler {
                name: "ClaudeBot".to_string(),
                company: "Anthropic".to_string(),
                category: CrawlerCategory::Training,
                purpose: "Claude model training".to_string(),
            },
            AICrawler {
                name: "anthropic-ai".to_string(),
                company: "Anthropic".to_string(),
                category: CrawlerCategory::Training,
                purpose: "Bulk model training".to_string(),
            },
            AICrawler {
                name: "Claude-Web".to_string(),
                company: "Anthropic".to_string(),
                category: CrawlerCategory::Training,
                purpose: "Web-focused training".to_string(),
            },
            AICrawler {
                name: "Google-Extended".to_string(),
                company: "Google".to_string(),
                category: CrawlerCategory::Training,
                purpose: "Gemini AI training".to_string(),
            },
            AICrawler {
                name: "GoogleOther".to_string(),
                company: "Google".to_string(),
                category: CrawlerCategory::Training,
                purpose: "Research & development".to_string(),
            },
            AICrawler {
                name: "Meta-ExternalAgent".to_string(),
                company: "Meta".to_string(),
                category: CrawlerCategory::Training,
                purpose: "AI model training".to_string(),
            },
            AICrawler {
                name: "FacebookBot".to_string(),
                company: "Meta".to_string(),
                category: CrawlerCategory::Training,
                purpose: "Speech recognition training".to_string(),
            },
            AICrawler {
                name: "Applebot-Extended".to_string(),
                company: "Apple".to_string(),
                category: CrawlerCategory::Training,
                purpose: "Generative AI training".to_string(),
            },
            AICrawler {
                name: "Amazonbot".to_string(),
                company: "Amazon".to_string(),
                category: CrawlerCategory::Training,
                purpose: "AI improvement, model training".to_string(),
            },
            AICrawler {
                name: "CCBot".to_string(),
                company: "Common Crawl".to_string(),
                category: CrawlerCategory::Training,
                purpose: "Open dataset collection".to_string(),
            },
            AICrawler {
                name: "Bytespider".to_string(),
                company: "ByteDance".to_string(),
                category: CrawlerCategory::Training,
                purpose: "AI training".to_string(),
            },
            AICrawler {
                name: "cohere-ai".to_string(),
                company: "Cohere".to_string(),
                category: CrawlerCategory::Training,
                purpose: "LLM training".to_string(),
            },
            AICrawler {
                name: "Diffbot".to_string(),
                company: "Diffbot".to_string(),
                category: CrawlerCategory::Training,
                purpose: "AI data extraction".to_string(),
            },
            AICrawler {
                name: "Omgilibot".to_string(),
                company: "Webz.io".to_string(),
                category: CrawlerCategory::Training,
                purpose: "Data collection for resale".to_string(),
            },
            AICrawler {
                name: "ImagesiftBot".to_string(),
                company: "The Hive".to_string(),
                category: CrawlerCategory::Training,
                purpose: "Image model training".to_string(),
            },
            // Search Crawlers
            AICrawler {
                name: "OAI-SearchBot".to_string(),
                company: "OpenAI".to_string(),
                category: CrawlerCategory::Search,
                purpose: "ChatGPT search indexing".to_string(),
            },
            AICrawler {
                name: "PerplexityBot".to_string(),
                company: "Perplexity".to_string(),
                category: CrawlerCategory::Search,
                purpose: "Search indexing".to_string(),
            },
            AICrawler {
                name: "YouBot".to_string(),
                company: "You.com".to_string(),
                category: CrawlerCategory::Search,
                purpose: "AI search".to_string(),
            },
            AICrawler {
                name: "DuckAssistBot".to_string(),
                company: "DuckDuckGo".to_string(),
                category: CrawlerCategory::Search,
                purpose: "AI-assisted answers".to_string(),
            },
            // User-Triggered
            AICrawler {
                name: "ChatGPT-User".to_string(),
                company: "OpenAI".to_string(),
                category: CrawlerCategory::UserTriggered,
                purpose: "User-requested fetching".to_string(),
            },
            AICrawler {
                name: "Perplexity-User".to_string(),
                company: "Perplexity".to_string(),
                category: CrawlerCategory::UserTriggered,
                purpose: "User-requested fetching".to_string(),
            },
            AICrawler {
                name: "Meta-ExternalFetcher".to_string(),
                company: "Meta".to_string(),
                category: CrawlerCategory::UserTriggered,
                purpose: "Real-time content fetching".to_string(),
            },
            // Other
            AICrawler {
                name: "Applebot".to_string(),
                company: "Apple".to_string(),
                category: CrawlerCategory::Other,
                purpose: "Siri, Spotlight, Safari".to_string(),
            },
            AICrawler {
                name: "Google-CloudVertexBot".to_string(),
                company: "Google".to_string(),
                category: CrawlerCategory::Other,
                purpose: "Cloud AI services".to_string(),
            },
            AICrawler {
                name: "Amzn-SearchBot".to_string(),
                company: "Amazon".to_string(),
                category: CrawlerCategory::Other,
                purpose: "Alexa and Rufus search".to_string(),
            },
        ]
    }

    /// Get the "major" AI bots that advertisers care about most
    pub fn get_major_bots() -> Vec<AICrawler> {
        let all = Self::get_all();
        let major_names = [
            "GPTBot",
            "ClaudeBot",
            "Google-Extended",
            "OAI-SearchBot",
            "PerplexityBot",
            "CCBot",
            "Bytespider",
            "Meta-ExternalAgent",
        ];

        all.into_iter()
            .filter(|bot| major_names.contains(&bot.name.as_str()))
            .collect()
    }
}
