mod config;
mod error;
mod handlers;
mod models;
mod time;

#[cfg(feature = "mqtt")]
mod mqtt;

use axum::{
    extract::Request,
    http::{header, HeaderValue, Method},
    middleware::{self, Next},
    response::Response,
    routing::get,
    Extension, Router,
};
use config::Config;
use std::sync::Arc;
use std::time::Duration;
use time::ChronyTracker;
use tokio::signal;
use tower_http::{
    cors::{Any, CorsLayer},
    limit::RequestBodyLimitLayer,
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing::info;

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = Config::from_env()?;
    config.validate()?;

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("time_api={}", config.log_level).into()),
        )
        .init();

    info!("Starting Time API v0.1.0");
    info!("Listening on {}:{}", config.http.host, config.http.port);

    // Initialize chrony tracker
    let chrony_tracker = Arc::new(ChronyTracker::new());

    // Initialize MQTT if configured
    #[cfg(feature = "mqtt")]
    if let Some(ref mqtt_config) = config.mqtt {
        match mqtt::MqttClient::new(mqtt_config) {
            Ok(mqtt_client) => {
                let mqtt_client = Arc::new(mqtt_client);
                info!("MQTT client initialized, base topic: {}", mqtt_client.base_topic());

                // Start PPS publishing task
                let pps_client = mqtt_client.clone();
                tokio::spawn(async move {
                    mqtt::pps::start_pps_task(pps_client).await;
                });

                // Start health publishing task
                let health_client = mqtt_client.clone();
                let health_chrony = chrony_tracker.clone();
                tokio::spawn(async move {
                    mqtt::health::start_health_task(health_client, health_chrony).await;
                });

                info!("MQTT PPS and health publishing tasks started");
            }
            Err(e) => {
                tracing::error!("Failed to initialize MQTT client: {}", e);
                tracing::warn!("Continuing without MQTT support");
            }
        }
    }

    // Build CORS layer
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE, header::ACCEPT]);

    // Build router with layers applied in correct order
    let app = Router::new()
        .route("/", get(handlers::root))
        .route("/times", get(handlers::times))
        .route("/health", get(handlers::health))
        .route("/ready", get(handlers::ready))
        .layer(Extension(chrony_tracker.clone()))
        .layer(middleware::from_fn(security_headers))
        .layer(RequestBodyLimitLayer::new(1024 * 10)) // 10KB max
        .layer(TimeoutLayer::new(Duration::from_secs(5)))
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    // Create bind address
    let addr = format!("{}:{}", config.http.host, config.http.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    info!("Server started successfully on {}", addr);

    // Run server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("Server shutdown complete");
    Ok(())
}

/// Add security headers to all responses
async fn security_headers(req: Request, next: Next) -> Response {
    let mut response = next.run(req).await;

    let headers = response.headers_mut();

    // HSTS: Force HTTPS for 1 year
    headers.insert(
        header::STRICT_TRANSPORT_SECURITY,
        HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    );

    // Prevent MIME sniffing
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );

    // Prevent clickjacking
    headers.insert(
        header::X_FRAME_OPTIONS,
        HeaderValue::from_static("DENY"),
    );

    // Referrer policy
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("no-referrer"),
    );

    // Permissions policy (formerly Feature-Policy)
    headers.insert(
        header::HeaderName::from_static("permissions-policy"),
        HeaderValue::from_static("geolocation=(), microphone=(), camera=()"),
    );

    // Content Security Policy
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static("default-src 'self'; style-src 'unsafe-inline'"),
    );

    response
}

/// Wait for shutdown signal (SIGTERM or SIGINT)
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received SIGINT, shutting down gracefully...");
        },
        _ = terminate => {
            info!("Received SIGTERM, shutting down gracefully...");
        },
    }
}
