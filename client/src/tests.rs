#[cfg(test)]
use crate::{AuthMethod, ExchangeRateClient};
use std::env;
use std::time::Duration;

#[test]
fn test_url_construction_in_url_auth() {
    let client = ExchangeRateClient {
        api_key: "test_key".to_string(),
        base_url: "https://v6.exchangerate-api.com/v6".to_string(),
        auth_method: AuthMethod::InUrl,
        http_client: reqwest::Client::new(),
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
