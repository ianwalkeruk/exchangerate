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
    long_about = "A command line tool to interact with the Exchange Rate API (https://www.exchangerate-api.com/), providing currency conversion and exchange rate information. Requires an API key which can be obtained for free from the Exchange Rate API website."
)]
struct Cli {
    /// API key for the Exchange Rate API (can also be set via EXCHANGE_RATE_API_KEY env var)
    #[arg(
        long,
        value_name = "API_KEY",
        help = "Your Exchange Rate API key. Get one at https://www.exchangerate-api.com/"
    )]
    api_key: Option<String>,

    /// Authentication method (bearer or url)
    #[arg(
        long,
        default_value = "bearer",
        help = "Method to authenticate with the API. 'bearer' (more secure) sends the API key in the Authorization header. 'url' includes the API key in the URL."
    )]
    auth_method: Option<String>,

    /// Output format (text, json, csv)
    #[arg(
        long,
        default_value = "text",
        help = "Format for command output. 'text' for human-readable output, 'json' for JSON format, 'csv' for comma-separated values."
    )]
    format: Option<String>,

    /// Disable colored output
    #[arg(
        long,
        help = "Disable colored output in text mode. Useful for scripts or terminals that don't support ANSI colors."
    )]
    no_color: bool,

    /// Disable caching
    #[arg(
        long,
        help = "Disable caching of API responses. By default, responses are cached to reduce API calls and improve performance."
    )]
    no_cache: bool,

    /// Enable verbose output
    #[arg(
        short,
        long,
        help = "Enable verbose output with additional information about the request and response."
    )]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Get latest exchange rates for a base currency
    #[command(
        about = "Get latest exchange rates for a base currency",
        long_about = "Retrieves the latest exchange rates for a specified base currency. Returns rates for all available currencies."
    )]
    Latest {
        /// Base currency code (e.g., USD)
        #[arg(
            help = "The base currency code (e.g., USD, EUR, GBP). Must be a valid 3-letter currency code."
        )]
        base_currency: String,
    },

    /// Convert an amount from one currency to another
    #[command(
        about = "Convert an amount from one currency to another",
        long_about = "Converts a specified amount from one currency to another using the latest exchange rates."
    )]
    Convert {
        /// Amount to convert
        #[arg(help = "The amount to convert. Can be any positive number.")]
        amount: f64,

        /// Source currency code (e.g., USD)
        #[arg(
            help = "The source currency code (e.g., USD, EUR, GBP). Must be a valid 3-letter currency code."
        )]
        from_currency: String,

        /// Target currency code (e.g., EUR)
        #[arg(
            help = "The target currency code (e.g., USD, EUR, GBP). Must be a valid 3-letter currency code."
        )]
        to_currency: String,
    },

    /// Get direct conversion rate between two currencies
    #[command(
        about = "Get direct conversion rate between two currencies",
        long_about = "Retrieves the direct conversion rate between two specified currencies."
    )]
    Pair {
        /// Source currency code (e.g., GBP)
        #[arg(
            help = "The source currency code (e.g., USD, EUR, GBP). Must be a valid 3-letter currency code."
        )]
        from_currency: String,

        /// Target currency code (e.g., JPY)
        #[arg(
            help = "The target currency code (e.g., USD, EUR, GBP). Must be a valid 3-letter currency code."
        )]
        to_currency: String,
    },

    /// List all supported currency codes
    #[command(
        about = "List all supported currency codes",
        long_about = "Lists all currency codes supported by the Exchange Rate API along with their full names."
    )]
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

    if cli.verbose {
        println!("{} Using Exchange Rate API", "Info:".bold().blue());
    }

    // Create client builder
    let mut client_builder = ExchangeRateClient::builder().api_key(api_key);

    // Configure auth method if specified
    if let Some(auth_method_str) = cli.auth_method {
        let auth_method = match auth_method_str.to_lowercase().as_str() {
            "bearer" => {
                if cli.verbose {
                    println!(
                        "{} Using bearer token authentication",
                        "Info:".bold().blue()
                    );
                }
                client::AuthMethod::BearerToken
            }
            "url" => {
                if cli.verbose {
                    println!("{} Using URL authentication", "Info:".bold().blue());
                }
                client::AuthMethod::InUrl
            }
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
        if cli.verbose {
            println!("{} Cache disabled", "Info:".bold().blue());
        }
        client_builder = client_builder.disable_cache();
    } else if cli.verbose {
        println!("{} Using cache to reduce API calls", "Info:".bold().blue());
    }

    // Build the client
    let client = client_builder.build().map_err(|e| CliError::from(e))?;

    // Execute the requested command
    match cli.command {
        Commands::Latest { base_currency } => {
            if cli.verbose {
                println!(
                    "{} Fetching latest rates with base currency: {}",
                    "Info:".bold().blue(),
                    base_currency
                );
            }
            commands::latest::execute(&client, &base_currency, cli.format.as_deref(), cli.verbose)
                .await?
        }
        Commands::Convert {
            amount,
            from_currency,
            to_currency,
        } => {
            if cli.verbose {
                println!(
                    "{} Converting {:.2} {} to {}",
                    "Info:".bold().blue(),
                    amount,
                    from_currency,
                    to_currency
                );
            }
            commands::convert::execute(
                &client,
                amount,
                &from_currency,
                &to_currency,
                cli.format.as_deref(),
                cli.verbose,
            )
            .await?
        }
        Commands::Pair {
            from_currency,
            to_currency,
        } => {
            if cli.verbose {
                println!(
                    "{} Getting exchange rate from {} to {}",
                    "Info:".bold().blue(),
                    from_currency,
                    to_currency
                );
            }
            commands::pair::execute(
                &client,
                &from_currency,
                &to_currency,
                cli.format.as_deref(),
                cli.verbose,
            )
            .await?
        }
        Commands::Codes => {
            if cli.verbose {
                println!(
                    "{} Fetching supported currency codes",
                    "Info:".bold().blue()
                );
            }
            commands::codes::execute(&client, cli.format.as_deref(), cli.verbose).await?
        }
    }

    if cli.verbose {
        println!("{} Command completed successfully", "Info:".bold().blue());
    }

    Ok(())
}
