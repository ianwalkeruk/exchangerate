use client::ExchangeRateClient;

use crate::error::CliError;
use crate::formatters;
use crate::utils;

pub async fn execute(
    client: &ExchangeRateClient,
    amount: f64,
    from_currency: &str,
    to_currency: &str,
    format: Option<&str>,
) -> Result<(), CliError> {
    // Validate currency codes
    utils::validate_currency_code(from_currency)?;
    utils::validate_currency_code(to_currency)?;
    // Get the conversion rate
    let rates = client.get_latest_rates(from_currency).await?;
    let rate = rates.get_rate(to_currency).unwrap_or(0.0);

    // Calculate the converted amount
    let converted_amount = client.convert(amount, from_currency, to_currency).await?;

    // Format and print the result
    let output = formatters::format_conversion(
        amount,
        from_currency,
        to_currency,
        converted_amount,
        rate,
        format,
    )?;
    println!("{}", output);

    Ok(())
}
