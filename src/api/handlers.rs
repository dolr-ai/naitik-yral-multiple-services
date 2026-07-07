use crate::events::types::{EventPayload, NewAIInfluencerMsgPayload};
use crate::state::AppState;
use crate::utils::error::{Error, NullOk, Result};
use crate::{types::ApiResult, utils::error::ErrorWrapper};
use axum::{extract::State, http::HeaderMap, response::IntoResponse, Json};
use reqwest::StatusCode;
use std::sync::Arc;

#[utoipa::path(
    get,
    path = "/authenticated_health",
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
pub async fn authenticated_health(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<ApiResult<String>>> {
    let token = headers
        .get("Authorization")
        .ok_or(Error::AuthTokenMissing)?
        .to_str()
        .map_err(|_| Error::AuthTokenInvalid)?;
    let token = token.trim_start_matches("Bearer ");

    // Verify JWT token
    crate::auth::verify_token(token, &state.jwt_details)?;

    Ok(Json(Ok("Authenticated health check passed".to_string())))
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

#[utoipa::path(
    post,
    path = "/api/v1/events/new-ai-influencer-message",
    request_body = NewAIInfluencerMsgPayload,
    tag = "events",
    responses(
        (status = 200, description = "Event sent successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn new_ai_influencer_message(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<NewAIInfluencerMsgPayload>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let token = headers
        .get("Authorization")
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                "Missing Authorization header".to_string(),
            )
        })?
        .to_str()
        .map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                "Invalid Authorization header encoding".to_string(),
            )
        })?;

    let token = token.trim_start_matches("Bearer ");

    // Verify JWT token
    crate::auth::verify_token(token, &state.jwt_details).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            "Invalid authentication token".to_string(),
        )
    })?;

    let event = EventPayload::NewAIInfluencerMsgPayload(payload);
    event.send_notification(&state).await;

    Ok((
        StatusCode::OK,
        "AI Influencer Chat Notification Processed".to_string(),
    ))
}
