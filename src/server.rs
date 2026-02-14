use crate::analyzer::RobotAnalyzer;
use crate::models::{AnalysisStatus, AnalyzeRequest, AnalyzeResponse};
use anyhow::Result;
use axum::{
    extract::Json,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use serde_json::json;
use tower_http::cors::{Any, CorsLayer};

pub async fn start_server(host: &str, port: u16) -> Result<()> {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(health_check))
        .route("/health", get(health_check))
        .route("/analyze", post(analyze_handler))
        .layer(cors);

    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    println!("🚀 PolicyCheck server listening on http://{}", addr);
    println!("\nEndpoints:");
    println!("  GET  /health  - Health check");
    println!("  POST /analyze - Check publisher policies (robots.txt, RSL licenses, TDM)");
    println!("\nExample:");
    println!(r#"  curl -X POST http://{}/analyze \"#, addr);
    println!(r#"    -H "Content-Type: application/json" \"#);
    println!(r#"    -d '{{"urls": ["https://example.com"], "user_agent": "MyBot"}}'"#);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> impl IntoResponse {
    Json(json!({
        "status": "healthy",
        "service": "policycheck",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

async fn analyze_handler(
    Json(request): Json<AnalyzeRequest>,
) -> Result<Json<AnalyzeResponse>, (StatusCode, String)> {
    if request.urls.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "No URLs provided".to_string()));
    }

    let analyzer = RobotAnalyzer::new(request.user_agent);
    let results = analyzer.analyze_urls(&request.urls).await;

    let successful = results
        .iter()
        .filter(|r| matches!(r.status, AnalysisStatus::Success))
        .count();

    let response = AnalyzeResponse {
        total: results.len(),
        successful,
        failed: results.len() - successful,
        results,
    };

    Ok(Json(response))
}
