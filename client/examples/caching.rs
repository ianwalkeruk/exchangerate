use client::{ExchangeRateClient, InMemoryCache};
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get API key from environment variable
    let api_key = std::env::var("EXCHANGE_RATE_API_KEY")
        .expect("EXCHANGE_RATE_API_KEY environment variable must be set");

    // Create a client with in-memory caching enabled
    let client = ExchangeRateClient::builder()
        .api_key(api_key)
        .with_cache(Arc::new(InMemoryCache::new()))
        .build()?;

    println!("First request (will fetch from API):");
    let start = std::time::Instant::now();
    let rates = client.get_latest_rates("USD").await?;
    println!("Request took: {:?}", start.elapsed());
    println!("USD to EUR rate: {}", rates.get_rate("EUR").unwrap_or(0.0));

    // Wait a moment to demonstrate the difference
    std::thread::sleep(Duration::from_millis(500));

    println!("\nSecond request (should be cached):");
    let start = std::time::Instant::now();
    let rates = client.get_latest_rates("USD").await?;
    println!("Request took: {:?}", start.elapsed());
    println!("USD to EUR rate: {}", rates.get_rate("EUR").unwrap_or(0.0));

    // Try a different base currency (should not be cached)
    println!("\nRequest with different base currency (should fetch from API):");
    let start = std::time::Instant::now();
    let rates = client.get_latest_rates("EUR").await?;
    println!("Request took: {:?}", start.elapsed());
    println!("EUR to USD rate: {}", rates.get_rate("USD").unwrap_or(0.0));

    // Try pair conversion (should use a different cache key)
    println!("\nPair conversion request:");
    let start = std::time::Instant::now();
    let rate = client.get_pair_conversion("USD", "EUR").await?;
    println!("Request took: {:?}", start.elapsed());
    println!("USD to EUR direct rate: {}", rate);

    // Second pair conversion (should be cached)
    println!("\nSecond pair conversion request (should be cached):");
    let start = std::time::Instant::now();
    let rate = client.get_pair_conversion("USD", "EUR").await?;
    println!("Request took: {:?}", start.elapsed());
    println!("USD to EUR direct rate: {}", rate);

    Ok(())
}
