use client::{AuthMethod, ExchangeRateClient};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get API key from environment variable
    let api_key = env::var("EXCHANGE_RATE_API_KEY")
        .expect("Please set the EXCHANGE_RATE_API_KEY environment variable");
    
    println!("Creating Exchange Rate API client with InUrl authentication...");
    
    // Create client with InUrl authentication (less secure but simpler)
    let client = ExchangeRateClient::builder()
        .api_key(api_key)
        .auth_method(AuthMethod::InUrl) // Explicitly set InUrl auth
        .build()?;
    
    println!("Fetching latest exchange rates with EUR as base currency...");
    
    // Get latest rates with EUR as base currency
    let rates = client.get_latest_rates("EUR").await?;
    
    println!("Base currency: {}", rates.base_code);
    println!("Last updated: {}", rates.time_last_update_utc);
    
    // Print some common currency rates
    println!("\nExchange rates from EUR:");
    println!("  USD: {:.4}", rates.get_rate("USD").unwrap_or(0.0));
    println!("  GBP: {:.4}", rates.get_rate("GBP").unwrap_or(0.0));
    println!("  JPY: {:.4}", rates.get_rate("JPY").unwrap_or(0.0));
    println!("  CHF: {:.4}", rates.get_rate("CHF").unwrap_or(0.0));
    println!("  CNY: {:.4}", rates.get_rate("CNY").unwrap_or(0.0));
    
    // Convert 100 EUR to USD
    let amount = 100.0;
    let from_currency = "EUR";
    let to_currency = "USD";
    
    println!("\nConverting {:.2} {} to {}...", amount, from_currency, to_currency);
    let converted = client.convert(amount, from_currency, to_currency).await?;
    println!("{:.2} {} = {:.2} {}", amount, from_currency, converted, to_currency);
    
    Ok(())
}