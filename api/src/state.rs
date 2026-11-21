use std::sync::Arc;
use domain::figma::usecase::FigmaUseCase;
use axum::extract::FromRef;

#[derive(FromRef,Clone)]
pub struct AppState {
    pub figma_service: Arc<dyn FigmaUseCase>,
}