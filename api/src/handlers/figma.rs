use std::sync::Arc;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use domain::figma::model::{FigmaDocument, FigmaLink};
use domain::figma::usecase::FigmaUseCase;
use crate::{ErrorResponse, FigmaElementQuery};

pub async fn get_figma_element(
    State(service): State<Arc<dyn FigmaUseCase>>,
    Query(query): Query<FigmaElementQuery>,
) -> Result<Json<FigmaDocument>, (StatusCode, Json<ErrorResponse>)> {
    let link = FigmaLink::parse(&query.url).map_err(|error| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: error.to_string(),
            }),
        )
    })?;

    service.get_element(&link).await.map(Json).map_err(|error| {
        (
            StatusCode::BAD_GATEWAY,
            Json(ErrorResponse {
                error: error.to_string(),
            }),
        )
    })
}
