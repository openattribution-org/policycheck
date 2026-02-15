use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

mod ai_crawlers;
mod analyzer;
mod fetcher;
mod models;
mod output;
mod server;

use analyzer::RobotAnalyzer;

#[derive(Clone, ValueEnum)]
enum OutputFormat {
    Table,
    Json,
    Csv,
    Compact,
}

#[derive(Parser)]
#[command(name = "policycheck")]
#[command(about = "Publisher policy compliance checker - verifies robots.txt, RSL licenses, and TDM policies", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze robots.txt from URLs
    Analyze {
        /// URLs to analyze (can be specified multiple times)
        #[arg(short, long)]
        url: Vec<String>,

        /// Path to CSV file containing URLs (one per line or in 'url' column)
        #[arg(short, long)]
        csv: Option<PathBuf>,

        /// User agent to check permissions for
        #[arg(short = 'a', long, default_value = "*")]
        user_agent: String,

        /// Output format
        #[arg(short, long, default_value = "table")]
        format: OutputFormat,

        /// Save output to file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Start HTTP API server
    Serve {
        /// Port to listen on
        #[arg(short, long, default_value = "3000")]
        port: u16,

        /// Host to bind to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Analyze {
            url,
            csv,
            user_agent,
            format,
            output,
        } => {
            let analyzer = RobotAnalyzer::new(user_agent);

            // Collect URLs from both sources
            let mut urls = url;

            if let Some(csv_path) = csv {
                let csv_urls = analyzer
                    .read_csv(&csv_path)
                    .context("Failed to read CSV file")?;
                urls.extend(csv_urls);
            }

            if urls.is_empty() {
                eprintln!("Error: No URLs provided. Use --url or --csv");
                std::process::exit(1);
            }

            // Analyze all URLs
            let results = analyzer.analyze_urls(&urls).await;

            // Output results
            let output_str = match format {
                OutputFormat::Json => output::format_json(&results)?,
                OutputFormat::Compact => output::format_compact(&results)?,
                OutputFormat::Csv => output::format_csv(&results)?,
                OutputFormat::Table => output::format_table(&results)?,
            };

            if let Some(output_path) = output {
                std::fs::write(&output_path, &output_str).context("Failed to write output file")?;
                println!("Results written to {}", output_path.display());
            } else {
                println!("{}", output_str);
            }
        }
        Commands::Serve { port, host } => {
            server::start_server(&host, port).await?;
        }
    }

    Ok(())
}
