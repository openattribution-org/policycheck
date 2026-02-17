use crate::analyzer::RobotAnalyzer;
use anyhow::Result;
use axum::{
    extract::{DefaultBodyLimit, Json},
    http::{HeaderValue, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use policycheck_core::models::{AnalysisResult, AnalysisStatus};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use tower_http::cors::{Any, CorsLayer};

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

const MAX_URLS_PER_REQUEST: usize = 100;

struct ApiError {
    status: StatusCode,
    message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let body = Json(serde_json::json!({
            "error": self.message
        }));
        (self.status, body).into_response()
    }
}

pub fn build_router() -> Router {
    let cors = if let Ok(allowed_origins) = env::var("ALLOWED_ORIGINS") {
        let origins: Vec<HeaderValue> = allowed_origins
            .split(',')
            .filter_map(|s| s.trim().parse::<HeaderValue>().ok())
            .collect();

        CorsLayer::new()
            .allow_origin(origins)
            .allow_methods(Any)
            .allow_headers(Any)
    } else {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
    };

    Router::new()
        .route("/", get(health_check))
        .route("/health", get(health_check))
        .route("/analyze", post(analyze_handler))
        .layer(cors)
        .layer(DefaultBodyLimit::max(1_048_576))
}

pub async fn start_server(host: &str, port: u16) -> Result<()> {
    let app = build_router();

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
) -> Result<Json<AnalyzeResponse>, ApiError> {
    if request.urls.is_empty() {
        return Err(ApiError {
            status: StatusCode::BAD_REQUEST,
            message: "No URLs provided".to_string(),
        });
    }

    if request.urls.len() > MAX_URLS_PER_REQUEST {
        return Err(ApiError {
            status: StatusCode::BAD_REQUEST,
            message: format!(
                "Too many URLs: {} provided, maximum is {}",
                request.urls.len(),
                MAX_URLS_PER_REQUEST
            ),
        });
    }

    println!(
        "📊 Analyzing {} URL(s) with user-agent: {} | URLs: {:?}",
        request.urls.len(),
        request.user_agent,
        request.urls
    );

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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_health_check() {
        let app = build_router();

        let request = axum::http::Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(body["status"], "healthy");
    }

    #[tokio::test]
    async fn test_empty_urls() {
        let app = build_router();

        let request = axum::http::Request::builder()
            .uri("/analyze")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::to_string(&json!({
                    "urls": [],
                    "user_agent": "TestBot"
                }))
                .unwrap(),
            ))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(body["error"], "No URLs provided");
    }

    #[tokio::test]
    async fn test_too_many_urls() {
        let app = build_router();

        let urls: Vec<String> = (0..101)
            .map(|i| format!("https://example-{}.com", i))
            .collect();

        let request = axum::http::Request::builder()
            .uri("/analyze")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::to_string(&json!({
                    "urls": urls,
                    "user_agent": "TestBot"
                }))
                .unwrap(),
            ))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body: serde_json::Value = serde_json::from_slice(&body).unwrap();

        let error_msg = body["error"].as_str().unwrap();
        assert!(
            error_msg.contains("100"),
            "Error should mention max 100 URLs"
        );
    }
}
