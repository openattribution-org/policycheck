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
            "User Agents",
            "Crawl Delay",
            "Path Allowed",
            "RSL Licenses",
            "TDM Reserved",
            "Sitemaps",
            "Disallowed",
        ]);

    for result in results {
        let status_str = match result.status {
            AnalysisStatus::Success => "✓ Success",
            AnalysisStatus::FetchError => "✗ Fetch Error",
            AnalysisStatus::ParseError => "✗ Parse Error",
            AnalysisStatus::InvalidUrl => "✗ Invalid URL",
        };

        let user_agents_str = if result.user_agents.is_empty() {
            "-".to_string()
        } else {
            result.user_agents.join(", ")
        };

        let crawl_delay_str = result
            .crawl_delay
            .map(|d| format!("{}s", d))
            .unwrap_or_else(|| "-".to_string());

        let allowed_str = if matches!(result.status, AnalysisStatus::Success) {
            if result.is_path_allowed {
                "✓ Yes"
            } else {
                "✗ No"
            }
        } else {
            "-"
        };

        let sitemaps_str = if result.sitemaps.is_empty() {
            "-".to_string()
        } else {
            result.sitemaps.len().to_string()
        };

        let disallowed_str = if result.disallowed_paths.is_empty() {
            "-".to_string()
        } else {
            result.disallowed_paths.len().to_string()
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

        table.add_row(vec![
            Cell::new(&result.url),
            Cell::new(status_str),
            Cell::new(user_agents_str),
            Cell::new(crawl_delay_str),
            Cell::new(allowed_str),
            Cell::new(licenses_str),
            Cell::new(tdm_str),
            Cell::new(sitemaps_str),
            Cell::new(disallowed_str),
        ]);
    }

    Ok(table.to_string())
}

pub fn format_json(results: &[AnalysisResult]) -> Result<String> {
    let json = serde_json::to_string_pretty(results)?;
    Ok(json)
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

                if let Some(ref tdm) = result.tdm_policy {
                    output.push_str("TDM Policy:\n");
                    output.push_str(&format!(
                        "  {} TDM Reservation: {}\n",
                        if tdm.is_reserved { "⚠️ " } else { "✓" },
                        if tdm.is_reserved { "YES (reserved)" } else { "NO (unreserved)" }
                    ));

                    if let Some(ref matched) = tdm.matched_rule {
                        output.push_str(&format!("  🎯 Matched Rule: {}\n", matched.location));
                        if let Some(ref policy_url) = matched.tdm_policy {
                            output.push_str(&format!("  📄 Policy: {}\n", policy_url));
                        }
                    }

                    output.push_str(&format!("  📋 Total Rules: {}\n", tdm.rules.len()));
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
