use client::ExchangeRateClient;

use crate::error::CliError;
use crate::formatters;
use crate::utils;

pub async fn execute(
    client: &ExchangeRateClient,
    base_currency: &str,
    format: Option<&str>,
) -> Result<(), CliError> {
    // Validate currency code
    utils::validate_currency_code(base_currency)?;

    // Get latest rates
    let rates = client.get_latest_rates(base_currency).await?;

    // Format and print the result
    let output = formatters::format_latest_rates(&rates, format)?;
    println!("{}", output);

    Ok(())
}
