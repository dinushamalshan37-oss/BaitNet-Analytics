use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;

use crate::models::indicator::{Indicator, IndicatorKind};

use super::{result, Provider, ProviderError};

pub struct WhoisProvider { client: Client }
impl WhoisProvider { pub fn new(client: Client) -> Self { Self { client } } }

#[async_trait]
impl Provider for WhoisProvider {
    fn name(&self) -> &'static str { "rdap" }
    fn supports(&self, kind: &IndicatorKind) -> bool { matches!(kind, IndicatorKind::Domain | IndicatorKind::Url) }
    async fn lookup(&self, indicator: &Indicator) -> Result<super::super::models::result::ProviderResult, ProviderError> {
        let domain = url::Url::parse(&format!("http://{}", indicator.value)).ok()
            .and_then(|url| url.host_str().map(str::to_owned)).unwrap_or_else(|| indicator.value.clone());
        let response = self.client.get(format!("https://rdap.org/domain/{domain}"))
            .header("accept", "application/rdap+json").send().await?.error_for_status()?;
        let data: Value = response.json().await?;
        Ok(result(self.name(), data))
    }
}
