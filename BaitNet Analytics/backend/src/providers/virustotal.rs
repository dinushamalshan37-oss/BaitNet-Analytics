use async_trait::async_trait;
use base64::Engine;
use reqwest::{Client, StatusCode};
use serde_json::Value;

use crate::models::indicator::{Indicator, IndicatorKind};

use super::{result, Provider, ProviderError};

pub struct VirusTotalProvider {
    client: Client,
    key: String,
}

impl VirusTotalProvider {
    pub fn new(client: Client, key: String) -> Self { Self { client, key } }
}

#[async_trait]
impl Provider for VirusTotalProvider {
    fn name(&self) -> &'static str { "virustotal" }
    fn supports(&self, kind: &IndicatorKind) -> bool {
        matches!(kind, IndicatorKind::Ipv4 | IndicatorKind::Ipv6 | IndicatorKind::Domain | IndicatorKind::Url)
    }

    async fn lookup(&self, indicator: &Indicator) -> Result<super::super::models::result::ProviderResult, ProviderError> {
        let (resource, value) = match indicator.kind {
            IndicatorKind::Ipv4 | IndicatorKind::Ipv6 => ("ip_addresses", indicator.value.clone()),
            IndicatorKind::Domain => ("domains", indicator.value.clone()),
            IndicatorKind::Url => ("urls", base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&indicator.value)),
            _ => return Err(ProviderError::Lookup("unsupported indicator".to_owned())),
        };
        let response = self.client.get(format!("https://www.virustotal.com/api/v3/{resource}/{value}"))
            .header("x-apikey", &self.key).send().await?;
        if response.status() == StatusCode::NOT_FOUND {
            return Ok(result(self.name(), serde_json::json!({ "found": false })));
        }
        let response = response.error_for_status()?;
        let data: Value = response.json().await?;
        Ok(result(self.name(), data))
    }
}
