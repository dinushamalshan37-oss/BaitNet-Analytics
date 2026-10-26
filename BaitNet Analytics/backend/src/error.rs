use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("invalid input: {0}")]
    BadRequest(String),
    #[error("database error")]
    Database(#[from] sqlx::Error),
    #[error("cache unavailable")]
    Cache(#[from] redis::RedisError),
    #[error("internal server error")]
    Internal,
    #[error("too many requests")]
    TooManyRequests,
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error, message) = match self {
            Self::BadRequest(message) => (StatusCode::BAD_REQUEST, "bad_request", message),
            Self::Database(error) => {
                tracing::error!(%error, "database request failed");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", "Database request failed".to_owned())
            }
            Self::Cache(error) => {
                tracing::error!(%error, "cache request failed");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", "Cache request failed".to_owned())
            }
            Self::Internal => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", "Request failed".to_owned()),
            Self::TooManyRequests => (StatusCode::TOO_MANY_REQUESTS, "rate_limited", "Too many requests; please try again later".to_owned()),
        };
        (status, Json(ErrorBody { error: error.to_owned(), message })).into_response()
    }
}
