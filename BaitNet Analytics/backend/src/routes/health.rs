use axum::{extract::State, Json};

use super::{health_response, AppState};

pub async fn health(State(state): State<AppState>) -> (axum::http::StatusCode, Json<super::HealthResponse>) {
    let database_ok = sqlx::query("SELECT 1").execute(&state.pool).await.is_ok();
    let cache_ok = state.cache.ping().await.is_ok();
    health_response(database_ok, cache_ok)
}
