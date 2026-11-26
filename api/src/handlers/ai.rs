use std::sync::Arc;

use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use domain::ai::model::AiResult;
use domain::ai::usecase::AiUseCase;
use domain::figma::model::FigmaLink;

use crate::handlers::error::{ApiError, bad_gateway, bad_request};
use crate::{AiGenerateQuery, ErrorResponse};

pub async fn generate(
    State(service): State<Arc<dyn AiUseCase>>,
    Query(query): Query<AiGenerateQuery>,
) -> Result<Json<AiResult>, ApiError> {
    let link = FigmaLink::parse(&query.url).map_err(bad_request)?;

    let scale = query.scale.unwrap_or(1.0);
    if !(0.01..=4.0).contains(&scale) {
        return Err(bad_request("scale must be between 0.01 and 4.0"));
    }

    service
        .get_result(&link, scale)
        .await
        .map(Json)
        .map_err(bad_gateway)
}
