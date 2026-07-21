use axum::{routing::get, Router};
use naitik_yral_multiple_services::middleware::create_before_send;
use naitik_yral_multiple_services::state::AppState;
use naitik_yral_multiple_services::utils::error::*;
use naitik_yral_multiple_services::{api::handlers::*, events::*};
use naitik_yral_multiple_services::{config::AppConfig, ApiDoc};
use sentry_tower::{NewSentryLayer, SentryHttpLayer};
use std::env;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use utoipa_swagger_ui::SwaggerUi;

fn setup_sentry_subscriber() {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    // Configure sentry_tracing to only capture errors as events
    // and only warnings as breadcrumbs (not debug/info)
    let sentry_layer = sentry_tracing::layer().event_filter(|metadata| {
        use sentry_tracing::EventFilter;
        match *metadata.level() {
            tracing::Level::ERROR => EventFilter::Event,
            tracing::Level::WARN => EventFilter::Breadcrumb,
            _ => EventFilter::Ignore, // Ignore DEBUG, INFO, TRACE
        }
    });

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,hyper=warn,reqwest=warn,tower_http=warn".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .with(sentry_layer)
        .init();
}

async fn main_impl() -> Result<()> {
    let conf = AppConfig::load()?;

    let state = Arc::new(AppState::new(&conf).await?);

    let sentry_tower_layer = ServiceBuilder::new()
        .layer(NewSentryLayer::new_from_top())
        .layer(SentryHttpLayer::with_transaction());

    let router = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(post_event))
        .routes(routes!(handle_bulk_events))
        .routes(routes!(post_event_v2))
        .routes(routes!(handle_bulk_events_v2))
        .routes(routes!(new_ai_influencer_message))
        .routes(routes!(
            naitik_yral_multiple_services::notifications::register_device
        ))
        .routes(routes!(
            naitik_yral_multiple_services::notifications::unregister_device
        ))
        .routes(routes!(
            naitik_yral_multiple_services::notifications::send_notification
        ))
        .with_state(state.clone());

    let (router, api) = router.split_for_parts();

    let router =
        router.merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api.clone()));

    // Build the application router with all routes defined here
    let app = Router::new()
        // API routes
        .route("/healthz", get(healthz))
        .route("/authenticated_health", get(authenticated_health))
        .fallback_service(router)
        .layer(CorsLayer::permissive())
        .layer(sentry_tower_layer)
        // Add shared state
        .with_state(state.clone());

    let listener = tokio::net::TcpListener::bind(conf.bind_address)
        .await
        .map_err(Error::IO)?;

    log::info!("Server starting on {}", conf.bind_address);

    axum::serve(listener, app).await.map_err(Error::IO)?;

    Ok(())
}

fn main() {
    let _guard = sentry::init((
        "https://7767215193ccf6dd49f0348527386ef5@sentry.naitik.yral.com/8",
        sentry::ClientOptions {
            release: sentry::release_name!(),
            environment: Some(
                env::var("APP_ENV")
                    .unwrap_or_else(|_| "production".to_string())
                    .into(),
            ),
            server_name: Some(
                hostname::get()
                    .ok()
                    .and_then(|h| h.into_string().ok())
                    .unwrap_or_else(|| "unknown".to_string())
                    .into(),
            ),
            send_default_pii: true,
            traces_sample_rate: 0.01, //lower sampling for lower data accumulation.
            attach_stacktrace: true,
            auto_session_tracking: true,
            max_breadcrumbs: 100, // Store more breadcrumbs for better context
            before_send: Some(create_before_send()),
            ..Default::default()
        },
    ));

    setup_sentry_subscriber();

    log::info!("Sentry initialized successfully");

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            main_impl().await.unwrap();
        });
}
