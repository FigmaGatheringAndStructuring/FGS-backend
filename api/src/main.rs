mod handlers;
mod routes;
mod state;
use crate::routes::{ai, figma, health};
use crate::state::AppState;
use axum::Router;
use infra::ai::AiClient;
use infra::figma::FigmaClient;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use usecase::{AiService, FigmaService};

#[derive(Debug, Deserialize)]
struct FigmaPageQuery {
    url: String,
}

#[derive(Debug, Deserialize)]
struct FigmaRenderQuery {
    url: String,
    scale: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct AiGenerateQuery {
    url: String,
    scale: Option<f64>,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let token = std::env::var("FIGMA_TOKEN")
        .map_err(|_| anyhow::anyhow!("FIGMA_TOKEN environment variable is required"))?;
    let figma_client = Arc::new(FigmaClient::new(token));

    let ai_api_key = std::env::var("AI_API_KEY").unwrap_or_default();
    let ai_base_url = std::env::var("AI_BASE_URL")
        .ok()
        .filter(|base_url| !base_url.trim().is_empty());
    let mut ai_client = AiClient::new(ai_api_key);
    if let Some(base_url) = ai_base_url {
        ai_client = ai_client.with_base_url(base_url);
    }
    let ai_client = Arc::new(ai_client);

    let figma_service = Arc::new(FigmaService::new(figma_client));

    let state = AppState {
        ai_service: Arc::new(AiService::new(figma_service.clone(), ai_client)),
        figma_service,
    };

    let app = Router::new()
        .nest("/health", health::health_routes())
        .nest("/api/figma", figma::figma_routes())
        .nest("/api/ai", ai::ai_routes())
        .with_state(state);

    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "8000".to_string());
    let listener = tokio::net::TcpListener::bind(format!("{host}:{port}")).await?;

    println!("listening on http://{host}:{port}");
    axum::serve(listener, app).await?;

    Ok(())
}
