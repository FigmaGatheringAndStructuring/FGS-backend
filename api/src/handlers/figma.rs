use std::sync::Arc;

use axum::Json;
use axum::body::Body;
use axum::extract::{Query, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::Response;
use domain::figma::model::{FigmaLink, FigmaPageLayout};
use domain::figma::usecase::FigmaUseCase;

use crate::{ErrorResponse, FigmaPageQuery, FigmaRenderQuery};

type ApiError = (StatusCode, Json<ErrorResponse>);

fn bad_request(error: impl ToString) -> ApiError {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            error: error.to_string(),
        }),
    )
}

fn bad_gateway(error: impl ToString) -> ApiError {
    (
        StatusCode::BAD_GATEWAY,
        Json(ErrorResponse {
            error: error.to_string(),
        }),
    )
}

pub async fn get_figma_page(
    State(service): State<Arc<dyn FigmaUseCase>>,
    Query(query): Query<FigmaPageQuery>,
) -> Result<Json<FigmaPageLayout>, ApiError> {
    let link = FigmaLink::parse(&query.url).map_err(bad_request)?;

    service.get_page(&link).await.map(Json).map_err(bad_gateway)
}

pub async fn render_figma_page(
    State(service): State<Arc<dyn FigmaUseCase>>,
    Query(query): Query<FigmaRenderQuery>,
) -> Result<Response, ApiError> {
    let link = FigmaLink::parse(&query.url).map_err(bad_request)?;

    let scale = query.scale.unwrap_or(1.0);
    if !(0.01..=4.0).contains(&scale) {
        return Err(bad_request("scale must be between 0.01 and 4.0"));
    }

    let image = service
        .render_page(&link, scale)
        .await
        .map_err(bad_gateway)?;

    let content_type = HeaderValue::from_str(&image.content_type)
        .unwrap_or(HeaderValue::from_static("application/octet-stream"));

    let mut response = Response::new(Body::from(image.bytes));
    response
        .headers_mut()
        .insert(header::CONTENT_TYPE, content_type);

    Ok(response)
}
