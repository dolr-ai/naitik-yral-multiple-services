use candid::Principal;

use crate::config::AppConfig;
use crate::consts::{RECSYS_ENDPOINT, YRAL_METADATA_URL};
use crate::dragonfly::{
    get_redis_store_ca_cert, get_redis_store_client_cert, get_redis_store_client_key,
    init_dragonfly_redis_store, DragonflyPool,
};
use crate::metadata_client::MetadataClient;
use crate::utils::error::Result;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
type HmacSha256 = Hmac<Sha256>;

pub static IC_AGENT_URL: &str = "https://ic0.app";

#[derive(Clone)]
pub struct AppState {
    pub dragonfly_redis_store: Arc<DragonflyPool>,
    pub jwt_details: crate::auth::JwtDetails,
    pub recsys_client: crate::utils::recsys_client::RecsysClient,
    pub notification_client: crate::events::push_notification::NotificationClient,
    pub yral_metadata_client: MetadataClient<true>,
}

impl AppState {
    pub async fn new(app_config: &AppConfig) -> Result<Self> {
        let redis_store_ca_cert_bytes = get_redis_store_ca_cert()?;
        let redis_store_client_cert_bytes = get_redis_store_client_cert()?;
        let redis_store_client_key_bytes = get_redis_store_client_key()?;

        Ok(AppState {
            dragonfly_redis_store: init_dragonfly_redis_store(
                redis_store_ca_cert_bytes,
                redis_store_client_cert_bytes,
                redis_store_client_key_bytes,
            )
            .await?,
            jwt_details: crate::auth::init_jwt(app_config)?,
            recsys_client: crate::utils::recsys_client::RecsysClient::new(),
            notification_client: crate::events::push_notification::NotificationClient::new(
                env::var("YRAL_METADATA_NOTIFICATION_API_KEY").unwrap_or_default(),
            ),
            yral_metadata_client: init_yral_metadata_client(app_config),
        })
    }

    pub async fn get_individual_canister_by_user_principal(
        &self,
        user_principal: Principal,
    ) -> std::result::Result<Principal, crate::utils::error::Error> {
        let meta = self
            .yral_metadata_client
            .get_user_metadata_v2(user_principal.to_string())
            .await
            .map_err(|e| crate::utils::error::Error::Unknown(e.to_string()))?;

        match meta {
            Some(meta) => Ok(meta.user_canister_id),
            None => Err(crate::utils::error::Error::Unknown(
                "user metadata does not exist in yral_metadata_service".to_string(),
            )),
        }
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

        let req_client = self.recsys_client.client.clone();
        let path = self.recsys_client.url.path().to_string();
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

pub fn init_yral_metadata_client(conf: &AppConfig) -> MetadataClient<true> {
    MetadataClient::with_base_url(YRAL_METADATA_URL.clone())
        .with_jwt_token(conf.yral_metadata_token.clone())
}
