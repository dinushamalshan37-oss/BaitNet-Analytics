mod cache;
mod config;
mod db;
mod error;
mod engine;
mod models;
mod providers;
mod routes;

use std::net::SocketAddr;

use axum::{routing::get, Router};
use config::Config;
use sqlx::postgres::PgPoolOptions;
use tower::limit::ConcurrencyLimitLayer;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::path::Path::new(".env").exists() {
        dotenvy::dotenv()?;
    }
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    let config = Config::from_env()?;
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;
    db::migrate(&pool).await?;
    let cache = cache::Cache::connect(&config.redis_url, config.redis_ttl_seconds).await?;

    let cors = CorsLayer::new()
        .allow_origin(config.frontend_origin.parse::<axum::http::HeaderValue>()?)
        .allow_methods([axum::http::Method::GET, axum::http::Method::POST])
        .allow_headers([axum::http::header::CONTENT_TYPE]);

    let state = routes::AppState { pool, cache, config };
    let backend_port = state.config.backend_port;
    let app = Router::new()
        .route("/health", get(routes::health::health))
        .nest("/api", routes::api_router())
        .with_state(state.clone())
        .layer(ConcurrencyLimitLayer::new(100))
        .layer(axum::middleware::from_fn_with_state(state, routes::rate_limit))
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    let address = SocketAddr::from(([0, 0, 0, 0], backend_port));
    let listener = tokio::net::TcpListener::bind(address).await?;
    tracing::info!(%address, "OSINT tracker API listening");
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await?;
    Ok(())
}
