use std::{collections::HashMap, sync::Arc};

use axum::{extract::State, response::IntoResponse, Json};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use yral_metrics::metrics::sealed_metric::SealedMetric;

use crate::events::event::WareHouseEvent;
use crate::{
    events::{event::Event, types::AnalyticsEvent},
    state::AppState,
};
pub use yral_types::delegated_identity::DelegatedIdentityWire;

pub mod event;
pub mod push_notification;
pub mod types;
pub mod utils;

/// Convert PascalCase to snake_case (e.g., "VideoDurationWatched" -> "video_duration_watched")
fn to_snake_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 5);
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct EventBulkRequest {
    pub delegated_identity_wire: DelegatedIdentityWire,
    pub events: Vec<AnalyticsEvent>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct VerifiedEventBulkRequest {
    pub events: Vec<AnalyticsEvent>,
}

/// V2 bulk event request with delegated identity auth and arbitrary payloads
#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct EventBulkRequestV2 {
    pub delegated_identity_wire: DelegatedIdentityWire,
    /// Array of event payloads - each must contain "event" field for the event name
    #[schema(value_type = Vec<Object>)]
    pub events: Vec<Value>,
}

/// V2 verified bulk events (after middleware validation)
#[derive(Clone, Serialize, Deserialize)]
pub struct VerifiedEventBulkRequestV2 {
    pub events: Vec<Value>,
    pub user_id: String, // User ID from delegated identity
}

#[derive(Serialize, Deserialize, Clone, ToSchema, Debug)]
pub struct EventRequest {
    pub event: String,
    pub params: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/events",
    request_body = EventRequest,
    tag = "events",
    responses(
        (status = 200, description = "Event sent successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn post_event(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<EventRequest>,
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

    let warehouse_event = WareHouseEvent {
        event: payload.event,
        params: payload.params,
    };

    let event = Event::new(warehouse_event);
    process_event_impl(event, state.clone())
        .await
        .map_err(|e| {
            log::error!("Failed to process event: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to process event".to_string(),
            )
        })?;

    Ok((StatusCode::OK, "Event processed".to_string()))
}

pub async fn process_event_impl(
    event: Event,
    state: Arc<AppState>,
) -> Result<(), crate::utils::error::Error> {
    let mut video_view_counts = HashMap::new();
    event
        .process_video_view_count(&state, &mut video_view_counts)
        .await
        .map_err(|e| {
            log::error!("Failed to process event rest: {e}");
            crate::utils::error::Error::Unknown("Failed to process event".to_string())
        })?;

    state.send_bulk_view_count_to_recsys(video_view_counts);

    Ok(())
}

#[utoipa::path(
    post,
    path = "/api/v2/events",
    request_body = EventRequest,
    tag = "events",
    responses(
        (status = 200, description = "Event sent successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn post_event_v2(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<EventRequest>,
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

    // Convert event name to snake_case for backwards compat with mobile sending PascalCase
    let event_name = to_snake_case(&payload.event);

    let warehouse_event = WareHouseEvent {
        event: event_name,
        params: payload.params,
    };

    let event = Event::new(warehouse_event);
    process_event_impl(event, state.clone())
        .await
        .map_err(|e| {
            log::error!("Failed to process event: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to process event".to_string(),
            )
        })?;

    Ok((StatusCode::OK, "Event processed".to_string()))
}

#[utoipa::path(
    post,
    path = "/api/v1/events/bulk",
    request_body = EventBulkRequest,
    tag = "events",
    responses(
        (status = 200, description = "Bulk event success"),
        (status = 400, description = "Bulk event failed"),
        (status = 500, description = "Internal server error"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn handle_bulk_events(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(request): Json<VerifiedEventBulkRequest>,
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

    process_bulk_events_impl(request, state)
        .await
        .map_err(|e| {
            log::error!("Failed to process bulk events: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to process bulk events".to_string(),
            )
        })?;

    Ok((StatusCode::OK, "Events processed".to_string()))
}

pub async fn process_bulk_events_impl(
    request: VerifiedEventBulkRequest,
    state: Arc<AppState>,
) -> Result<(), crate::utils::error::Error> {
    let mut video_view_counts: HashMap<String, u64> = HashMap::new(); // Cache for view counts to minimize send view count to recsys
    for req_event in request.events {
        let event = Event::new(WareHouseEvent {
            event: req_event.tag(),
            params: req_event.params().to_string(),
        });

        event
            .process_video_view_count(&state, &mut video_view_counts)
            .await
            .map_err(|e| {
                log::error!("Failed to process bulk event: {e}");
                crate::utils::error::Error::Unknown("Failed to process bulk events".to_string())
            })?;
    }

    // After processing all events, we can send updated view counts to recsys-system in bulk
    state.send_bulk_view_count_to_recsys(video_view_counts);

    Ok(())
}

#[utoipa::path(
    post,
    path = "/api/v2/events/bulk",
    request_body = EventBulkRequestV2,
    tag = "events",
    responses(
        (status = 200, description = "Bulk event success"),
        (status = 400, description = "Bulk event failed"),
        (status = 500, description = "Internal server error"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn handle_bulk_events_v2(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(request): Json<VerifiedEventBulkRequestV2>,
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

    process_bulk_events_impl_v2(request, state)
        .await
        .map_err(|e| {
            log::error!("Failed to process bulk events: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to process bulk events".to_string(),
            )
        })?;

    Ok((StatusCode::OK, "Events processed".to_string()))
}

pub async fn process_bulk_events_impl_v2(
    request: VerifiedEventBulkRequestV2,
    state: Arc<AppState>,
) -> Result<(), crate::utils::error::Error> {
    let mut video_view_counts: HashMap<String, u64> = HashMap::new(); // Cache for view counts to minimize send view count to recsys
    for mut payload in request.events {
        // Extract event name and convert PascalCase to snake_case for backwards compat
        let event_name = payload
            .get("event")
            .and_then(|v| v.as_str())
            .map(to_snake_case)
            .unwrap_or_else(|| "unknown".to_string());

        if event_name == "video_started" {
            if let Value::Object(ref mut map) = payload {
                if !map.contains_key("user_id") {
                    map.insert(
                        "user_id".to_string(),
                        Value::String(request.user_id.clone()),
                    );
                }
            }
        }

        // Remove "event" field from params (old AnalyticsEventV3.params() didn't include it)
        if let Value::Object(ref mut map) = payload {
            map.remove("event");
        }

        let event = Event::new(WareHouseEvent {
            event: event_name,
            params: payload.to_string(),
        });

        event
            .process_video_view_count(&state, &mut video_view_counts)
            .await
            .map_err(|e| {
                log::error!("Failed to process bulk event: {e}");
                crate::utils::error::Error::Unknown("Failed to process bulk events".to_string())
            })?;
    }

    state.send_bulk_view_count_to_recsys(video_view_counts);

    Ok(())
}
