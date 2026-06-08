use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

use crate::events::types::AnalyticsEvent;
pub use yral_types::delegated_identity::DelegatedIdentityWire;

pub mod events;
pub mod types;

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
