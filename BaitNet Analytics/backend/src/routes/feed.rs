use axum::{extract::{Query, State}, Json};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use validator::Validate;

use crate::error::AppError;

use super::AppState;

#[derive(Deserialize, Validate)]
pub struct FeedQuery {
    #[validate(range(min = 1, max = 100))]
    limit: Option<i64>,
}

#[derive(Serialize)]
pub struct FeedResponse {
    items: Vec<Value>,
}

pub async fn feed(State(state): State<AppState>, Query(query): Query<FeedQuery>) -> Result<Json<FeedResponse>, AppError> {
    query.validate().map_err(|error| AppError::BadRequest(error.to_string()))?;
    let limit = query.limit.unwrap_or(30);
    let indicators = sqlx::query_as::<_, (uuid::Uuid, String, String, i16, String, chrono::DateTime<chrono::Utc>)>(
        "SELECT id, kind, value, risk_score, verdict, last_seen FROM indicators WHERE risk_score >= 35 ORDER BY last_seen DESC LIMIT $1",
    ).bind(limit).fetch_all(&state.pool).await?;
    let reports = sqlx::query_as::<_, (uuid::Uuid, String, String, String, String, chrono::DateTime<chrono::Utc>)>(
        "SELECT r.id, i.kind, i.value, r.category, r.description, r.created_at FROM user_reports r JOIN indicators i ON i.id = r.indicator_id ORDER BY r.created_at DESC LIMIT $1",
    ).bind(limit).fetch_all(&state.pool).await?;

    let mut items: Vec<Value> = indicators.into_iter().map(|(id, kind, value, score, verdict, created_at)| {
        json!({ "id": id, "type": "indicator", "kind": kind, "value": mask_indicator(&kind, &value), "score": score, "verdict": verdict, "created_at": created_at })
    }).collect();
    items.extend(reports.into_iter().map(|(id, kind, value, category, description, created_at)| {
        json!({ "id": id, "type": "report", "kind": kind, "value": mask_indicator(&kind, &value), "category": category, "description": redact_description(&description), "created_at": created_at })
    }));
    items.sort_by(|left, right| right["created_at"].as_str().cmp(&left["created_at"].as_str()));
    items.truncate(limit as usize);
    Ok(Json(FeedResponse { items }))
}

fn mask_indicator(kind: &str, value: &str) -> String {
    match kind {
        "email" => value.split_once('@').map(|(local, domain)| format!("{}***@{domain}", local.chars().take(2).collect::<String>())).unwrap_or_else(|| "[redacted]".to_owned()),
        "phone" => {
            let digits: String = value.chars().filter(char::is_ascii_digit).collect();
            format!("***{}", digits.chars().rev().take(4).collect::<String>().chars().rev().collect::<String>())
        }
        _ => value.to_owned(),
    }
}

fn redact_description(description: &str) -> String {
    let email = Regex::new(r"(?i)\b[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}\b").expect("static regex");
    let phone = Regex::new(r"(?x)(?<![A-Za-z0-9])\+?[0-9][0-9 .()\-]{7,}[0-9](?![A-Za-z0-9])").expect("static regex");
    phone.replace_all(&email.replace_all(description, "[email redacted]"), "[phone redacted]").into_owned()
}
