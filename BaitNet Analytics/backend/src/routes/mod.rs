pub mod feed;
pub mod health;
pub mod report;
pub mod search;
pub mod stats;

use std::net::SocketAddr;

use axum::{
    extract::{ConnectInfo, Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use sqlx::PgPool;

use crate::{cache::Cache, config::Config, error::AppError};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub cache: Cache,
    pub config: Config,
}

pub fn api_router() -> Router<AppState> {
    Router::new()
        .route("/search", get(search::search))
        .route("/feed", get(feed::feed))
        .route("/report", post(report::submit_report))
        .route("/stats", get(stats::stats))
        .layer(axum::extract::DefaultBodyLimit::max(16 * 1024))
}

pub async fn rate_limit(State(state): State<AppState>, request: Request, next: Next) -> Response {
    let peer = request.extensions().get::<ConnectInfo<SocketAddr>>()
        .map(|info| info.0.ip().to_string())
        .unwrap_or_else(|| "unknown".to_owned());
    match state.cache.allow_requests(&format!("rate:global:{peer}"), 120, 60).await {
        Ok(true) => next.run(request).await,
        Ok(false) => AppError::TooManyRequests.into_response(),
        Err(error) => {
            tracing::error!(%error, "global rate limiter unavailable");
            AppError::Cache(error).into_response()
        }
    }
}

#[derive(Serialize)]
pub struct HealthResponse {
    status: &'static str,
    database: &'static str,
    cache: &'static str,
}

pub fn health_response(database_ok: bool, cache_ok: bool) -> (StatusCode, Json<HealthResponse>) {
    let healthy = database_ok && cache_ok;
    (
        if healthy { StatusCode::OK } else { StatusCode::SERVICE_UNAVAILABLE },
        Json(HealthResponse {
            status: if healthy { "ok" } else { "degraded" },
            database: if database_ok { "ok" } else { "unavailable" },
            cache: if cache_ok { "ok" } else { "unavailable" },
        }),
    )
}
