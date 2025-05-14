use clap::{Parser, Subcommand};
use client::ExchangeRateClient;
use colored::Colorize;
use std::env;
use std::process;

mod commands;
mod error;
mod formatters;
mod utils;

use error::CliError;

#[derive(Parser)]
#[command(
    name = "exchangerate",
    about = "Command line interface for the Exchange Rate API",
    version,
    author,
    long_about = "A command line tool to interact with the Exchange Rate API, providing currency conversion and exchange rate information."
)]
struct Cli {
    /// API key for the Exchange Rate API (can also be set via EXCHANGE_RATE_API_KEY env var)
    #[arg(long, env = "EXCHANGE_RATE_API_KEY", hide_env_values = true)]
    api_key: Option<String>,

    /// Authentication method (bearer or url)
    #[arg(long, default_value = "bearer")]
    auth_method: Option<String>,

    /// Output format (text, json, csv)
    #[arg(long, default_value = "text")]
    format: Option<String>,

    /// Disable colored output
    #[arg(long)]
    no_color: bool,

    /// Disable caching
    #[arg(long)]
    no_cache: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Get latest exchange rates for a base currency
    Latest {
        /// Base currency code (e.g., USD)
        base_currency: String,
    },

    /// Convert an amount from one currency to another
    Convert {
        /// Amount to convert
        amount: f64,

        /// Source currency code (e.g., USD)
        from_currency: String,

        /// Target currency code (e.g., EUR)
        to_currency: String,
    },

    /// Get direct conversion rate between two currencies
    Pair {
        /// Source currency code (e.g., GBP)
        from_currency: String,

        /// Target currency code (e.g., JPY)
        to_currency: String,
    },

    /// List all supported currency codes
    Codes,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Configure colored output
    if cli.no_color {
        colored::control::set_override(false);
    }

    // Run the CLI and handle errors
    if let Err(err) = run(cli).await {
        eprintln!("{} {}", "Error:".bold().red(), err);
        process::exit(1);
    }
}

async fn run(cli: Cli) -> Result<(), CliError> {
    // Get API key from args or environment
    let api_key = match cli.api_key {
        Some(key) => key,
        None => env::var("EXCHANGE_RATE_API_KEY")?,
    };

    // Create client builder
    let mut client_builder = ExchangeRateClient::builder().api_key(api_key);

    // Configure auth method if specified
    if let Some(auth_method_str) = cli.auth_method {
        let auth_method = match auth_method_str.to_lowercase().as_str() {
            "bearer" => client::AuthMethod::BearerToken,
            "url" => client::AuthMethod::InUrl,
            _ => {
                eprintln!(
                    "{} Invalid auth method: {}. Using default (bearer).",
                    "Warning:".bold().yellow(),
                    auth_method_str
                );
                client::AuthMethod::BearerToken
            }
        };
        client_builder = client_builder.auth_method(auth_method);
    }

    // Disable cache if requested
    if cli.no_cache {
        client_builder = client_builder.disable_cache();
    }

    // Build the client
    let client = client_builder.build().map_err(|e| CliError::from(e))?;

    // Execute the requested command
    match cli.command {
        Commands::Latest { base_currency } => {
            commands::latest::execute(&client, &base_currency, cli.format.as_deref()).await?
        }
        Commands::Convert {
            amount,
            from_currency,
            to_currency,
        } => {
            commands::convert::execute(
                &client,
                amount,
                &from_currency,
                &to_currency,
                cli.format.as_deref(),
            )
            .await?
        }
        Commands::Pair {
            from_currency,
            to_currency,
        } => {
            commands::pair::execute(&client, &from_currency, &to_currency, cli.format.as_deref())
                .await?
        }
        Commands::Codes => commands::codes::execute(&client, cli.format.as_deref()).await?,
    }

    Ok(())
}
