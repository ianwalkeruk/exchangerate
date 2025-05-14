use once_cell::sync::Lazy;
use std::collections::HashSet;

use crate::error::CliError;
use colored::Colorize;

/// A static set of common currency codes for validation
static COMMON_CURRENCIES: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    let mut set = HashSet::new();
    // Add common currency codes
    set.insert("USD"); // US Dollar
    set.insert("EUR"); // Euro
    set.insert("GBP"); // British Pound
    set.insert("JPY"); // Japanese Yen
    set.insert("AUD"); // Australian Dollar
    set.insert("CAD"); // Canadian Dollar
    set.insert("CHF"); // Swiss Franc
    set.insert("CNY"); // Chinese Yuan
    set.insert("HKD"); // Hong Kong Dollar
    set.insert("NZD"); // New Zealand Dollar
    set.insert("SEK"); // Swedish Krona
    set.insert("KRW"); // South Korean Won
    set.insert("SGD"); // Singapore Dollar
    set.insert("NOK"); // Norwegian Krone
    set.insert("MXN"); // Mexican Peso
    set.insert("INR"); // Indian Rupee
    set.insert("RUB"); // Russian Ruble
    set.insert("ZAR"); // South African Rand
    set.insert("TRY"); // Turkish Lira
    set.insert("BRL"); // Brazilian Real
    set.insert("TWD"); // Taiwan Dollar
    set.insert("DKK"); // Danish Krone
    set.insert("PLN"); // Polish Zloty
    set.insert("THB"); // Thai Baht
    set.insert("IDR"); // Indonesian Rupiah
    set
});

/// Validates a currency code format (3 uppercase letters) and provides suggestions for common typos
pub fn validate_currency_code(code: &str) -> Result<(), CliError> {
    // Check basic format (3 uppercase letters)
    if code.len() != 3 {
        return Err(CliError::InvalidCurrency(format!(
            "{} (should be 3 letters)",
            code
        )));
    }

    if !code.chars().all(|c| c.is_ascii_alphabetic()) {
        return Err(CliError::InvalidCurrency(format!(
            "{} (should only contain letters)",
            code
        )));
    }

    // Convert to uppercase for comparison
    let uppercase_code = code.to_uppercase();

    // If the code is already uppercase and valid, return Ok
    if code == uppercase_code && code.chars().all(|c| c.is_ascii_uppercase()) {
        return Ok(());
    }

    // If the uppercase version is a valid currency, suggest it
    if COMMON_CURRENCIES.contains(uppercase_code.as_str()) {
        return Err(CliError::InvalidCurrency(format!(
            "{} (did you mean {}?)",
            code, uppercase_code
        )));
    }

    // If we get here, the code is not valid
    Err(CliError::InvalidCurrency(code.to_string()))
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

    // Format with appropriate decimal places
    // JPY and other currencies typically don't use decimal places
    if currency == "JPY" || currency == "KRW" || currency == "IDR" {
        format!("{}{:.0}", symbol, amount)
    } else {
        format!("{}{:.2}", symbol, amount)
    }
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
        "KRW" => "₩",
        "RUB" => "₽",
        "TRY" => "₺",
        "BRL" => "R$",
        "ZAR" => "R",
        "SEK" | "NOK" | "DKK" => "kr",
        "PLN" => "zł",
        "THB" => "฿",
        "MXN" => "Mex$",
        "SGD" => "S$",
        "HKD" => "HK$",
        "NZD" => "NZ$",
        _ => "",
    }
}
