use anyhow::Result;
use client::ExchangeRateClient;

use crate::formatters;

pub async fn execute(
    client: &ExchangeRateClient,
    from_currency: &str,
    to_currency: &str,
    format: Option<&str>,
) -> Result<()> {
    // Get the conversion rate
    let rate = client
        .get_pair_conversion(from_currency, to_currency)
        .await?;

    // Format and print the result
    let output = formatters::format_pair_rate(from_currency, to_currency, rate, format)?;
    println!("{}", output);

    Ok(())
}
