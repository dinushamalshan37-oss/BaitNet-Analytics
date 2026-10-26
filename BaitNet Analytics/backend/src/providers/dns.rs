use async_trait::async_trait;
use hickory_resolver::TokioAsyncResolver;
use serde_json::{json, Value};

use crate::models::indicator::{Indicator, IndicatorKind};

use super::{result, Provider, ProviderError};

pub struct DnsProvider {
    resolver: TokioAsyncResolver,
}

impl DnsProvider {
    pub fn new(resolver: TokioAsyncResolver) -> Self {
        Self { resolver }
    }
}

#[async_trait]
impl Provider for DnsProvider {
    fn name(&self) -> &'static str { "dns" }

    fn supports(&self, kind: &IndicatorKind) -> bool {
        matches!(kind, IndicatorKind::Domain | IndicatorKind::Url)
    }

    async fn lookup(&self, indicator: &Indicator) -> Result<super::super::models::result::ProviderResult, ProviderError> {
        let host = url::Url::parse(&format!("http://{}", indicator.value))
            .map(|url| url.host_str().unwrap_or_default().to_owned())
            .unwrap_or_else(|_| indicator.value.clone());
        let lookup_ip = self.resolver.lookup_ip(&host).await;
        let addresses: Vec<String> = lookup_ip
            .map_err(|error| ProviderError::Lookup(error.to_string()))?
            .iter()
            .map(|ip| ip.to_string())
            .collect();
        let mx = self.resolver.mx_lookup(&host).await.map(|lookup| lookup.iter().map(|record| record.exchange().to_utf8()).collect::<Vec<_>>()).unwrap_or_default();
        let ns = self.resolver.ns_lookup(&host).await.map(|lookup| lookup.iter().map(|record| record.to_utf8()).collect::<Vec<_>>()).unwrap_or_default();
        let data: Value = json!({ "addresses": addresses, "mx": mx, "nameservers": ns });
        Ok(result(self.name(), data))
    }
}
