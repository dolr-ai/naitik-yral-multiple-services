use candid::Principal;

use crate::config::AppConfig;
use crate::consts::YRAL_METADATA_URL;
use crate::dragonfly::{
    get_redis_store_ca_cert, get_redis_store_client_cert, get_redis_store_client_key,
    init_dragonfly_redis_store, DragonflyPool,
};
use crate::utils::error::Result;
use std::env;
use std::sync::Arc;
use yral_metadata_client::MetadataClient;

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
            yral_metadata_client: init_yral_metadata_client(&app_config),
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
}

pub fn init_yral_metadata_client(conf: &AppConfig) -> MetadataClient<true> {
    MetadataClient::with_base_url(YRAL_METADATA_URL.clone())
        .with_jwt_token(conf.yral_metadata_token.clone())
}
