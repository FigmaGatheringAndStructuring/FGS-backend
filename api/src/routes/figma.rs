use std::sync::Arc;
use axum::Router;
use axum::routing::get;
use domain::figma::usecase::FigmaUseCase;
use crate::handlers::figma;

pub fn figma_routes() -> Router<Arc<dyn FigmaUseCase>> {
    Router::new()
        .route("/element", get(figma::get_figma_element))
}