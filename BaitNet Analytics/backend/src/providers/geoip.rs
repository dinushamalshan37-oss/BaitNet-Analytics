use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;
use std::net::IpAddr;

use crate::models::indicator::{Indicator, IndicatorKind};

use super::{public_ip, result, Provider, ProviderError};

pub struct GeoIpProvider { client: Client }
impl GeoIpProvider { pub fn new(client: Client) -> Self { Self { client } } }

#[async_trait]
impl Provider for GeoIpProvider {
    fn name(&self) -> &'static str { "geoip" }
    fn supports(&self, kind: &IndicatorKind) -> bool { matches!(kind, IndicatorKind::Ipv4 | IndicatorKind::Ipv6) }
    async fn lookup(&self, indicator: &Indicator) -> Result<super::super::models::result::ProviderResult, ProviderError> {
        let ip = indicator.value.parse::<IpAddr>().map_err(|error| ProviderError::Lookup(error.to_string()))?;
        if !public_ip(ip) {
            return Ok(result(self.name(), serde_json::json!({ "available": false, "reason": "non_public_address" })));
        }
        let response = self.client.get(format!("https://ipwho.is/{ip}?fields=success,message,country,region,city,latitude,longitude,connection"))
            .send().await?.error_for_status()?;
        let data: Value = response.json().await?;
        Ok(result(self.name(), data))
    }
}
