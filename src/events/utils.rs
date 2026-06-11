use crate::state::AppState;
use crate::utils::error::Result;
use redis::AsyncCommands;

pub async fn get_total_view_count(app_state: &AppState, video_id: &str) -> Result<u64> {
    let video_hash_key = format!("impressions:rewards:video:{}", video_id);
    let count: Option<String> = app_state
        .dragonfly_redis_store
        .execute_with_retry(|mut conn| {
            let key = video_hash_key.clone();
            async move { conn.hget(&key, "total_count_all").await }
        })
        .await?;

    Ok(count.and_then(|s| s.parse().ok()).unwrap_or(0))
}
