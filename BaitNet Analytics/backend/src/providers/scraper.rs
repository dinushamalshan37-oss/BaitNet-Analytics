use std::{collections::HashSet, net::IpAddr, time::Duration};

use async_trait::async_trait;
use futures::StreamExt;
use hickory_resolver::TokioAsyncResolver;
use regex::Regex;
use reqwest::{redirect::Policy, Client};
use scraper::{Html, Selector};
use serde_json::{json, Value};
use url::Url;

use crate::models::indicator::{Indicator, IndicatorKind};

use super::{public_ip, result, Provider, ProviderError};

const MAX_BODY_BYTES: usize = 2 * 1024 * 1024;
const MAX_REDIRECTS: usize = 3;

pub struct ScraperProvider {
    resolver: TokioAsyncResolver,
}

impl ScraperProvider {
    pub fn new(resolver: TokioAsyncResolver) -> Self { Self { resolver } }

    async fn safe_fetch(&self, mut url: Url) -> Result<Vec<u8>, ProviderError> {
        for redirect_count in 0..=MAX_REDIRECTS {
            if !matches!(url.scheme(), "http" | "https") || !url.username().is_empty() || url.password().is_some() {
                return Err(ProviderError::Lookup("scraper refused unsupported URL".to_owned()));
            }
            let host = url.host_str().ok_or_else(|| ProviderError::Lookup("URL has no host".to_owned()))?.to_owned();
            let addresses: Vec<IpAddr> = if let Ok(ip) = host.parse::<IpAddr>() {
                vec![ip]
            } else {
                self.resolver.lookup_ip(&host).await
                    .map_err(|error| ProviderError::Lookup(format!("DNS lookup failed: {error}")))?
                    .iter().collect()
            };
            if addresses.is_empty() || addresses.iter().any(|address| !public_ip(*address)) {
                return Err(ProviderError::Lookup("scraper blocked a non-public DNS address".to_owned()));
            }

            let builder = Client::builder()
                .redirect(Policy::none())
                .timeout(Duration::from_secs(10))
                .user_agent("OSINT-Tracker/0.1 (+public defensive intelligence; no JavaScript)")
                .resolve_to_addrs(&host, &addresses.iter().map(|address| std::net::SocketAddr::new(*address, url.port_or_known_default().unwrap_or(443))).collect::<Vec<_>>());
            let client = builder.build()?;
            let response = client.get(url.clone()).send().await?;

            if response.status().is_redirection() {
                if redirect_count == MAX_REDIRECTS {
                    return Err(ProviderError::Lookup("scraper exceeded redirect limit".to_owned()));
                }
                let location = response.headers().get(reqwest::header::LOCATION)
                    .and_then(|value| value.to_str().ok())
                    .ok_or_else(|| ProviderError::Lookup("redirect omitted a valid location".to_owned()))?;
                url = url.join(location).map_err(|error| ProviderError::Lookup(format!("invalid redirect URL: {error}")))?;
                continue;
            }
            if !response.status().is_success() {
                return Err(ProviderError::Lookup(format!("page returned HTTP {}", response.status())));
            }
            if response.headers().get(reqwest::header::CONTENT_LENGTH)
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.parse::<usize>().ok())
                .is_some_and(|length| length > MAX_BODY_BYTES)
            {
                return Err(ProviderError::Lookup("page exceeds the 2 MB body limit".to_owned()));
            }
            let mut stream = response.bytes_stream();
            let mut body = Vec::new();
            while let Some(chunk) = stream.next().await {
                let chunk = chunk?;
                if body.len() + chunk.len() > MAX_BODY_BYTES {
                    return Err(ProviderError::Lookup("page exceeds the 2 MB body limit".to_owned()));
                }
                body.extend_from_slice(&chunk);
            }
            return Ok(body);
        }
        Err(ProviderError::Lookup("scraper exceeded redirect limit".to_owned()))
    }

    fn extract(html: &str, origin: &Url) -> Value {
        let document = Html::parse_document(html);
        let anchor_selector = Selector::parse("a[href]").expect("static selector");
        let form_selector = Selector::parse("form").expect("static selector");
        let input_selector = Selector::parse("input").expect("static selector");
        let email_re = Regex::new(r"(?i)\b[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}\b").expect("static regex");
        let phone_re = Regex::new(r"(?x)(?<![A-Za-z0-9])\+?[0-9][0-9 .()\-]{7,}[0-9](?![A-Za-z0-9])").expect("static regex");
        let btc_re = Regex::new(r"(?i)\b(?:[13][a-km-z1-9]{25,34}|bc1[ac-hj-np-z02-9]{11,71})\b").expect("static regex");
        let eth_re = Regex::new(r"(?i)\b0x[a-f0-9]{40}\b").expect("static regex");

        let mut links = HashSet::new();
        let mut domains = HashSet::new();
        let mut social_links = HashSet::new();
        for element in document.select(&anchor_selector) {
            if let Some(href) = element.value().attr("href") {
                if let Ok(link) = origin.join(href) {
                    if matches!(link.scheme(), "http" | "https") {
                        if let Some(host) = link.host_str() {
                            if host != origin.host_str().unwrap_or_default() {
                                domains.insert(host.to_owned());
                                let host = host.to_ascii_lowercase();
                                if ["facebook.com", "instagram.com", "x.com", "twitter.com", "linkedin.com", "youtube.com", "tiktok.com", "t.me", "telegram.me"]
                                    .iter().any(|social| host == *social || host.ends_with(&format!(".{social}")))
                                {
                                    social_links.insert(link.to_string());
                                }
                            }
                        }
                        links.insert(link.to_string());
                    }
                }
            }
        }
        let forms: Vec<Value> = document.select(&form_selector).take(50).map(|form| {
            let action = form.value().attr("action").unwrap_or("");
            let mut indicators = Vec::new();
            let action_url = origin.join(action).map(|url| url.to_string()).unwrap_or_else(|_| action.to_owned());
            let action_lower = action_url.to_ascii_lowercase();
            if action_lower.contains("payment") || action_lower.contains("checkout") {
                indicators.push("payment");
            }
            let mut password_present = false;
            let mut payment_field_present = false;
            for input in form.select(&input_selector) {
                let kind = input.value().attr("type").unwrap_or("").to_ascii_lowercase();
                let name = input.value().attr("name").unwrap_or("").to_ascii_lowercase();
                let name = format!("{} {}", name, input.value().attr("id").unwrap_or("").to_ascii_lowercase());
                password_present |= kind == "password";
                payment_field_present |= ["card", "cvv", "cvc", "iban", "payment"].iter().any(|marker| name.contains(marker));
            }
            if password_present && !indicators.contains(&"payment") { indicators.push("login"); }
            if payment_field_present && !indicators.contains(&"payment") { indicators.push("payment"); }
            json!({
                "action": action_url,
                "method": form.value().attr("method").unwrap_or("get"),
                "indicators": indicators
            })
        }).collect();

        let text = document.root_element().text().collect::<Vec<_>>().join(" ");
        let searchable_text = format!("{html} {text}");
        let collect_matches = |regex: &Regex| -> Vec<String> {
            regex.find_iter(&searchable_text).take(20).map(|value| value.as_str().to_owned()).collect()
        };
        let emails: Vec<String> = collect_matches(&email_re);
        let phones: Vec<String> = collect_matches(&phone_re);
        let bitcoin_wallets: Vec<String> = collect_matches(&btc_re);
        let ethereum_wallets: Vec<String> = collect_matches(&eth_re);
        json!({
            "outbound_links": links.into_iter().take(100).collect::<Vec<_>>(),
            "social_links": social_links.into_iter().take(100).collect::<Vec<_>>(),
            "linked_domains": domains.into_iter().take(100).collect::<Vec<_>>(),
            "emails": emails,
            "phones": phones,
            "bitcoin_wallets": bitcoin_wallets,
            "ethereum_wallets": ethereum_wallets,
            "forms": forms
        })
    }
}

#[async_trait]
impl Provider for ScraperProvider {
    fn name(&self) -> &'static str { "page_scraper" }
    fn supports(&self, kind: &IndicatorKind) -> bool { matches!(kind, IndicatorKind::Url | IndicatorKind::Domain) }
    async fn lookup(&self, indicator: &Indicator) -> Result<super::super::models::result::ProviderResult, ProviderError> {
        let url = match indicator.kind {
            IndicatorKind::Url => Url::parse(&indicator.value).map_err(|error| ProviderError::Lookup(error.to_string()))?,
            IndicatorKind::Domain => Url::parse(&format!("https://{}", indicator.value)).map_err(|error| ProviderError::Lookup(error.to_string()))?,
            _ => return Err(ProviderError::Lookup("unsupported indicator".to_owned())),
        };
        let body = self.safe_fetch(url.clone()).await?;
        let html = String::from_utf8_lossy(&body);
        Ok(result(self.name(), Self::extract(&html, &url)))
    }
}
