use crate::state::AppState;
use crate::utils::error::{Error, NullOk, Result};
use crate::utils::recsys_client::get_view_counts_for_videos;
use crate::{
    types::ApiResult,
    utils::error::{ErrorWrapper, OkWrapper},
};
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;

#[utoipa::path(
    post,
    path = "/recsys/send-view-count",
    request_body = Value,
    responses(
        (status = 200, description = "View count sent to recsys successfully", body = NullOk), // OkWrapper<()> panics for some reason
        (status = 400, description = "Invalid request", body = ErrorWrapper<crate::utils::error::Error>),
        (status = 401, description = "Unauthorized", body = ErrorWrapper<crate::utils::error::Error>),
        (status = 500, description = "Internal server error", body = ErrorWrapper<crate::utils::error::Error>)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn send_view_count_to_recsys(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<Vec<String>>,
) -> Result<Json<ApiResult<()>>> {
    let token = headers
        .get("Authorization")
        .ok_or(Error::AuthTokenMissing)?
        .to_str()
        .map_err(|_| Error::AuthTokenInvalid)?;
    let token = token.trim_start_matches("Bearer ");

    // Verify JWT token
    crate::auth::verify_token(token, &state.jwt_details)?;

    // Get view counts for the requested video IDs and send to recsys-system
    let view_counts = get_view_counts_for_videos(&state.dragonfly_redis_store, &req).await?;
    state
        .recsys_client
        .send_bulk_view_count_to_recsys(view_counts);

    Ok(Json(Ok(())))
}

#[utoipa::path(
    get,
    path = "/healthz",
    responses(
        (status = 200, description = "Service is healthy", body = serde_json::Value)
    ),
    tag = "Health"
)]
pub async fn healthz() -> axum::response::Response {
    Json(serde_json::json!({"status": "ok"})).into_response()
}
