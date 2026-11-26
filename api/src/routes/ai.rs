use crate::handlers::ai;
use crate::state::AppState;
use axum::Router;
use axum::routing::get;

pub fn ai_routes() -> Router<AppState> {
    Router::new().route("/generate", get(ai::generate))
}
