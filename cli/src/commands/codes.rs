use client::ExchangeRateClient;
use colored::Colorize;

use crate::error::CliError;
use crate::formatters;

/// Execute the codes command
///
/// # Arguments
///
/// * `client` - The Exchange Rate API client
/// * `format` - The output format (text, json, csv)
/// * `verbose` - Whether to enable verbose output
///
/// # Returns
///
/// * `Result<(), CliError>` - Ok if successful, Err otherwise
pub async fn execute(
    client: &ExchangeRateClient,
    format: Option<&str>,
    verbose: bool,
) -> Result<(), CliError> {
    // Get supported currency codes
    if verbose {
        println!(
            "{} Fetching supported currency codes",
            "Info:".bold().blue()
        );
    }

    let codes = client.get_supported_codes().await?;

    if verbose {
        println!(
            "{} Retrieved {} supported currency codes",
            "Info:".bold().blue(),
            codes.supported_codes.len()
        );
    }

    // Format and print the result
    if verbose {
        println!("{} Formatting output", "Info:".bold().blue());
    }

    let output = formatters::format_currency_codes(&codes, format)?;
    println!("{}", output);

    Ok(())
}
