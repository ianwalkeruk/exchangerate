use client::ExchangeRateClient;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get API key from environment variable
    let api_key = env::var("EXCHANGE_RATE_API_KEY")
        .expect("Please set the EXCHANGE_RATE_API_KEY environment variable");
    
    println!("Creating Exchange Rate API client...");
    
    // Create client with bearer token authentication (default)
    let client = ExchangeRateClient::builder()
        .api_key(api_key)
        .build()?;
    
    println!("Fetching latest exchange rates with USD as base currency...");
    
    // Get latest rates with USD as base currency
    let rates = client.get_latest_rates("USD").await?;
    
    println!("Base currency: {}", rates.base_code);
    println!("Last updated: {}", rates.time_last_update_utc);
    
    // Print some common currency rates
    println!("\nExchange rates from USD:");
    println!("  EUR: {:.4}", rates.get_rate("EUR").unwrap_or(0.0));
    println!("  GBP: {:.4}", rates.get_rate("GBP").unwrap_or(0.0));
    println!("  JPY: {:.4}", rates.get_rate("JPY").unwrap_or(0.0));
    println!("  CAD: {:.4}", rates.get_rate("CAD").unwrap_or(0.0));
    println!("  AUD: {:.4}", rates.get_rate("AUD").unwrap_or(0.0));
    
    // Convert 100 USD to EUR
    let amount = 100.0;
    let from_currency = "USD";
    let to_currency = "EUR";
    
    println!("\nConverting {amount:.2} {from_currency} to {to_currency}...");
    let converted = client.convert(amount, from_currency, to_currency).await?;
    println!("{amount:.2} {from_currency} = {converted:.2} {to_currency}");
    
    // Get direct pair conversion rate
    println!("\nGetting direct pair conversion rate from GBP to JPY...");
    let rate = client.get_pair_conversion("GBP", "JPY").await?;
    println!("1 GBP = {rate:.4} JPY");
    
    // Get supported currencies
    println!("\nFetching supported currencies...");
    let currencies = client.get_supported_codes().await?;
    println!("Supported currencies (first 5):");
    for (i, (code, name)) in currencies.iter().take(5).enumerate() {
        println!("  {}. {} - {}", i + 1, code, name);
    }
    
    println!("\nTotal supported currencies: {}", currencies.len());
    
    Ok(())
}