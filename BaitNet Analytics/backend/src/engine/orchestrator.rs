use std::{sync::Arc, time::Duration};

use futures::{stream::FuturesUnordered, StreamExt};
use sqlx::PgPool;
use tokio::sync::Semaphore;
use uuid::Uuid;

use crate::{
    cache::Cache,
    config::Config,
    engine::{normalize::normalize, scoring},
    error::AppError,
    models::{indicator::{Indicator, IndicatorKind}, result::{RelatedIndicator, SearchResult}},
    providers::{create_providers, Provider, ProviderContext, ProviderError},
};

pub async fn search(
    pool: &PgPool,
    cache: &Cache,
    config: &Config,
    input: &str,
) -> Result<SearchResult, AppError> {
    let indicator = normalize(input).map_err(AppError::BadRequest)?;
    let cache_key = format!("search:{}:{}", indicator.kind, indicator.value);
    if let Some(cached) = cache.get::<SearchResult>(&cache_key).await? {
        return Ok(cached);
    }

    let context = ProviderContext::new().map_err(|error| {
        tracing::error!(%error, "provider initialization failed");
        AppError::Internal
    })?;
    let providers = create_providers(&context, config);
    let mut results = run_providers(&providers, &indicator, Some(cache)).await;
    persist(pool, &indicator, &results).await?;
    let id = persist_extracted_relations(pool, &indicator, &results).await?;
    redact_scraper_personal_data(&mut results);
    let report_count: i64 = sqlx::query_scalar("SELECT count(*) FROM user_reports WHERE indicator_id = $1")
        .bind(id).fetch_one(pool).await?;
    let linked_bad_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM relations r JOIN indicators i ON i.id = r.to_indicator WHERE r.from_indicator = $1 AND i.risk_score >= 70",
    ).bind(id).fetch_one(pool).await?;
    let score = scoring::calculate(&results, report_count, linked_bad_count);
    sqlx::query("UPDATE indicators SET risk_score = $2, verdict = $3, last_seen = now() WHERE id = $1")
        .bind(id).bind(i16::from(score.score)).bind(format!("{:?}", score.verdict)).execute(pool).await?;
    let related = related_indicators(pool, id).await?;
    let result = SearchResult {
        indicator: Indicator { id: Some(id), ..indicator },
        score: score.score,
        verdict: score.verdict,
        reasons: score.reasons,
        providers: std::mem::take(&mut results),
        related,
    };
    cache.set(&cache_key, &result).await?;
    Ok(result)
}

async fn run_providers(
    providers: &[Box<dyn Provider>],
    indicator: &Indicator,
    cache: Option<&Cache>,
) -> Vec<crate::models::result::ProviderResult> {
    let semaphore = Arc::new(Semaphore::new(5));
    let mut pending = FuturesUnordered::new();
    for provider in providers.iter().filter(|provider| provider.supports(&indicator.kind)) {
        let provider = provider.as_ref();
        let indicator = indicator.clone();
        let semaphore = semaphore.clone();
        let cache = cache.cloned();
        pending.push(async move {
            let permit = semaphore.acquire_owned().await;
            if let Err(error) = permit {
                tracing::error!(%error, provider = provider.name(), "provider semaphore closed");
                return None;
            }
            for attempt in 0..=2 {
                if let Some(cache) = &cache {
                    let (limit, window) = provider_budget(provider.name());
                    match cache.allow_requests(&format!("rate:provider:{}", provider.name()), limit, window).await {
                        Ok(true) => {}
                        Ok(false) => {
                            tracing::warn!(provider = provider.name(), "provider rate limit reached; skipping lookup");
                            return None;
                        }
                        Err(error) => {
                            tracing::error!(provider = provider.name(), %error, "provider rate limiter failed");
                            return None;
                        }
                    }
                }
                match provider.lookup(&indicator).await {
                    Ok(result) => return Some(result),
                    Err(error) if attempt < 2 && retryable(&error) => {
                        tracing::warn!(provider = provider.name(), attempt, %error, "retrying provider request");
                        tokio::time::sleep(Duration::from_millis(200 * (1 << attempt))).await;
                    }
                    Err(error) => {
                        tracing::warn!(provider = provider.name(), %error, "provider lookup failed");
                        return None;
                    }
                }
            }
            None
        });
    }
    let mut results = Vec::new();
    while let Some(result) = pending.next().await {
        if let Some(result) = result {
            results.push(result);
        }
    }
    results
}

fn provider_budget(provider: &str) -> (u64, u64) {
    match provider {
        "virustotal" => (4, 60),
        "shodan" => (5, 60),
        "hibp" | "phishtank" => (10, 60),
        "page_scraper" => (20, 60),
        _ => (60, 60),
    }
}

fn retryable(error: &ProviderError) -> bool {
    match error {
        ProviderError::Request(error) => error.is_timeout() || error.is_connect() || error.status().is_some_and(|status| status.is_server_error() || status.as_u16() == 429),
        ProviderError::Response(_) | ProviderError::Lookup(_) => false,
    }
}

async fn persist(pool: &PgPool, indicator: &Indicator, results: &[crate::models::result::ProviderResult]) -> Result<(), AppError> {
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO indicators (kind, value) VALUES ($1, $2) ON CONFLICT (kind, value) DO UPDATE SET last_seen = now() RETURNING id",
    ).bind(indicator.kind.to_string()).bind(&indicator.value).fetch_one(pool).await?;
    for result in results {
        sqlx::query("INSERT INTO lookups (indicator_id, provider, raw_json) VALUES ($1, $2, $3)")
            .bind(id).bind(&result.provider).bind(scrubbed_provider_data(result)).execute(pool).await?;
    }
    Ok(())
}

