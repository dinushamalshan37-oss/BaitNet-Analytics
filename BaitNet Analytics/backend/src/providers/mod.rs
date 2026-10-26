pub mod dns;
pub mod geoip;
pub mod hibp;
pub mod phishtank;
pub mod scraper;
pub mod shodan;
pub mod virustotal;
pub mod whois;

use std::{net::IpAddr, time::Duration};

use async_trait::async_trait;
use hickory_resolver::TokioAsyncResolver;
use reqwest::Client;
use serde_json::Value;
use thiserror::Error;

use crate::{config::Config, models::{indicator::{Indicator, IndicatorKind}, result::ProviderResult}};

#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("provider request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("provider returned an unexpected response: {0}")]
    Response(String),
    #[error("provider lookup failed: {0}")]
    Lookup(String),
}

#[async_trait]
pub trait Provider: Send + Sync {
    fn name(&self) -> &'static str;
    fn supports(&self, kind: &IndicatorKind) -> bool;
    async fn lookup(&self, indicator: &Indicator) -> Result<ProviderResult, ProviderError>;
}

pub struct ProviderContext {
    pub client: Client,
    pub resolver: TokioAsyncResolver,
}

impl ProviderContext {
    pub fn new() -> Result<Self, ProviderError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .user_agent("OSINT-Tracker/0.1 (public defensive reputation lookup)")
            .build()?;
        let resolver = TokioAsyncResolver::tokio_from_system_conf()
            .map_err(|error| ProviderError::Lookup(format!("DNS resolver configuration failed: {error}")))?;
        Ok(Self { client, resolver })
    }
}

pub fn create_providers(context: &ProviderContext, config: &Config) -> Vec<Box<dyn Provider>> {
    let mut providers: Vec<Box<dyn Provider>> = vec![
        Box::new(dns::DnsProvider::new(context.resolver.clone())),
        Box::new(whois::WhoisProvider::new(context.client.clone())),
        Box::new(scraper::ScraperProvider::new(context.resolver.clone())),
    ];
    if let Some(key) = &config.virustotal_api_key {
        providers.push(Box::new(virustotal::VirusTotalProvider::new(context.client.clone(), key.clone())));
    }
    if let Some(key) = &config.shodan_api_key {
        providers.push(Box::new(shodan::ShodanProvider::new(context.client.clone(), key.clone())));
    }
    if let Some(key) = &config.hibp_api_key {
        providers.push(Box::new(hibp::HibpProvider::new(context.client.clone(), key.clone())));
    }
    if let Some(key) = &config.phishtank_api_key {
        providers.push(Box::new(phishtank::PhishTankProvider::new(context.client.clone(), key.clone())));
    }
    providers.push(Box::new(geoip::GeoIpProvider::new(context.client.clone())));
    providers
}

pub fn public_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            let octets = ip.octets();
            !ip.is_private()
                && !ip.is_loopback()
                && !ip.is_link_local()
                && !ip.is_unspecified()
                && !ip.is_multicast()
                && !ip.is_broadcast()
                && !(octets[0] == 100 && (64..=127).contains(&octets[1]))
                && octets[0] != 0
        }
        IpAddr::V6(ip) => {
            !ip.is_loopback()
                && !ip.is_unspecified()
                && !ip.is_multicast()
                && !ip.is_unique_local()
                && !ip.is_unicast_link_local()
                && ip.to_ipv4_mapped().map(public_ip).unwrap_or(true)
        }
    }
}

pub fn result(provider: &str, data: Value) -> ProviderResult {
    ProviderResult { provider: provider.to_owned(), data }
}

#[cfg(test)]
mod tests {
    use super::public_ip;
    use std::net::IpAddr;

    #[test]
    fn blocks_non_public_addresses_for_page_fetches() {
        for address in ["127.0.0.1", "10.1.2.3", "172.16.1.1", "192.168.1.1", "169.254.169.254", "::1", "fd00::1", "fe80::1"] {
            assert!(!public_ip(address.parse::<IpAddr>().expect("valid address")), "{address} should be blocked");
        }
        assert!(public_ip("8.8.8.8".parse().expect("valid address")));
    }
}
