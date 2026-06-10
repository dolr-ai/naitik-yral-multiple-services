use crate::{events::warehouse_events::WarehouseEvent, state::AppState};
use axum::{extract::State, Json};
// use http::header::CONTENT_TYPE;
use log::{debug, error};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlatEvent {
    pub event: String,
    #[serde(flatten)]
    pub params: Value,
}

#[derive(Debug)]
pub struct Event {
    pub event: WarehouseEvent,
}

impl Event {
    pub fn new(event: WarehouseEvent) -> Self {
        Self { event }
    }
}
