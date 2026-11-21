use crate::handlers::figma;
use axum::Router;
use axum::routing::get;
use crate::state::AppState;

pub fn figma_routes() -> Router<AppState> {
    Router::new()
        .route("/element", get(figma::get_figma_element))
        .route("/page", get(figma::get_figma_page))
        .route("/page/render", get(figma::render_figma_page))
}
