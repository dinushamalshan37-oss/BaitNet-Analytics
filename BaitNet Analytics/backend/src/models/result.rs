use super::indicator::Indicator;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum Verdict {
    Safe,
    Suspicious,
    Malicious,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ScoreReason {
    pub signal: String,
    pub weight: i16,
    pub detail: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProviderResult {
    pub provider: String,
    pub data: Value,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RelatedIndicator {
    pub id: Uuid,
    pub kind: String,
    pub value: String,
    pub relation_type: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SearchResult {
    pub indicator: Indicator,
    pub score: u8,
    pub verdict: Verdict,
    pub reasons: Vec<ScoreReason>,
    pub providers: Vec<ProviderResult>,
    pub related: Vec<RelatedIndicator>,
}
