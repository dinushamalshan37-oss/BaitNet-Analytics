use axum::{extract::{ConnectInfo, State}, Json};
use serde::Deserialize;
use std::net::SocketAddr;
use validator::Validate;

use crate::{engine::normalize::normalize, error::AppError};

use super::AppState;

#[derive(Deserialize, Validate)]
pub struct ReportRequest {
    #[validate(length(min = 1, max = 2048))]
    indicator: String,
    #[validate(length(min = 1, max = 40))]
    category: String,
    #[validate(length(min = 1, max = 2000))]
    description: String,
}

#[derive(serde::Serialize)]
pub struct ReportResponse {
    pub id: uuid::Uuid,
    pub status: &'static str,
}

pub async fn submit_report(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    Json(request): Json<ReportRequest>,
) -> Result<(axum::http::StatusCode, Json<ReportResponse>), AppError> {
    request.validate().map_err(|error| AppError::BadRequest(error.to_string()))?;
    let category = request.category.trim().to_ascii_lowercase();
    if !matches!(category.as_str(), "phishing" | "fraud" | "impersonation" | "malware" | "other") {
        return Err(AppError::BadRequest("Category must be phishing, fraud, impersonation, malware, or other".to_owned()));
    }
    let description = request.description.trim();
    if description.is_empty() || description.chars().any(char::is_control) {
        return Err(AppError::BadRequest("Description must contain readable text".to_owned()));
    }
    let indicator = normalize(&request.indicator).map_err(AppError::BadRequest)?;
    let peer = peer.ip().to_string();
    if !state.cache.allow_requests(&format!("rate:report:{peer}"), 5, 3600).await? {
        return Err(AppError::TooManyRequests);
    }

    let indicator_id: uuid::Uuid = sqlx::query_scalar(
        "INSERT INTO indicators (kind, value) VALUES ($1, $2) ON CONFLICT (kind, value) DO UPDATE SET last_seen = now() RETURNING id",
    ).bind(indicator.kind.to_string()).bind(&indicator.value).fetch_one(&state.pool).await?;
    let row = sqlx::query_as::<_, (uuid::Uuid,)>(
        "INSERT INTO user_reports (indicator_id, category, description) VALUES ($1, $2, $3) RETURNING id",
    ).bind(indicator_id).bind(category).bind(description).fetch_one(&state.pool).await?;
    state.cache.delete(&format!("search:{}:{}", indicator.kind, indicator.value)).await?;
    Ok((axum::http::StatusCode::CREATED, Json(ReportResponse { id: row.0, status: "received" })))
}
