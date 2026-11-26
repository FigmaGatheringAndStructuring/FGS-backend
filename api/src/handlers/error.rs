use crate::ErrorResponse;
use axum::Json;
use axum::http::StatusCode;

pub type ApiError = (StatusCode, Json<ErrorResponse>);
pub fn bad_request(error: impl ToString) -> ApiError {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            error: error.to_string(),
        }),
    )
}

pub fn bad_gateway(error: impl ToString) -> ApiError {
    (
        StatusCode::BAD_GATEWAY,
        Json(ErrorResponse {
            error: error.to_string(),
        }),
    )
}
