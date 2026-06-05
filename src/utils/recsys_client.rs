use std::{collections::HashMap, sync::Arc};

use crate::utils::error::Result;
use crate::{consts::RECSYS_ENDPOINT, dragonfly::DragonflyPool};
use hmac::{Hmac, Mac};
use redis::AsyncCommands;
use reqwest::{Client, Url};
use sha2::Sha256;
type HmacSha256 = Hmac<Sha256>;

#[derive(Clone)]
pub struct RecsysClient {
    pub client: Client,
    pub url: Url,
}

impl Default for RecsysClient {
    fn default() -> Self {
        Self::new()
    }
}

impl RecsysClient {
    pub fn new() -> Self {
        let client = Client::new();
        let url = Url::parse(RECSYS_ENDPOINT).expect("Invalid recsys endpoint URL");

        let secret = std::env::var("RECSYS_INTERNAL_CALL_SECRET_KEY").unwrap_or_default();
        if secret.is_empty() {
            log::error!("RECSYS_INTERNAL_CALL_SECRET_KEY is not set");
        } else {
            log::info!("RECSYS_INTERNAL_CALL_SECRET_KEY is set");
        }
        Self { client, url }
    }

    pub fn send_bulk_view_count_to_recsys(&self, video_view_counts: HashMap<String, u64>) {
        // check if the map is empty, if yes then skip sending request to recsys-system
        if video_view_counts.is_empty() {
            log::info!("No view counts to send to recsys-system, skipping request");
            return;
        }

        log::info!(
            "Sending bulk view counts for {} videos to recsys-system",
            video_view_counts.len()
        );

        let req_client = self.client.clone();
        let path = self.url.path().to_string();
        let secret = std::env::var("RECSYS_INTERNAL_CALL_SECRET_KEY").unwrap_or_default();

        tokio::spawn(async move {
            let payload = serde_json::json!(video_view_counts
                .iter()
                .map(|(video_id, count)| {
                    serde_json::json!({
                        "video_id": video_id,
                        "total_count_all": count,
                    })
                })
                .collect::<Vec<_>>());

            let payload_str = payload.to_string();
            let timestamp = chrono::Utc::now().timestamp().to_string();

            let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
                .expect("HMAC can take key of any size");
            mac.update(timestamp.as_bytes());
            mac.update(b"\n");
            mac.update(b"POST");
            mac.update(b"\n");
            mac.update(path.as_bytes());
            mac.update(b"\n");
            mac.update(payload_str.as_bytes());
            let signature = hex::encode(mac.finalize().into_bytes());

            match req_client
                .post(RECSYS_ENDPOINT)
                .header("x-internal-timestamp", timestamp)
                .header("x-internal-signature", signature)
                .header("Content-Type", "application/json")
                .body(payload_str)
                .send()
                .await
            {
                Ok(response) => {
                    if response.status().is_success() {
                        log::info!("Successfully sent bulk view counts to recsys-system");
                    } else {
                        log::error!(
                            "Failed to send bulk view counts to recsys-system: HTTP {}",
                            response.status()
                        );
                    }
                }
                Err(e) => {
                    log::error!("Error sending bulk view counts to recsys-system: {}", e);
                }
            }
        });
    }
}

pub async fn get_total_count_all(redis_pool: &Arc<DragonflyPool>, video_id: &str) -> Result<u64> {
    let video_hash_key = format!("impressions:rewards:video:{}", video_id);
    let count: Option<String> = redis_pool
        .execute_with_retry(|mut conn| {
            let key = video_hash_key.clone();
            async move { conn.hget(&key, "total_count_all").await }
        })
        .await?;

    Ok(count.and_then(|s| s.parse().ok()).unwrap_or(0))
}

pub async fn get_view_counts_for_videos(
    redis_pool: &Arc<DragonflyPool>,
    video_ids: &[String],
) -> Result<HashMap<String, u64>> {
    if video_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let keys: Vec<String> = video_ids
        .iter()
        .map(|id| format!("impressions:rewards:video:{}", id))
        .collect();

    let counts: Vec<Option<String>> = redis_pool
        .execute_with_retry(|mut conn| {
            let keys = keys.clone();
            async move {
                let mut pipe = redis::pipe();
                for key in keys {
                    pipe.hget(key, "total_count_all");
                }
                pipe.query_async(&mut conn).await
            }
        })
        .await?;

    let mut view_counts = HashMap::with_capacity(video_ids.len());
    for (video_id, count_str) in video_ids.iter().zip(counts) {
        let count = count_str.and_then(|s| s.parse().ok()).unwrap_or(0);
        view_counts.insert(video_id.clone(), count);
    }

    Ok(view_counts)
}
