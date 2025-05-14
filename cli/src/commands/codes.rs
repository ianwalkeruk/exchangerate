use client::ExchangeRateClient;

use crate::error::CliError;
use crate::formatters;

pub async fn execute(client: &ExchangeRateClient, format: Option<&str>) -> Result<(), CliError> {
    // Get supported currency codes
    let codes = client.get_supported_codes().await?;

    // Format and print the result
    let output = formatters::format_currency_codes(&codes, format)?;
    println!("{}", output);

    Ok(())
}
