use axum::Router;
use axum::routing::get;
use crate::handlers::health;
use crate::state::AppState;

pub fn health_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(health::health))
}