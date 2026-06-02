use once_cell::sync::Lazy;

use crate::auth::Claims;

pub static CLAIMS: Lazy<Claims> = Lazy::new(|| Claims {
    sub: "off-chain-agent".to_string(),
    company: "gobazzinga".to_string(),
    exp: 317125598072, // TODO: To be changed later when expiring tokens periodically
});

pub const RECSYS_ENDPOINT: &str =
    "https://recsys-influencer-feed.ansuman.yral.com/api/v1/internal/feed-recsys/view-counts";
