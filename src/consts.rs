use crate::auth::Claims;
use once_cell::sync::Lazy;
use reqwest::Url;

pub static CLAIMS: Lazy<Claims> = Lazy::new(|| Claims {
    sub: "naitik-yral-multiple-services".to_string(),
    company: "gobazzinga".to_string(),
    exp: 4934769340, // TODO: To be changed later when expiring tokens periodically
});

pub const RECSYS_ENDPOINT: &str =
    "https://recsys-influencer-feed.ansuman.yral.com/api/v1/internal/feed-recsys/view-counts";

pub static YRAL_METADATA_URL: Lazy<Url> =
    Lazy::new(|| Url::parse("https://metadata.yral.com/").unwrap());

