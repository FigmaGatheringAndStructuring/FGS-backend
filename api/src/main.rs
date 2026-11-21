mod handlers;
mod routes;
mod state;

use std::sync::Arc;

use crate::routes::{figma, health};
use axum::Router;
use domain::figma::usecase::FigmaUseCase;
use infra::figma::FigmaClient;
use serde::{Deserialize, Serialize};
use usecase::FigmaService;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
struct FigmaElementQuery {
    url: String,
}

#[derive(Debug, Deserialize)]
struct FigmaPageQuery {
    url: String,
}

#[derive(Debug, Deserialize)]
struct FigmaRenderQuery {
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
    let client = Arc::new(FigmaClient::new(token));

    let state = AppState{
        figma_service: Arc::new(FigmaService::new(client)),
    };

    let app = Router::new()
        .nest("/health", health::health_routes())
        .nest("/api/figma", figma::figma_routes())
        .with_state(state);

    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "8000".to_string());
    let listener = tokio::net::TcpListener::bind(format!("{host}:{port}")).await?;

    println!("listening on http://{host}:{port}");
    axum::serve(listener, app).await?;

    Ok(())
}
