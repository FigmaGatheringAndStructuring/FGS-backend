use axum::extract::FromRef;
use domain::figma::usecase::FigmaUseCase;
use std::sync::Arc;

#[derive(FromRef, Clone)]
pub struct AppState {
    pub figma_service: Arc<dyn FigmaUseCase>,
}
