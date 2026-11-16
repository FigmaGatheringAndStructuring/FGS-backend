use std::sync::Arc;
use axum::Router;
use axum::routing::get;
use domain::figma::usecase::FigmaUseCase;
use crate::handlers::health;

pub fn health_routes() -> Router<Arc<dyn FigmaUseCase>> {
    Router::new()
        .route("/", get(health::health))
}