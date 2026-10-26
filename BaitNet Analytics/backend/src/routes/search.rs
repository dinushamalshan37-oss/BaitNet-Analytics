use axum::{extract::{Query, State}, Json};
use serde::Deserialize;
use validator::Validate;

use crate::{engine::orchestrator, error::AppError, models::result::SearchResult};

use super::AppState;

#[derive(Deserialize, Validate)]
pub struct SearchQuery {
    #[validate(length(min = 1, max = 2048))]
    q: Option<String>,
}

pub async fn search(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<SearchResult>, AppError> {
    query.validate().map_err(|error| AppError::BadRequest(error.to_string()))?;
    let value = query.q.ok_or_else(|| AppError::BadRequest("Query parameter q is required".to_owned()))?;
    let result = orchestrator::search(&state.pool, &state.cache, &state.config, &value).await?;
    Ok(Json(result))
}
