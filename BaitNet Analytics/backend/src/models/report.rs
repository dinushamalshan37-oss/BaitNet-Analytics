use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UserReport {
    pub id: Uuid,
    pub indicator_id: Uuid,
    pub kind: String,
    pub value: String,
    pub category: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
}
