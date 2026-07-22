use candid::Principal;
use multi_service_types::SendNotificationReq;
use serde_json::Value;

use crate::{events::types::deserialize_event_payload, state::AppState};

const MULTI_SERVICE_DEFAULT_API_URL: &str = "https://multi-service.naitik.yral.com";

#[derive(Clone)]
pub struct NotificationClient {
    api_key: String,
}

impl NotificationClient {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }

    pub async fn send_notification(&self, data: SendNotificationReq, user_id: Principal) {
        let client = reqwest::Client::new();
        let url = format!(
            "{}/api/v1/notifications/{}/send",
            MULTI_SERVICE_DEFAULT_API_URL,
            user_id.to_text()
        );

        let res = client
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(&data)
            .send()
            .await;

        if let Err(e) = res {
            log::error!("Error sending notification: {e:?}");
        }
    }
}

const NOTIFICATION_EVENTS: &[&str] = &[
    "video_upload_successful",
    "like_video",
    "video_approved",
    "video_disapproved",
    "tournament_started",
    "tournament_ended_winner",
    "reward_earned",
    "follow_user",
    "new_ai_influencer_message",
];

pub async fn dispatch_notif(
    event_type: &str,
    params: Value,
    app_state: &AppState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if !NOTIFICATION_EVENTS.contains(&event_type) {
        return Ok(());
    }

    let event = deserialize_event_payload(event_type, params)?;
    event.send_notification(app_state).await;
    Ok(())
}
