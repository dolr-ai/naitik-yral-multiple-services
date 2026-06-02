use crate::auth::init_jwt;
use crate::auth::JwtDetails;
use crate::config::AppConfig;
use crate::dragonfly::{
    get_redis_store_ca_cert, get_redis_store_client_cert, get_redis_store_client_key,
    init_dragonfly_redis_store, DragonflyPool,
};
use crate::utils::error::{Error, Result};
use std::sync::Arc;

pub static IC_AGENT_URL: &str = "https://ic0.app";

#[derive(Clone)]
pub struct AppState {
    pub dragonfly_redis_store: Arc<DragonflyPool>,
    pub jwt_details: JwtDetails,
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
            jwt_details: init_jwt(app_config)?,
        })
    }
}
