pub mod api;
pub mod auth;
pub mod config;
pub mod consts;
pub mod dragonfly;
pub mod events;
pub mod metadata_client;
pub mod metadata_types;
pub mod metrics;
pub mod middleware;
pub mod state;
pub mod types;
pub mod utils;
pub mod yral_identity;

use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};

struct BearerAuth;

impl Modify for BearerAuth {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            )
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        api::handlers::healthz,
        api::handlers::authenticated_health,
        events::post_event,
        events::post_event_v2,
        events::handle_bulk_events,
        events::handle_bulk_events_v2,
    ),
    components(
        schemas()
    ),
    modifiers(&BearerAuth),
    tags(
        (name = "Health", description = "Health check"),
    ),
    info(
        title = "YRAL multiple services API",
        version = "1.0.0",
        description = "API for YRAL multiple services",
        contact(
            name = "YRAL Team",
            url = "https://yral.com"
        )
    )
)]
pub struct ApiDoc;
