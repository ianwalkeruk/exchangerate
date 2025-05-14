use client::ExchangeRateClient;
use colored::Colorize;

use crate::error::CliError;
use crate::formatters;
use crate::utils;

/// Execute the pair command
///
/// # Arguments
///
/// * `client` - The Exchange Rate API client
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
    from_currency: &str,
    to_currency: &str,
    format: Option<&str>,
    verbose: bool,
) -> Result<(), CliError> {
    // Validate currency codes
    utils::validate_currency_code(from_currency)?;
    utils::validate_currency_code(to_currency)?;

    if verbose {
        println!(
            "{} Validating currency codes: {} and {}",
            "Info:".bold().blue(),
            from_currency,
            to_currency
        );
    }

    // Get the conversion rate
    if verbose {
        println!(
            "{} Fetching conversion rate from {} to {}",
            "Info:".bold().blue(),
            from_currency,
            to_currency
        );
    }

    let rate = client
        .get_pair_conversion(from_currency, to_currency)
        .await?;

    if verbose {
        println!(
            "{} Found rate: 1 {} = {:.4} {}",
            "Info:".bold().blue(),
            from_currency,
            rate,
            to_currency
        );
    }

    // Format and print the result
    if verbose {
        println!("{} Formatting output", "Info:".bold().blue());
    }

    let output = formatters::format_pair_rate(from_currency, to_currency, rate, format)?;
    println!("{}", output);

    Ok(())
}
