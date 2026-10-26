use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;

use crate::models::indicator::{Indicator, IndicatorKind};

use super::{result, Provider, ProviderError};

pub struct ShodanProvider { client: Client, key: String }
impl ShodanProvider { pub fn new(client: Client, key: String) -> Self { Self { client, key } } }

#[async_trait]
impl Provider for ShodanProvider {
    fn name(&self) -> &'static str { "shodan" }
    fn supports(&self, kind: &IndicatorKind) -> bool { matches!(kind, IndicatorKind::Ipv4 | IndicatorKind::Ipv6) }
    async fn lookup(&self, indicator: &Indicator) -> Result<super::super::models::result::ProviderResult, ProviderError> {
        let response = self.client.get(format!("https://api.shodan.io/shodan/host/{}?key={}", indicator.value, self.key)).send().await?.error_for_status()?;
        let data: Value = response.json().await?;
        Ok(result(self.name(), data))
    }
}
