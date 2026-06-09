use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

use crate::{
    events::{
        types::AnalyticsEvent,
        warehouse_events::{Empty, WarehouseEvent},
    },
    state::AppState,
};
use warehouse_events::warehouse_events_server::WarehouseEvents;
pub use yral_types::delegated_identity::DelegatedIdentityWire;

pub mod warehouse_events {
    tonic::include_proto!("warehouse_events");
    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("warehouse_events_descriptor");
}

pub mod event;
pub mod push_notification;
pub mod types;

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

pub struct WarehouseEventsService {
    pub shared_state: Arc<AppState>,
}

#[tonic::async_trait]
impl WarehouseEvents for WarehouseEventsService {
    async fn send_event(
        &self,
        request: tonic::Request<WarehouseEvent>,
    ) -> Result<tonic::Response<Empty>, tonic::Status> {
        let shared_state = self.shared_state.clone();
        let mut video_view_counts: HashMap<String, u64> = HashMap::new();

        let request = request.into_inner();
        let event = event::Event::new(request);

        // process_event_impl(event, shared_state, &mut video_view_counts)
        //     .await
        //     .map_err(|e| {
        //         log::error!("Failed to process event grpc: {e}");
        //         tonic::Status::internal("Failed to process event")
        //     })?;

        Ok(tonic::Response::new(Empty {}))
    }
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
    event: String,
    params: String,
}
