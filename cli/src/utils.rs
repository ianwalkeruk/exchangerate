use crate::error::CliError;
use colored::Colorize;

/// Validates a currency code format (3 uppercase letters)
pub fn validate_currency_code(code: &str) -> Result<(), CliError> {
    if code.len() != 3 || !code.chars().all(|c| c.is_ascii_uppercase()) {
        return Err(CliError::InvalidCurrency(code.to_string()));
    }
    Ok(())
}

/// Prints a helpful message about environment variables
pub fn print_env_help() {
    println!(
        "\n{}\n{}\n",
        "Tip:".bold().green(),
        "You can set the EXCHANGE_RATE_API_KEY environment variable to avoid passing the API key each time."
    );
}

/// Formats a currency amount with symbol
pub fn format_currency_amount(amount: f64, currency: &str) -> String {
    let symbol = get_currency_symbol(currency);
    format!("{}{:.2}", symbol, amount)
}

/// Gets the currency symbol for a currency code
fn get_currency_symbol(code: &str) -> &str {
    match code {
        "USD" => "$",
        "EUR" => "€",
        "GBP" => "£",
        "JPY" => "¥",
        "CNY" => "¥",
        "AUD" => "A$",
        "CAD" => "C$",
        "CHF" => "Fr",
        "INR" => "₹",
        _ => "",
    }
}
