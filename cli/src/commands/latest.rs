use client::ExchangeRateClient;
use colored::Colorize;

use crate::error::CliError;
use crate::formatters;
use crate::utils;

/// Execute the latest command
///
/// # Arguments
///
/// * `client` - The Exchange Rate API client
/// * `base_currency` - The base currency code
/// * `format` - The output format (text, json, csv)
/// * `verbose` - Whether to enable verbose output
///
/// # Returns
///
/// * `Result<(), CliError>` - Ok if successful, Err otherwise
pub async fn execute(
    client: &ExchangeRateClient,
    base_currency: &str,
    format: Option<&str>,
    verbose: bool,
) -> Result<(), CliError> {
    // Validate currency code
    utils::validate_currency_code(base_currency)?;

    if verbose {
        println!(
            "{} Validating currency code: {}",
            "Info:".bold().blue(),
            base_currency
        );
    }

    // Get latest rates
    if verbose {
        println!(
            "{} Fetching latest rates for {}",
            "Info:".bold().blue(),
            base_currency
        );
    }

    let rates = client.get_latest_rates(base_currency).await?;

    if verbose {
        println!(
            "{} Retrieved {} currency rates, last updated: {}",
            "Info:".bold().blue(),
            rates.conversion_rates.len(),
            rates.time_last_update_utc
        );
    }

    // Format and print the result
    if verbose {
        println!("{} Formatting output", "Info:".bold().blue());
    }

    let output = formatters::format_latest_rates(&rates, format)?;
    println!("{}", output);

    Ok(())
}
