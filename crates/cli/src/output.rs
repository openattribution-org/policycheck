use anyhow::Result;
use comfy_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, *};
use policycheck_core::ai_crawlers::{AICrawler, BotStatus};
use policycheck_core::models::{AnalysisResult, AnalysisStatus};

pub fn format_table(results: &[AnalysisResult]) -> Result<String> {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec![
            "URL",
            "Status",
            "Path Allowed",
            "RSL Licenses",
            "TDM Reserved",
            "Robots Meta",
            "Markdown",
            "AI Bots Summary",
        ]);

    for result in results {
        let status_str = match result.status {
            AnalysisStatus::Success => "✓ Success",
            AnalysisStatus::FetchError => "✗ Fetch Error",
            AnalysisStatus::ParseError => "✗ Parse Error",
        };

        let allowed_str = if matches!(result.status, AnalysisStatus::Success) {
            if result.is_path_allowed {
                "✓ Yes"
            } else {
                "✗ No"
            }
        } else {
            "-"
        };

        let licenses_str = if result.active_licenses.is_empty() {
            "-".to_string()
        } else {
            result.active_licenses.len().to_string()
        };

        let tdm_str = if let Some(ref tdm) = result.tdm_policy {
            if tdm.is_reserved {
                "⚠️  Yes"
            } else {
                "✓ No"
            }
        } else {
            "-"
        };

        let robots_meta_str = match result.robots_meta {
            Some(ref rm) => {
                let mut parts = Vec::new();
                if rm.is_noindex {
                    parts.push("noindex");
                }
                if rm.is_nofollow {
                    parts.push("nofollow");
                }
                if parts.is_empty() {
                    "-".to_string()
                } else {
                    parts.join(", ")
                }
            }
            None => "N/A".to_string(),
        };

        let markdown_str = if let Some(ref md) = result.markdown_agents {
            if md.supported {
                "✓ Yes"
            } else {
                "✗ No"
            }
        } else {
            "-"
        };

        // AI bot summary
        let blocked_count = result
            .ai_bot_analysis
            .iter()
            .filter(|b| matches!(b.status, BotStatus::Blocked))
            .count();
        let allowed_count = result
            .ai_bot_analysis
            .iter()
            .filter(|b| matches!(b.status, BotStatus::Allowed))
            .count();

        let ai_summary = if result.ai_bot_analysis.is_empty() {
            "-".to_string()
        } else {
            format!("{} blocked, {} allowed", blocked_count, allowed_count)
        };

        table.add_row(vec![
            Cell::new(&result.url),
            Cell::new(status_str),
            Cell::new(allowed_str),
            Cell::new(licenses_str),
            Cell::new(tdm_str),
            Cell::new(robots_meta_str),
            Cell::new(markdown_str),
            Cell::new(ai_summary),
        ]);
    }

    Ok(table.to_string())
}

pub fn format_json(results: &[AnalysisResult]) -> Result<String> {
    let json = serde_json::to_string_pretty(results)?;
    Ok(json)
}

