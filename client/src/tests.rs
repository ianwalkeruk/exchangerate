#[cfg(test)]
use crate::{AuthMethod, CacheBackend, CacheConfig, ExchangeRateClient, InMemoryCache};
use std::env;
use std::sync::Arc;
use std::time::Duration;

#[test]
fn test_url_construction_in_url_auth() {
    let client = ExchangeRateClient {
        api_key: "test_key".to_string(),
        base_url: "https://v6.exchangerate-api.com/v6".to_string(),
        auth_method: AuthMethod::InUrl,
        http_client: reqwest::Client::new(),
        cache: None,
        cache_config: CacheConfig::default(),
    };

    let url = client.build_url("latest", &["USD"]);
    assert_eq!(
        url,
        "https://v6.exchangerate-api.com/v6/test_key/latest/USD"
    );
}

#[test]
fn test_url_construction_bearer_token() {
    let client = ExchangeRateClient {
        api_key: "test_key".to_string(),
        base_url: "https://v6.exchangerate-api.com/v6".to_string(),
        auth_method: AuthMethod::BearerToken,
        http_client: reqwest::Client::new(),
        cache: None,
        cache_config: CacheConfig::default(),
    };

    let url = client.build_url("latest", &["USD"]);
    assert_eq!(url, "https://v6.exchangerate-api.com/v6/latest/USD");
}

#[test]
fn test_builder_pattern() {
    let client = ExchangeRateClient::builder()
        .api_key("test_key")
        .auth_method(AuthMethod::BearerToken)
        .timeout(Duration::from_secs(60))
        .build()
        .unwrap();

    assert_eq!(client.api_key, "test_key");
    assert!(matches!(client.auth_method, AuthMethod::BearerToken));
}

#[tokio::test]
#[ignore] // Requires API key, so ignore by default
async fn test_get_latest_rates() {
    // This test requires an actual API key
    let api_key = env::var("EXCHANGE_RATE_API_KEY").expect("EXCHANGE_RATE_API_KEY not set");

    let client = ExchangeRateClient::builder()
        .api_key(api_key)
        .build()
        .unwrap();

    let rates = client.get_latest_rates("USD").await.unwrap();

    // Verify we got some rates back
    assert!(!rates.conversion_rates.is_empty());

    // Check for some common currencies
    assert!(rates.get_rate("EUR").is_some());
    assert!(rates.get_rate("GBP").is_some());
    assert!(rates.get_rate("JPY").is_some());
}

#[tokio::test]
#[ignore] // Requires API key, so ignore by default
async fn test_convert_currency() {
    // This test requires an actual API key
    let api_key = env::var("EXCHANGE_RATE_API_KEY").expect("EXCHANGE_RATE_API_KEY not set");

    let client = ExchangeRateClient::builder()
        .api_key(api_key)
        .build()
        .unwrap();

    // Convert 100 USD to EUR
    let amount_in_eur = client.convert(100.0, "USD", "EUR").await.unwrap();

    // Just verify we got a positive number back
    assert!(amount_in_eur > 0.0);

    println!("100 USD = {amount_in_eur} EUR");
}

#[tokio::test]
#[ignore] // Requires API key, so ignore by default
async fn test_bearer_token_auth() {
    // This test requires an actual API key
    let api_key = env::var("EXCHANGE_RATE_API_KEY").expect("EXCHANGE_RATE_API_KEY not set");

    let client = ExchangeRateClient::builder()
        .api_key(api_key)
        .auth_method(AuthMethod::BearerToken) // Explicitly set bearer token auth
        .build()
        .unwrap();

    // Test that we can still get rates with bearer token auth
    let rates = client.get_latest_rates("USD").await.unwrap();

    // Verify we got some rates back
    assert!(!rates.conversion_rates.is_empty());
}

#[tokio::test]
async fn test_caching() {
    // Create a client with caching enabled
    let cache = Arc::new(InMemoryCache::new());

    let client = ExchangeRateClient::builder()
        .api_key("test_key")
        .with_cache(cache.clone())
        .build()
        .unwrap();

    // Create a mock response
    use crate::models::ExchangeRateResponse;
    use chrono::Utc;
    use std::collections::HashMap;

    let mut rates = HashMap::new();
    rates.insert("EUR".to_string(), 0.85);
    rates.insert("GBP".to_string(), 0.75);

    let response = ExchangeRateResponse {
        result: "success".to_string(),
        documentation: "https://www.exchangerate-api.com/docs".to_string(),
        terms_of_use: "https://www.exchangerate-api.com/terms".to_string(),
        time_last_update_unix: 1620000000,
        time_last_update_utc: "Mon, 03 May 2021 00:00:00 +0000".to_string(),
        time_next_update_unix: 1620086400,
        time_next_update_utc: "Tue, 04 May 2021 00:00:00 +0000".to_string(),
        base_code: "USD".to_string(),
        conversion_rates: rates,
    };

    // Store the response in the cache
    let cached_response = crate::cache::CachedResponse {
        response: response.clone(),
        cached_at: Utc::now(),
        expires_at: Utc::now() + chrono::Duration::hours(1),
    };

    // Create the same cache key that the client will use
    use crate::cache::create_cache_key;
    let cache_key = create_cache_key("latest", &["USD"]);

    // Store in cache
    cache
        .set_exchange_rate(&cache_key, cached_response)
        .await
        .unwrap();

    // Now try to get the response from the client
    // This should use the cached response without making an API call
    let result = client.get_latest_rates("USD").await;

    // Print the error if there is one
    if let Err(ref e) = result {
        println!("Error: {:?}", e);
    }

    // Since we're not making a real API call, this would fail if it tried to
    // but should succeed because it's using the cache
    assert!(result.is_ok());

    let retrieved = result.unwrap();
    assert_eq!(retrieved.base_code, "USD");
    assert_eq!(retrieved.get_rate("EUR").unwrap(), 0.85);
    assert_eq!(retrieved.get_rate("GBP").unwrap(), 0.75);
}
