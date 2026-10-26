use async_trait::async_trait;
use reqwest::{header, Client};
use serde_json::Value;
use url::form_urlencoded;

use crate::models::indicator::{Indicator, IndicatorKind};

use super::{result, Provider, ProviderError};

pub struct HibpProvider { client: Client, key: String }
impl HibpProvider { pub fn new(client: Client, key: String) -> Self { Self { client, key } } }

#[async_trait]
impl Provider for HibpProvider {
    fn name(&self) -> &'static str { "hibp" }
    fn supports(&self, kind: &IndicatorKind) -> bool { matches!(kind, IndicatorKind::Email) }
    async fn lookup(&self, indicator: &Indicator) -> Result<super::super::models::result::ProviderResult, ProviderError> {
        let account: String = form_urlencoded::byte_serialize(indicator.value.as_bytes()).collect();
        let response = self.client.get(format!("https://haveibeenpwned.com/api/v3/breachedaccount/{account}?truncateResponse=true"))
            .header(header::USER_AGENT, "OSINT-Tracker/0.1")
            .header("hibp-api-key", &self.key).send().await?;
        if response.status().as_u16() == 404 {
            return Ok(result(self.name(), serde_json::json!({ "breached": false, "breaches": [] })));
        }
        let response = response.error_for_status()?;
        let breaches: Vec<Value> = response.json().await?;
        Ok(result(self.name(), serde_json::json!({ "breached": !breaches.is_empty(), "breaches": breaches })))
    }
}