/// Format results as CSV with major AI bot columns - perfect for advertiser analysis
pub fn format_csv(results: &[AnalysisResult]) -> Result<String> {
    let mut wtr = csv::Writer::from_writer(vec![]);

    let major_bots = AICrawler::get_major_bots();

    // Build header row
    let mut headers: Vec<String> = vec![
        "URL".into(),
        "Status".into(),
        "Path Allowed".into(),
        "RSL Licenses".into(),
        "TDM Reserved".into(),
        "Robots-Meta-NoIndex".into(),
        "Robots-Meta-NoFollow".into(),
        "CS-Search".into(),
        "CS-AI-Input".into(),
        "CS-AI-Train".into(),
        "Markdown".into(),
        "Markdown Tokens".into(),
        "MD-CS-Search".into(),
        "MD-CS-AI-Input".into(),
        "MD-CS-AI-Train".into(),
    ];
    for bot in &major_bots {
        headers.push(bot.name.clone());
    }
    headers.push("All User Agents".into());
    wtr.write_record(&headers)?;

    for result in results {
        let mut row: Vec<String> = Vec::new();

        row.push(result.url.clone());
        row.push(format!("{:?}", result.status));

        let path_allowed = if matches!(result.status, AnalysisStatus::Success) {
            if result.is_path_allowed {
                "Yes"
            } else {
                "No"
            }
        } else {
            "Error"
        };
        row.push(path_allowed.into());
        row.push(result.active_licenses.len().to_string());

        let tdm_reserved = match result.tdm_policy {
            Some(ref tdm) if tdm.is_reserved => "Yes",
            Some(_) => "No",
            None => "N/A",
        };
        row.push(tdm_reserved.into());

        let (noindex_str, nofollow_str) = match result.robots_meta {
            Some(ref rm) => (
                if rm.is_noindex { "Yes" } else { "No" },
                if rm.is_nofollow { "Yes" } else { "No" },
            ),
            None => ("N/A", "N/A"),
        };
        row.push(noindex_str.into());
        row.push(nofollow_str.into());

        row.push(
            result
                .content_signal_search
                .as_deref()
                .unwrap_or("unspecified")
                .into(),
        );
        row.push(
            result
                .content_signal_ai_input
                .as_deref()
                .unwrap_or("unspecified")
                .into(),
        );
        row.push(
            result
                .content_signal_ai_train
                .as_deref()
                .unwrap_or("unspecified")
                .into(),
        );

        // Markdown for Agents columns
        if let Some(ref md) = result.markdown_agents {
            row.push(if md.supported { "Yes" } else { "No" }.into());
            row.push(
                md.token_count
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "N/A".into()),
            );
            row.push(
                md.http_content_signal_search
                    .as_deref()
                    .unwrap_or("unspecified")
                    .into(),
            );
            row.push(
                md.http_content_signal_ai_input
                    .as_deref()
                    .unwrap_or("unspecified")
                    .into(),
            );
            row.push(
                md.http_content_signal_ai_train
                    .as_deref()
                    .unwrap_or("unspecified")
                    .into(),
            );
        } else {
            row.push("N/A".into());
            row.push("N/A".into());
            row.push("N/A".into());
            row.push("N/A".into());
            row.push("N/A".into());
        }

        for major_bot in &major_bots {
            let bot_status = result
                .ai_bot_analysis
                .iter()
                .find(|b| b.bot_name == major_bot.name)
                .map(|b| match b.status {
                    BotStatus::Blocked => "Blocked",
                    BotStatus::Allowed => "Allowed",
                })
                .unwrap_or("Unknown");
            row.push(bot_status.into());
        }

        let all_user_agents = result.user_agents.join("; ");
        row.push(all_user_agents);

        wtr.write_record(&row)?;
    }

    let data = String::from_utf8(wtr.into_inner()?)?;
    Ok(data)
}

