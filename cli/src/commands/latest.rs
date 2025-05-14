use anyhow::Result;
use client::ExchangeRateClient;

use crate::formatters;

pub async fn execute(
    client: &ExchangeRateClient,
    base_currency: &str,
    format: Option<&str>,
) -> Result<()> {
    // Get latest rates
    let rates = client.get_latest_rates(base_currency).await?;

    // Format and print the result
    let output = formatters::format_latest_rates(&rates, format)?;
    println!("{}", output);

    Ok(())
}
