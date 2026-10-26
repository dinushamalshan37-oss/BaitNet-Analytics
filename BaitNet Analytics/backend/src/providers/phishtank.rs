use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;

use crate::models::indicator::{Indicator, IndicatorKind};

use super::{result, Provider, ProviderError};

pub struct PhishTankProvider { client: Client, key: String }
impl PhishTankProvider { pub fn new(client: Client, key: String) -> Self { Self { client, key } } }

#[async_trait]
impl Provider for PhishTankProvider {
    fn name(&self) -> &'static str { "phishtank" }
    fn supports(&self, kind: &IndicatorKind) -> bool { matches!(kind, IndicatorKind::Url | IndicatorKind::Domain) }
    async fn lookup(&self, indicator: &Indicator) -> Result<super::super::models::result::ProviderResult, ProviderError> {
        let response = self.client.post("https://checkurl.phishtank.com/checkurl/")
            .form(&[("url", indicator.value.as_str()), ("format", "json"), ("app_key", &self.key)])
            .send().await?.error_for_status()?;
        let data: Value = response.json().await?;
        Ok(result(self.name(), data))
    }
}
