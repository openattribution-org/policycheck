use crate::ai_crawlers::{AICrawler, BotStatus};
use crate::models::{AnalysisResult, AnalysisStatus};
use anyhow::Result;
use comfy_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, *};

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
            "AI Bots Summary",
        ]);

    for result in results {
        let status_str = match result.status {
            AnalysisStatus::Success => "✓ Success",
            AnalysisStatus::FetchError => "✗ Fetch Error",
            AnalysisStatus::ParseError => "✗ Parse Error",
            AnalysisStatus::InvalidUrl => "✗ Invalid URL",
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
    let mut csv = String::new();

    // Get major bots for column headers
    let major_bots = AICrawler::get_major_bots();

    // Build header row
    let mut headers = vec![
        "URL",
        "Status",
        "Path Allowed",
        "RSL Licenses",
        "TDM Reserved",
        "CS-Search",
        "CS-AI-Input",
        "CS-AI-Train",
    ];

    // Add bot columns
    for bot in &major_bots {
        headers.push(&bot.name);
    }

    // Add final column for all user agents
    headers.push("All User Agents");

    csv.push_str(&headers.join(","));
    csv.push('\n');

    // Add data rows
    for result in results {
        let mut row = Vec::new();

        // Basic columns
        row.push(escape_csv_field(&result.url));
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
        row.push(path_allowed.to_string());

        row.push(result.active_licenses.len().to_string());

        let tdm_reserved = if let Some(ref tdm) = result.tdm_policy {
            if tdm.is_reserved {
                "Yes"
            } else {
                "No"
            }
        } else {
            "N/A"
        };
        row.push(tdm_reserved.to_string());

        // Add Content Signal columns
        row.push(
            result
                .content_signal_search
                .as_deref()
                .unwrap_or("unspecified")
                .to_string(),
        );
        row.push(
            result
                .content_signal_ai_input
                .as_deref()
                .unwrap_or("unspecified")
                .to_string(),
        );
        row.push(
            result
                .content_signal_ai_train
                .as_deref()
                .unwrap_or("unspecified")
                .to_string(),
        );

        // Add bot status columns
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

            row.push(bot_status.to_string());
        }

        // Add all user agents as final column
        let all_user_agents = if result.user_agents.is_empty() {
            String::new()
        } else {
            escape_csv_field(&result.user_agents.join("; "))
        };
        row.push(all_user_agents);

        csv.push_str(&row.join(","));
        csv.push('\n');
    }

    Ok(csv)
}

/// Escape CSV field if it contains comma, quote, or newline
fn escape_csv_field(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
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

                // Content Signals (Cloudflare AI policy framework)
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
