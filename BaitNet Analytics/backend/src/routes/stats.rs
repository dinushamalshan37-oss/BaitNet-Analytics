use axum::{extract::State, Json};
use serde_json::{json, Value};

use crate::error::AppError;

use super::AppState;

pub async fn stats(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    let by_type = sqlx::query_as::<_, (String, i64)>("SELECT kind, count(*) FROM indicators GROUP BY kind ORDER BY kind")
        .fetch_all(&state.pool).await?;
    let by_verdict = sqlx::query_as::<_, (String, i64)>("SELECT verdict, count(*) FROM indicators GROUP BY verdict ORDER BY verdict")
        .fetch_all(&state.pool).await?;
    let by_day = sqlx::query_as::<_, (chrono::NaiveDate, i64)>(
        "SELECT date_trunc('day', first_seen)::date AS day, count(*) FROM indicators WHERE first_seen >= now() - interval '30 days' GROUP BY day ORDER BY day",
    ).fetch_all(&state.pool).await?;
    Ok(Json(json!({
        "by_type": by_type.into_iter().map(|(kind, count)| json!({"kind":kind,"count":count})).collect::<Vec<_>>(),
        "by_verdict": by_verdict.into_iter().map(|(verdict, count)| json!({"verdict":verdict,"count":count})).collect::<Vec<_>>(),
        "by_day": by_day.into_iter().map(|(day, count)| json!({"day":day,"count":count})).collect::<Vec<_>>()
    })))
}
