use axum::extract::FromRef;
use domain::ai::usecase::AiUseCase;
use domain::figma::usecase::FigmaUseCase;
use std::sync::Arc;

#[derive(FromRef, Clone)]
pub struct AppState {
    pub figma_service: Arc<dyn FigmaUseCase>,
    pub ai_service: Arc<dyn AiUseCase>,
}
