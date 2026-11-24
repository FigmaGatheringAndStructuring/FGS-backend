use crate::handlers::figma;
use crate::state::AppState;
use axum::Router;
use axum::routing::get;

pub fn figma_routes() -> Router<AppState> {
    Router::new()
        .route("/page", get(figma::get_figma_page))
        .route("/page/render", get(figma::render_figma_page))
}