pub fn format_compact(results: &[AnalysisResult]) -> Result<String> {
    let mut output = String::new();

    for result in results {
        output.push_str(&format!("\n{}\n", "=".repeat(80)));
        output.push_str(&format!("URL: {}\n", result.url));
        output.push_str(&format!("Robots.txt: {}\n", result.robots_url));

        match result.status {
            AnalysisStatus::Success => {
                output.push_str("Status: ✓ Success\n\n");

                if !result.user_agents.is_empty() {
                    output.push_str("User Agents:\n");
                    for agent in &result.user_agents {
                        output.push_str(&format!("  • {}\n", agent));
                    }
                    output.push('\n');
                }

                if let Some(delay) = result.crawl_delay {
                    output.push_str(&format!("Crawl Delay: {}s\n\n", delay));
                }

                output.push_str(&format!(
                    "Path Access: {}\n\n",
                    if result.is_path_allowed {
                        "✓ Allowed"
                    } else {
                        "✗ Disallowed"
                    }
                ));

                if !result.allowed_paths.is_empty() {
                    output.push_str("Allowed Paths:\n");
                    for path in &result.allowed_paths {
                        output.push_str(&format!("  ✓ {}\n", path));
                    }
                    output.push('\n');
                }

                if !result.disallowed_paths.is_empty() {
                    output.push_str("Disallowed Paths:\n");
                    for path in &result.disallowed_paths {
                        output.push_str(&format!("  ✗ {}\n", path));
                    }
                    output.push('\n');
                }

                if !result.sitemaps.is_empty() {
                    output.push_str("Sitemaps:\n");
                    for sitemap in &result.sitemaps {
                        output.push_str(&format!("  • {}\n", sitemap));
                    }
                    output.push('\n');
                }

                if !result.active_licenses.is_empty() {
                    output.push_str("RSL Licenses (Active):\n");
                    for license in &result.active_licenses {
                        output.push_str(&format!("  📜 {}\n", license));
                    }
                    output.push('\n');
                }

                if !result.global_licenses.is_empty() && result.active_licenses.is_empty() {
                    output.push_str("RSL Licenses (Global):\n");
                    for license in &result.global_licenses {
                        output.push_str(&format!("  📜 {}\n", license));
                    }
                    output.push('\n');
                }

                if !result.group_licenses.is_empty() {
                    output.push_str("RSL Licenses (Group-Scoped):\n");
                    for license in &result.group_licenses {
                        output.push_str(&format!("  📜 {}\n", license));
                    }
                    output.push('\n');
                }

                // Content Signals
                if result.content_signal_search.is_some()
                    || result.content_signal_ai_input.is_some()
                    || result.content_signal_ai_train.is_some()
                {
                    output.push_str("Content Signals:\n");
                    if let Some(ref search) = result.content_signal_search {
                        let icon = if search == "yes" { "✓" } else { "✗" };
                        output.push_str(&format!("  {} search: {}\n", icon, search));
                    }
                    if let Some(ref ai_input) = result.content_signal_ai_input {
                        let icon = if ai_input == "yes" { "✓" } else { "✗" };
                        output.push_str(&format!("  {} ai-input: {}\n", icon, ai_input));
                    }
                    if let Some(ref ai_train) = result.content_signal_ai_train {
                        let icon = if ai_train == "yes" { "✓" } else { "✗" };
                        output.push_str(&format!("  {} ai-train: {}\n", icon, ai_train));
                    }
                    output.push('\n');
                }

                // Markdown for Agents
                if let Some(ref md) = result.markdown_agents {
                    output.push_str("Markdown for Agents:\n");
                    let icon = if md.supported { "✓" } else { "✗" };
                    output.push_str(&format!(
                        "  {} Supported: {}\n",
                        icon,
                        if md.supported { "Yes" } else { "No" }
                    ));
                    if let Some(tokens) = md.token_count {
                        output.push_str(&format!("  📊 Token Count: {}\n", tokens));
                    }
                    if md.http_content_signal_search.is_some()
                        || md.http_content_signal_ai_input.is_some()
                        || md.http_content_signal_ai_train.is_some()
                    {
                        output.push_str("  HTTP Content Signals:\n");
                        if let Some(ref search) = md.http_content_signal_search {
                            output.push_str(&format!("    search: {}\n", search));
                        }
                        if let Some(ref ai_input) = md.http_content_signal_ai_input {
                            output.push_str(&format!("    ai-input: {}\n", ai_input));
                        }
                        if let Some(ref ai_train) = md.http_content_signal_ai_train {
                            output.push_str(&format!("    ai-train: {}\n", ai_train));
                        }
                    }
                    output.push('\n');
                }

                if let Some(ref tdm) = result.tdm_policy {
                    output.push_str("TDM Policy:\n");
                    output.push_str(&format!(
                        "  {} TDM Reservation: {}\n",
                        if tdm.is_reserved { "⚠️ " } else { "✓" },
                        if tdm.is_reserved {
                            "YES (reserved)"
                        } else {
                            "NO (unreserved)"
                        }
                    ));

                    if let Some(ref matched) = tdm.matched_rule {
                        output.push_str(&format!("  🎯 Matched Rule: {}\n", matched.location));
                        if let Some(ref policy_url) = matched.tdm_policy {
                            output.push_str(&format!("  📄 Policy: {}\n", policy_url));
                        }
                    }

                    output.push_str(&format!("  📋 Total Rules: {}\n", tdm.rules.len()));
                }

                // Robots Meta (page-level directives)
                if let Some(ref rm) = result.robots_meta {
                    output.push_str("\nRobots Meta Directives:\n");
                    if rm.is_noindex {
                        output.push_str("  ✗ noindex\n");
                    }
                    if rm.is_nofollow {
                        output.push_str("  ✗ nofollow\n");
                    }
                    if !rm.is_noindex && !rm.is_nofollow {
                        output.push_str("  ✓ No restrictions\n");
                    }
                    for entry in &rm.entries {
                        let source = match entry.source {
                            policycheck_core::checks::robots_meta::RobotsMetaSource::MetaTag => {
                                "meta tag"
                            }
                            policycheck_core::checks::robots_meta::RobotsMetaSource::HttpHeader => {
                                "X-Robots-Tag"
                            }
                        };
                        let bot = entry.bot_name.as_deref().unwrap_or("all");
                        output.push_str(&format!("  {} ({}): {}\n", source, bot, entry.raw.trim()));
                    }
                }

                // AI Bot Analysis
                if !result.ai_bot_analysis.is_empty() {
                    output.push('\n');

                    let blocked: Vec<_> = result
                        .ai_bot_analysis
                        .iter()
                        .filter(|b| matches!(b.status, BotStatus::Blocked))
                        .collect();

                    let allowed: Vec<_> = result
                        .ai_bot_analysis
                        .iter()
                        .filter(|b| matches!(b.status, BotStatus::Allowed))
                        .collect();

                    output.push_str(&format!(
                        "AI Bot Analysis: {} blocked, {} allowed\n",
                        blocked.len(),
                        allowed.len()
                    ));

                    if !blocked.is_empty() {
                        output.push_str("\n🚫 Blocked AI Crawlers:\n");
                        for bot in blocked.iter().take(10) {
                            output.push_str(&format!(
                                "  ✗ {} — {} ({})\n",
                                bot.bot_name, bot.company, bot.category
                            ));
                        }
                        if blocked.len() > 10 {
                            output.push_str(&format!("  ... and {} more\n", blocked.len() - 10));
                        }
                    }

                    if !allowed.is_empty() {
                        output.push_str("\n✓ Allowed AI Crawlers:\n");
                        for bot in allowed.iter().take(10) {
                            output.push_str(&format!(
                                "  ✓ {} — {} ({})\n",
                                bot.bot_name, bot.company, bot.category
                            ));
                        }
                        if allowed.len() > 10 {
                            output.push_str(&format!("  ... and {} more\n", allowed.len() - 10));
                        }
                    }

                    output.push_str("\nℹ️  For full AI bot analysis, use --format csv\n");
                }
            }
            _ => {
                output.push_str(&format!("Status: ✗ {:?}\n", result.status));
                if let Some(error) = &result.error {
                    output.push_str(&format!("Error: {}\n", error));
                }
            }
        }
    }

    output.push_str(&format!("\n{}\n", "=".repeat(80)));

    let successful = results
        .iter()
        .filter(|r| matches!(r.status, AnalysisStatus::Success))
        .count();
    let failed = results.len() - successful;

    output.push_str(&format!(
        "\nSummary: {} total, {} successful, {} failed\n",
        results.len(),
        successful,
        failed
    ));

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_result(url: &str) -> AnalysisResult {
        AnalysisResult {
            url: url.to_string(),
            robots_url: format!("{}/robots.txt", url),
            status: AnalysisStatus::Success,
            user_agents: vec!["*".to_string()],
            crawl_delay: None,
            sitemaps: vec![],
            allowed_paths: vec![],
            disallowed_paths: vec![],
            is_path_allowed: true,
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
            error: None,
        }
    }

    #[test]
    fn test_csv_header_contains_bot_columns() {
        let results = vec![make_result("https://www.nytimes.com")];
        let csv = format_csv(&results).unwrap();
        let header_line = csv.lines().next().unwrap();
        assert!(header_line.contains("URL"));
        assert!(header_line.contains("GPTBot"));
        assert!(header_line.contains("ClaudeBot"));
        assert!(header_line.contains("Markdown"));
        assert!(header_line.contains("Markdown Tokens"));
        assert!(header_line.contains("All User Agents"));
    }

    #[test]
    fn test_csv_escapes_commas() {
        let mut result = make_result("https://www.nytimes.com");
        result.user_agents = vec!["Bot,One".to_string(), "BotTwo".to_string()];
        let csv = format_csv(&[result]).unwrap();
        let data_line = csv.lines().nth(1).unwrap();
        assert!(data_line.contains("\"Bot,One; BotTwo\""));
    }

    #[test]
    fn test_json_round_trip() {
        let result = make_result("https://github.com");
        let json_str = format_json(&[result]).unwrap();
        let parsed: Vec<AnalysisResult> = serde_json::from_str(&json_str).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].url, "https://github.com");
    }
}
