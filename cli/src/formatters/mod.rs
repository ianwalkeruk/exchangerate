use client::ExchangeRateResponse;
use colored::Colorize;
use prettytable::{Cell, Row, Table, format};
use serde_json::json;
use std::collections::HashMap;

use crate::error::CliError;
use crate::utils;

pub enum OutputFormat {
    Text,
    Json,
    Csv,
}

impl TryFrom<&str> for OutputFormat {
    type Error = CliError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "text" => Ok(OutputFormat::Text),
            "json" => Ok(OutputFormat::Json),
            "csv" => Ok(OutputFormat::Csv),
            _ => Err(CliError::InvalidFormat(s.to_string())),
        }
    }
}

pub fn format_latest_rates(
    response: &ExchangeRateResponse,
    format: Option<&str>,
) -> Result<String, CliError> {
    let format = match format {
        Some(fmt) => OutputFormat::try_from(fmt)?,
        None => OutputFormat::Text,
    };

    match format {
        OutputFormat::Text => format_latest_rates_text(response),
        OutputFormat::Json => format_latest_rates_json(response),
        OutputFormat::Csv => format_latest_rates_csv(response),
    }
}

fn format_latest_rates_text(response: &ExchangeRateResponse) -> Result<String, CliError> {
    let mut output = String::new();

    // Header
    output.push_str(&format!(
        "{} {}\n",
        "Base Currency:".bold().green(),
        response.base_code
    ));
    output.push_str(&format!(
        "{} {}\n",
        "Last Updated:".bold().green(),
        response.time_last_update_utc
    ));
    output.push_str(&format!(
        "{} {}\n\n",
        "Next Update:".bold().green(),
        response.time_next_update_utc
    ));

    // Create table
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);

    // Add header row
    table.set_titles(Row::new(vec![
        Cell::new("Currency").style_spec("Fb"),
        Cell::new("Code").style_spec("Fb"),
        Cell::new("Rate").style_spec("Fb"),
    ]));

    // Add data rows
    let mut rates: Vec<(&String, &f64)> = response.conversion_rates.iter().collect();
    rates.sort_by(|a, b| a.0.cmp(b.0));

    for (code, rate) in rates {
        table.add_row(Row::new(vec![
            Cell::new(""),
            Cell::new(code),
            Cell::new(&format!("{:.4}", rate)),
        ]));
    }

    // Convert table to string
    output.push_str(&table.to_string());
    output.push_str(&format!(
        "\n{} {}\n",
        "Total Currencies:".bold().green(),
        response.conversion_rates.len()
    ));

    Ok(output)
}

fn format_latest_rates_json(response: &ExchangeRateResponse) -> Result<String, CliError> {
    let json = json!({
        "base_currency": response.base_code,
        "last_updated": response.time_last_update_utc,
        "next_update": response.time_next_update_utc,
        "rates": response.conversion_rates
    });

    Ok(serde_json::to_string_pretty(&json)?)
}

fn format_latest_rates_csv(response: &ExchangeRateResponse) -> Result<String, CliError> {
    let mut output = String::new();

    // Header
    output.push_str("Currency Code,Rate\n");

    // Data rows
    let mut rates: Vec<(&String, &f64)> = response.conversion_rates.iter().collect();
    rates.sort_by(|a, b| a.0.cmp(b.0));

    for (code, rate) in rates {
        output.push_str(&format!("{},{:.4}\n", code, rate));
    }

    Ok(output)
}

pub fn format_conversion(
    amount: f64,
    from_currency: &str,
    to_currency: &str,
    converted_amount: f64,
    rate: f64,
    format: Option<&str>,
) -> Result<String, CliError> {
    let format = match format {
        Some(fmt) => OutputFormat::try_from(fmt)?,
        None => OutputFormat::Text,
    };

    match format {
        OutputFormat::Text => {
            format_conversion_text(amount, from_currency, to_currency, converted_amount, rate)
        }
        OutputFormat::Json => {
            format_conversion_json(amount, from_currency, to_currency, converted_amount, rate)
        }
        OutputFormat::Csv => {
            format_conversion_csv(amount, from_currency, to_currency, converted_amount, rate)
        }
    }
}

fn format_conversion_text(
    amount: f64,
    from_currency: &str,
    to_currency: &str,
    converted_amount: f64,
    rate: f64,
) -> Result<String, CliError> {
    let mut output = String::new();

    // Format amounts with appropriate currency symbols and decimal places
    let formatted_from_amount = utils::format_currency_amount(amount, from_currency);
    let formatted_to_amount = utils::format_currency_amount(converted_amount, to_currency);

    output.push_str(&format!(
        "{} {} {} = {} {}\n",
        "Conversion:".bold().green(),
        formatted_from_amount,
        from_currency,
        formatted_to_amount,
        to_currency
    ));
    output.push_str(&format!(
        "{} {:.4} {} per {}\n",
        "Rate:".bold().green(),
        rate,
        to_currency,
        from_currency
    ));

    Ok(output)
}

