use crate::state::AppState;
use crate::types::*;
use crate::utils::error::{Error, Result};
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
use candid::Principal;
use std::sync::Arc;

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
