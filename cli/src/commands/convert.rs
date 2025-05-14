use client::ExchangeRateClient;
use colored::Colorize;

use crate::error::CliError;
use crate::formatters;
use crate::utils;

/// Execute the convert command
///
/// # Arguments
///
/// * `client` - The Exchange Rate API client
/// * `amount` - The amount to convert
/// * `from_currency` - The source currency code
/// * `to_currency` - The target currency code
/// * `format` - The output format (text, json, csv)
/// * `verbose` - Whether to enable verbose output
///
/// # Returns
///
/// * `Result<(), CliError>` - Ok if successful, Err otherwise
pub async fn execute(
    client: &ExchangeRateClient,
    amount: f64,
    from_currency: &str,
    to_currency: &str,
    format: Option<&str>,
    verbose: bool,
) -> Result<(), CliError> {
    // Validate currency codes
    utils::validate_currency_code(from_currency)?;
    utils::validate_currency_code(to_currency)?;

    if verbose {
        println!("{} Validating currency codes", "Info:".bold().blue());
    }

    // Get the conversion rate
    if verbose {
        println!(
            "{} Fetching latest rates for {}",
            "Info:".bold().blue(),
            from_currency
        );
    }

    let rates = client.get_latest_rates(from_currency).await?;
    let rate = rates.get_rate(to_currency).unwrap_or(0.0);

    if verbose {
        println!(
            "{} Found rate: 1 {} = {:.4} {}",
            "Info:".bold().blue(),
            from_currency,
            rate,
            to_currency
        );
    }

    // Calculate the converted amount
    if verbose {
        println!("{} Converting amount", "Info:".bold().blue());
    }

    let converted_amount = client.convert(amount, from_currency, to_currency).await?;

    if verbose {
        println!(
            "{} Conversion result: {:.2} {} = {:.2} {}",
            "Info:".bold().blue(),
            amount,
            from_currency,
            converted_amount,
            to_currency
        );
    }

    // Format and print the result
    if verbose {
        println!("{} Formatting output", "Info:".bold().blue());
    }

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