fn format_conversion_json(
    amount: f64,
    from_currency: &str,
    to_currency: &str,
    converted_amount: f64,
    rate: f64,
) -> Result<String, CliError> {
    let json = json!({
        "amount": amount,
        "from_currency": from_currency,
        "to_currency": to_currency,
        "converted_amount": converted_amount,
        "rate": rate
    });

    Ok(serde_json::to_string_pretty(&json)?)
}

fn format_conversion_csv(
    amount: f64,
    from_currency: &str,
    to_currency: &str,
    converted_amount: f64,
    rate: f64,
) -> Result<String, CliError> {
    let mut output = String::new();

    // Header
    output.push_str("Amount,From Currency,To Currency,Converted Amount,Rate\n");

    // Data row
    output.push_str(&format!(
        "{:.2},{},{},{:.2},{:.4}\n",
        amount, from_currency, to_currency, converted_amount, rate
    ));

    Ok(output)
}

pub fn format_pair_rate(
    from_currency: &str,
    to_currency: &str,
    rate: f64,
    format: Option<&str>,
) -> Result<String, CliError> {
    let format = match format {
        Some(fmt) => OutputFormat::try_from(fmt)?,
        None => OutputFormat::Text,
    };

    match format {
        OutputFormat::Text => format_pair_rate_text(from_currency, to_currency, rate),
        OutputFormat::Json => format_pair_rate_json(from_currency, to_currency, rate),
        OutputFormat::Csv => format_pair_rate_csv(from_currency, to_currency, rate),
    }
}

fn format_pair_rate_text(
    from_currency: &str,
    to_currency: &str,
    rate: f64,
) -> Result<String, CliError> {
    let mut output = String::new();

    output.push_str(&format!(
        "{} 1 {} = {:.4} {}\n",
        "Conversion Rate:".bold().green(),
        from_currency,
        rate,
        to_currency
    ));

    Ok(output)
}

fn format_pair_rate_json(
    from_currency: &str,
    to_currency: &str,
    rate: f64,
) -> Result<String, CliError> {
    let json = json!({
        "from_currency": from_currency,
        "to_currency": to_currency,
        "rate": rate
    });

    Ok(serde_json::to_string_pretty(&json)?)
}

fn format_pair_rate_csv(
    from_currency: &str,
    to_currency: &str,
    rate: f64,
) -> Result<String, CliError> {
    let mut output = String::new();

    // Header
    output.push_str("From Currency,To Currency,Rate\n");

    // Data row
    output.push_str(&format!("{},{},{:.4}\n", from_currency, to_currency, rate));

    Ok(output)
}

pub fn format_currency_codes(
    codes: &[(String, String)],
    format: Option<&str>,
) -> Result<String, CliError> {
    let format = match format {
        Some(fmt) => OutputFormat::try_from(fmt)?,
        None => OutputFormat::Text,
    };

    match format {
        OutputFormat::Text => format_currency_codes_text(codes),
        OutputFormat::Json => format_currency_codes_json(codes),
        OutputFormat::Csv => format_currency_codes_csv(codes),
    }
}

fn format_currency_codes_text(codes: &[(String, String)]) -> Result<String, CliError> {
    let mut output = String::new();

    // Header
    output.push_str(&format!(
        "{}\n\n",
        "Supported Currency Codes".bold().green()
    ));

    // Create table
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);

    // Add header row
    table.set_titles(Row::new(vec![
        Cell::new("Code").style_spec("Fb"),
        Cell::new("Currency").style_spec("Fb"),
    ]));

    // Add data rows
    for (code, name) in codes {
        table.add_row(Row::new(vec![Cell::new(code), Cell::new(name)]));
    }

    // Convert table to string
    output.push_str(&table.to_string());
    output.push_str(&format!(
        "\n{} {}\n",
        "Total Currencies:".bold().green(),
        codes.len()
    ));

    Ok(output)
}

fn format_currency_codes_json(codes: &[(String, String)]) -> Result<String, CliError> {
    let mut map = HashMap::new();
    for (code, name) in codes {
        map.insert(code, name);
    }

    let json = json!({
        "currencies": map,
        "count": codes.len()
    });

    Ok(serde_json::to_string_pretty(&json)?)
}

fn format_currency_codes_csv(codes: &[(String, String)]) -> Result<String, CliError> {
    let mut output = String::new();

    // Header
    output.push_str("Code,Currency\n");

    // Data rows
    for (code, name) in codes {
        output.push_str(&format!("{},{}\n", code, name));
    }

    Ok(output)
}
