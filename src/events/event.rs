use crate::utils::error::Result;
use crate::{events::types::VideoDurationWatchedPayloadV2, utils::error::Error};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlatEvent {
    pub event: String,
    #[serde(flatten)]
    pub params: Value,
}

#[derive(Debug, Clone)]
pub struct WareHouseEvent {
    pub event: String,
    pub params: String,
}

#[derive(Debug)]
pub struct Event {
    pub event: WareHouseEvent,
}

impl Event {
    pub fn new(event: WareHouseEvent) -> Self {
        Self { event }
    }

    pub async fn process_video_view_count(
        &self,
        app_state: &crate::state::AppState,
        video_view_counts: &mut std::collections::HashMap<String, u64>,
    ) -> Result<()> {
        if self.event.event != "video_duration_watched" {
            return Ok(());
        }

        let params: Result<VideoDurationWatchedPayloadV2, _> =
            serde_json::from_str(&self.event.params);

        let params = match params {
            Ok(p) => p,
            Err(e) => {
                log::error!("Failed to parse video_duration_watched params for rewards: {e:?}");
                return Err(Error::Unknown(format!(
                    "Failed to parse video_duration_watched params for rewards: {e:?}"
                )));
            }
        };

        let app_state_arc = std::sync::Arc::new(app_state.clone());
        let video_id = params.video_id.as_ref().ok_or_else(|| {
            Error::Unknown("video_id is missing in video_duration_watched event".into())
        })?;

        let count = crate::events::utils::get_total_view_count(&app_state_arc, video_id)
            .await
            .unwrap_or(0);
        video_view_counts.insert(video_id.clone(), count);
        Ok(())
    }
}
