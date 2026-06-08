pub mod api;
pub mod auth;
pub mod config;
pub mod consts;
pub mod dragonfly;
pub mod events;
pub mod middleware;
pub mod state;
pub mod types;
pub mod utils;

use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use std::sync::Arc;
use utils::error::Error as ApiError;
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
        api::handlers::send_view_count_to_recsys,
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
struct ApiDoc;

pub async fn openapi_spec() -> impl IntoResponse {
    Json(ApiDoc::openapi())
}

pub async fn get_swagger(Path(tail): Path<String>) -> Result<Response, ApiError> {
    if tail == "swagger.json" {
        let spec = ApiDoc::openapi()
            .to_json()
            .map_err(|err| ApiError::SwaggerUi(err.to_string()))?;
        return Ok((StatusCode::OK, [("content-type", "application/json")], spec).into_response());
    }

    let config =
        Arc::new(utoipa_swagger_ui::Config::new(["/explorer/swagger.json"]).use_base_layout());

    match utoipa_swagger_ui::serve(&tail, config.clone())
        .map_err(|err| ApiError::SwaggerUi(err.to_string()))?
    {
        None => Err(ApiError::SwaggerUi(format!("path not found: {}", tail))),
        Some(file) => Ok((
            StatusCode::OK,
            [("content-type", file.content_type)],
            file.bytes.to_vec(),
        )
            .into_response()),
    }
}

pub async fn get_swagger_root() -> Result<Response, ApiError> {
    let config =
        Arc::new(utoipa_swagger_ui::Config::new(["/explorer/swagger.json"]).use_base_layout());

    match utoipa_swagger_ui::serve("index.html", config.clone())
        .map_err(|err| ApiError::SwaggerUi(err.to_string()))?
    {
        None => Err(ApiError::SwaggerUi("path not found".to_string())),
        Some(file) => Ok((
            StatusCode::OK,
            [("content-type", file.content_type)],
            file.bytes.to_vec(),
        )
            .into_response()),
    }
}
