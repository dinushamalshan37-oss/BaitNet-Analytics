use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::models::result::{ProviderResult, ScoreReason, Verdict};

#[derive(Debug)]
pub struct Score {
    pub score: u8,
    pub verdict: Verdict,
    pub reasons: Vec<ScoreReason>,
}

pub fn calculate(providers: &[ProviderResult], report_count: i64, linked_bad_count: i64) -> Score {
    let mut reasons = Vec::new();
    let mut points = 0_i16;

    if let Some(vt) = providers.iter().find(|result| result.provider == "virustotal") {
        let malicious = vt.data.pointer("/data/attributes/last_analysis_stats/malicious")
            .and_then(Value::as_i64).unwrap_or(0);
        let suspicious = vt.data.pointer("/data/attributes/last_analysis_stats/suspicious")
            .and_then(Value::as_i64).unwrap_or(0);
        if malicious > 0 {
            let weight = if malicious >= 5 { 55 } else { 40 };
            points += weight;
            reasons.push(reason("virustotal_malicious", weight, format!("{malicious} VirusTotal engines reported malicious activity")));
        }
        if suspicious > 0 {
            points += 15;
            reasons.push(reason("virustotal_suspicious", 15, format!("{suspicious} VirusTotal engines reported suspicious activity")));
        }
    }

    if let Some(phishtank) = providers.iter().find(|result| result.provider == "phishtank") {
        let confirmed = phishtank.data.pointer("/results/in_database").and_then(Value::as_bool).unwrap_or(false)
            || phishtank.data.pointer("/results/valid").and_then(Value::as_bool).unwrap_or(false);
        if confirmed {
            points += 70;
            reasons.push(reason("phishtank_confirmed", 70, "PhishTank lists this URL as a confirmed phishing site"));
        }
    }

    if let Some(rdap) = providers.iter().find(|result| result.provider == "rdap") {
        if let Some(days) = registration_age_days(&rdap.data) {
            if days < 30 {
                points += 15;
                reasons.push(reason("recent_domain_registration", 15, format!("Domain registration is approximately {days} days old")));
            }
        }
    }

    if report_count >= 3 {
        let weight = ((report_count.min(10) - 2) * 5) as i16;
        points += weight;
        reasons.push(reason("multiple_user_reports", weight, format!("{report_count} user reports are associated with this indicator")));
    }

    if linked_bad_count > 0 {
        let weight = (linked_bad_count.min(3) * 15) as i16;
        points += weight;
        reasons.push(reason("linked_to_known_bad", weight, format!("Linked to {linked_bad_count} known high-risk indicators")));
    }

    if let Some(hibp) = providers.iter().find(|result| result.provider == "hibp") {
        let count = hibp.data.pointer("/breaches").and_then(Value::as_array).map_or(0, Vec::len);
        if count > 0 {
            reasons.push(reason("email_breach_exposure", 0, format!("Email appeared in {count} breach(es); this is exposure information, not a scam signal")));
        }
    }

    let score = points.clamp(0, 100) as u8;
    let has_signal = reasons.iter().any(|item| item.weight > 0);
    let verdict = if !has_signal {
        Verdict::Unknown
    } else if score >= 70 {
        Verdict::Malicious
    } else if score >= 35 {
        Verdict::Suspicious
    } else {
        Verdict::Safe
    };
    Score { score, verdict, reasons }
}

fn reason(signal: &str, weight: i16, detail: String) -> ScoreReason {
    ScoreReason { signal: signal.to_owned(), weight, detail }
}

fn registration_age_days(data: &Value) -> Option<i64> {
    let registration = data.get("events")?.as_array()?.iter()
        .find(|event| event.get("eventAction").and_then(Value::as_str).is_some_and(|action| action == "registration"))?
        .get("eventDate")?.as_str()?;
    let registered = DateTime::parse_from_rfc3339(registration).ok()?.with_timezone(&Utc);
    Some(Utc::now().signed_duration_since(registered).num_days().max(0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn scores_phishing_as_malicious_with_explanation() {
        let providers = vec![ProviderResult { provider: "phishtank".to_owned(), data: json!({"results":{"in_database":true}}) }];
        let score = calculate(&providers, 0, 0);
        assert_eq!(score.score, 70);
        assert!(matches!(score.verdict, Verdict::Malicious));
        assert_eq!(score.reasons[0].signal, "phishtank_confirmed");
    }

    #[test]
    fn breach_exposure_does_not_increase_risk() {
        let providers = vec![ProviderResult { provider: "hibp".to_owned(), data: json!({"breaches":[{"Name":"Example"}]}) }];
        let score = calculate(&providers, 0, 0);
        assert_eq!(score.score, 0);
        assert!(matches!(score.verdict, Verdict::Unknown));
        assert_eq!(score.reasons[0].weight, 0);
    }

    #[test]
    fn multiple_signals_are_capped_and_explained() {
        let providers = vec![
            ProviderResult { provider: "virustotal".to_owned(), data: json!({"data":{"attributes":{"last_analysis_stats":{"malicious":8}}}}) },
            ProviderResult { provider: "phishtank".to_owned(), data: json!({"results":{"valid":true}}) },
        ];
        let score = calculate(&providers, 9, 5);
        assert_eq!(score.score, 100);
        assert!(matches!(score.verdict, Verdict::Malicious));
    }
}
