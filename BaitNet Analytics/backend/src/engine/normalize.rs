use std::{net::IpAddr, str::FromStr};

use phonenumber::{parse, Mode};
use regex::Regex;
use url::Url;

use crate::models::indicator::{Indicator, IndicatorKind};

pub fn normalize(input: &str) -> Result<Indicator, String> {
    let input = input.trim();
    if input.is_empty() || input.len() > 2048 || input.chars().any(char::is_control) {
        return Err("Indicator must be non-empty, at most 2048 characters, and contain no control characters".to_owned());
    }

    if let Ok(address) = IpAddr::from_str(input) {
        let (kind, value) = match address {
            IpAddr::V4(ip) => (IndicatorKind::Ipv4, ip.to_string()),
            IpAddr::V6(ip) => (IndicatorKind::Ipv6, ip.to_string()),
        };
        return Ok(Indicator::new(kind, value));
    }

    if input.contains("://") {
        let mut url = Url::parse(input).map_err(|_| "URL is invalid".to_owned())?;
        if !matches!(url.scheme(), "http" | "https")
            || url.host_str().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
        {
            return Err("Only public HTTP and HTTPS URLs without embedded credentials are supported".to_owned());
        }
        url.set_fragment(None);
        if let Some(url::Host::Domain(host)) = url.host() {
            let ascii_host = url::Host::parse(host).map_err(|_| "URL host is invalid".to_owned())?.to_string();
            url.set_host(Some(&ascii_host)).map_err(|_| "URL host is invalid".to_owned())?;
        }
        return Ok(Indicator::new(IndicatorKind::Url, url.to_string()));
    }

    if input.contains('@') {
        let (local, domain) = input.split_once('@').ok_or_else(|| "Email address is invalid".to_owned())?;
        let allowed_local = |character: char| {
            character.is_ascii_alphanumeric() || "!#$%&'*+/=?^_`{|}~.-".contains(character)
        };
        let ascii_domain = url::Host::parse(domain.trim_end_matches('.'))
            .map_err(|_| "Email address is invalid".to_owned())?
            .to_string()
            .to_ascii_lowercase();
        if local.is_empty()
            || local.len() > 64
            || !local.chars().all(allowed_local)
            || local.starts_with('.')
            || local.ends_with('.')
            || local.contains("..")
            || ascii_domain.len() > 253
            || !ascii_domain.contains('.')
            || ascii_domain.starts_with('.')
        {
            return Err("Email address is invalid".to_owned());
        }
        return Ok(Indicator::new(IndicatorKind::Email, format!("{}@{}", local, ascii_domain)));
    }

    if let Some(normalized) = normalize_phone(input) {
        return Ok(Indicator::new(IndicatorKind::Phone, normalized));
    }

    let lower = input.to_ascii_lowercase();
    let btc = Regex::new(r"^(?:[13][a-km-zA-HJ-NP-Z1-9]{25,34}|bc1[ac-hj-np-z02-9]{11,71})$")
        .map_err(|error| error.to_string())?;
    if btc.is_match(&lower) {
        return Ok(Indicator::new(IndicatorKind::Bitcoin, lower));
    }
    let eth = Regex::new(r"^0x[a-fA-F0-9]{40}$").map_err(|error| error.to_string())?;
    if eth.is_match(input) {
        return Ok(Indicator::new(IndicatorKind::Ethereum, lower));
    }

    let domain = input.trim_end_matches('.').to_ascii_lowercase();
    let ascii = url::Host::parse(&domain)
        .map_err(|_| "Indicator must be an IP, domain, URL, email, E.164 phone, or supported crypto wallet".to_owned())?
        .to_string()
        .to_ascii_lowercase();
    if ascii.len() > 253
        || !ascii.contains('.')
        || ascii.split('.').any(|label| label.is_empty() || label.len() > 63 || label.starts_with('-') || label.ends_with('-'))
        || ascii.chars().any(|character| !character.is_ascii_alphanumeric() && character != '.' && character != '-')
    {
        return Err("Domain name is invalid".to_owned());
    }
    Ok(Indicator::new(IndicatorKind::Domain, ascii))
}

fn normalize_phone(input: &str) -> Option<String> {
    if !input.starts_with('+') {
        return None;
    }
    let parsed = parse(None, input).ok()?;
    if !parsed.is_valid() {
        return None;
    }
    Some(parsed.format().mode(Mode::E164).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_and_normalizes_ip_and_domain() {
        let ip = normalize("8.8.8.8").expect("valid IP");
        assert_eq!(ip.kind, IndicatorKind::Ipv4);
        assert_eq!(ip.value, "8.8.8.8");

        let domain = normalize("ExAmPle.COM.").expect("valid domain");
        assert_eq!(domain.kind, IndicatorKind::Domain);
        assert_eq!(domain.value, "example.com");
    }

    #[test]
    fn normalizes_urls_and_rejects_credentials() {
        let result = normalize("HTTPS://Example.com/path#fragment").expect("valid URL");
        assert_eq!(result.kind, IndicatorKind::Url);
        assert_eq!(result.value, "https://example.com/path");
        assert!(normalize("https://user:pass@example.com").is_err());
    }

    #[test]
    fn rejects_unsupported_and_invalid_inputs() {
        assert!(normalize("").is_err());
        assert!(normalize("localhost").is_err());
        assert!(normalize("not an indicator").is_err());
    }

    #[test]
    fn recognizes_wallets_and_email() {
        assert_eq!(normalize("0x52908400098527886E0F7030069857D2E4169EE7").expect("wallet").kind, IndicatorKind::Ethereum);
        assert_eq!(normalize("Analyst@Example.COM").expect("email").value, "Analyst@example.com");
        assert_eq!(normalize("+14155552671").expect("E.164 phone").value, "+14155552671");
        assert!(normalize("4155552671").is_err());
        assert!(normalize("bad address@example.com").is_err());
    }
}