fn scrubbed_provider_data(result: &crate::models::result::ProviderResult) -> serde_json::Value {
    let mut data = result.data.clone();
    if result.provider == "page_scraper" {
        if let Some(object) = data.as_object_mut() {
            object.remove("emails");
            object.remove("phones");
        }
    }
    data
}

fn redact_scraper_personal_data(results: &mut [crate::models::result::ProviderResult]) {
    for result in results.iter_mut().filter(|result| result.provider == "page_scraper") {
        if let Some(object) = result.data.as_object_mut() {
            for field in ["emails", "phones"] {
                let count = object.get(field).and_then(serde_json::Value::as_array).map_or(0, Vec::len);
                object.remove(field);
                object.insert(format!("{field}_found"), serde_json::json!(count));
            }
        }
    }
}

async fn persist_extracted_relations(
    pool: &PgPool,
    indicator: &Indicator,
    results: &[crate::models::result::ProviderResult],
) -> Result<Uuid, AppError> {
    let from_id: Uuid = sqlx::query_scalar("SELECT id FROM indicators WHERE kind = $1 AND value = $2")
        .bind(indicator.kind.to_string()).bind(&indicator.value).fetch_one(pool).await?;
    let Some(scraped) = results.iter().find(|result| result.provider == "page_scraper") else { return Ok(from_id) };
    let fields = [
        ("linked_domains", "linked_domain", IndicatorKind::Domain),
        ("emails", "published_email", IndicatorKind::Email),
        ("phones", "published_phone", IndicatorKind::Phone),
        ("bitcoin_wallets", "published_wallet", IndicatorKind::Bitcoin),
        ("ethereum_wallets", "published_wallet", IndicatorKind::Ethereum),
    ];
    for (field, relation_type, kind) in fields {
        let Some(values) = scraped.data.get(field).and_then(serde_json::Value::as_array) else { continue };
        for value in values.iter().filter_map(serde_json::Value::as_str).take(20) {
            if let Ok(target) = normalize(value) {
                if target.kind != kind { continue; }
                let to_id: Uuid = sqlx::query_scalar(
                    "INSERT INTO indicators (kind, value) VALUES ($1, $2) ON CONFLICT (kind, value) DO UPDATE SET last_seen = now() RETURNING id",
                ).bind(target.kind.to_string()).bind(&target.value).fetch_one(pool).await?;
                if to_id != from_id {
                    sqlx::query("INSERT INTO relations (from_indicator, to_indicator, relation_type) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING")
                        .bind(from_id).bind(to_id).bind(relation_type).execute(pool).await?;
                }
            }
        }
    }
    Ok(from_id)
}

async fn related_indicators(pool: &PgPool, id: Uuid) -> Result<Vec<RelatedIndicator>, AppError> {
    let rows = sqlx::query_as::<_, (Uuid, String, String, String)>(
        "SELECT i.id, i.kind, i.value, r.relation_type FROM relations r JOIN indicators i ON i.id = r.to_indicator WHERE r.from_indicator = $1 ORDER BY i.risk_score DESC, i.last_seen DESC LIMIT 50",
    ).bind(id).fetch_all(pool).await?;
    Ok(rows.into_iter().map(|(id, kind, value, relation_type)| RelatedIndicator {
        id,
        kind,
        value: if kind == "email" { mask_email(&value) } else if kind == "phone" { mask_phone(&value) } else { value },
        relation_type,
    }).collect())
}

fn mask_email(value: &str) -> String {
    let Some((local, domain)) = value.split_once('@') else { return "[redacted]".to_owned() };
    format!("{}***@{domain}", local.chars().take(2).collect::<String>())
}

fn mask_phone(value: &str) -> String {
    let digits: String = value.chars().filter(char::is_ascii_digit).collect();
    format!("***{}", digits.chars().rev().take(4).collect::<String>().chars().rev().collect::<String>())
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use serde_json::json;

    use super::*;
    use crate::models::result::ProviderResult;
    use crate::providers::Provider;

    struct MockProvider {
        provider_name: &'static str,
        should_fail: bool,
    }

    #[async_trait]
    impl Provider for MockProvider {
        fn name(&self) -> &'static str { self.provider_name }
        fn supports(&self, _: &IndicatorKind) -> bool { true }
        async fn lookup(&self, _: &Indicator) -> Result<ProviderResult, ProviderError> {
            if self.should_fail {
                return Err(ProviderError::Lookup("mock failure".to_owned()));
            }
            Ok(ProviderResult { provider: self.name().to_owned(), data: json!({"ok": true}) })
        }
    }

    #[tokio::test]
    async fn provider_failure_does_not_discard_successful_results() {
        let providers: Vec<Box<dyn Provider>> = vec![
            Box::new(MockProvider { provider_name: "healthy", should_fail: false }),
            Box::new(MockProvider { provider_name: "broken", should_fail: true }),
        ];
        let indicator = Indicator::new(IndicatorKind::Domain, "example.com".to_owned());
        let results = run_providers(&providers, &indicator, None).await;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].provider, "healthy");
    }
}
