use crate::handlers::health;
use crate::state::AppState;
use axum::Router;
use axum::routing::get;

pub fn health_routes() -> Router<AppState> {
    Router::new().route("/", get(health::health))
}
