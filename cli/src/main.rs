use clap::{Parser, Subcommand};
use client::ExchangeRateClient;
use colored::Colorize;
use std::env;
use std::process;

mod commands;
mod config;
mod error;
mod formatters;
mod utils;

use config::Config;
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

    /// Manage configuration
    #[command(
        about = "Manage configuration",
        long_about = "View, set, or reset configuration options. Configuration is stored in ~/.config/exchangerate/config.json."
    )]
    Config {
        /// Configuration action to perform
        #[command(subcommand)]
        action: Option<ConfigAction>,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    /// View current configuration
    #[command(
        about = "View current configuration",
        long_about = "Display the current configuration settings."
    )]
    View,

    /// Set a configuration value
    #[command(
        about = "Set a configuration value",
        long_about = "Set a specific configuration value and save it to the configuration file."
    )]
    Set {
        /// Configuration key to set
        #[arg(
            help = "The configuration key to set (api_key, auth_method, default_format, use_color, use_cache)"
        )]
        key: String,

        /// Value to set
        #[arg(help = "The value to set for the specified key")]
        value: String,
    },

    /// Reset configuration to defaults
    #[command(
        about = "Reset configuration to defaults",
        long_about = "Reset all configuration values to their defaults."
    )]
    Reset,
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
    // Load configuration
    let config = Config::load()?;

    // Get API key from args, environment, or config
    let api_key = match cli.api_key {
        Some(key) => key,
        None => match env::var("EXCHANGE_RATE_API_KEY") {
            Ok(key) => key,
            Err(_) => match &config.api_key {
                Some(key) => key.clone(),
                None => return Err(CliError::MissingApiKey),
            },
        },
    };

    if cli.verbose {
        println!("{} Using Exchange Rate API", "Info:".bold().blue());
    }

    // Create client builder
    let mut client_builder = ExchangeRateClient::builder().api_key(api_key);

    // Configure auth method from args or config
    let auth_method_str = cli.auth_method.or_else(|| config.auth_method.clone());
    if let Some(auth_method_str) = auth_method_str {
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

    // Configure cache from args or config
    let use_cache = !cli.no_cache && config.use_cache.unwrap_or(true);
    if !use_cache {
        if cli.verbose {
            println!("{} Cache disabled", "Info:".bold().blue());
        }
        client_builder = client_builder.disable_cache();
    } else if cli.verbose {
        println!("{} Using cache to reduce API calls", "Info:".bold().blue());
    }

    // Build the client
    let client = client_builder.build().map_err(|e| CliError::from(e))?;

    // Get output format from args or config
    let format = cli
        .format
        .as_deref()
        .or_else(|| config.default_format.as_deref());

    // Execute the requested command
    match &cli.command {
        Commands::Latest { base_currency } => {
            if cli.verbose {
                println!(
                    "{} Fetching latest rates with base currency: {}",
                    "Info:".bold().blue(),
                    base_currency
                );
            }
            commands::latest::execute(&client, base_currency, format, cli.verbose).await?
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
                *amount,
                from_currency,
                to_currency,
                format,
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
            commands::pair::execute(&client, from_currency, to_currency, format, cli.verbose)
                .await?
        }
        Commands::Codes => {
            if cli.verbose {
                println!(
                    "{} Fetching supported currency codes",
                    "Info:".bold().blue()
                );
            }
            commands::codes::execute(&client, format, cli.verbose).await?
        }
        Commands::Config { action } => handle_config_command(action, &config, cli.verbose)?,
    }

    if cli.verbose {
        println!("{} Command completed successfully", "Info:".bold().blue());
    }

    Ok(())
}

/// Handle the config command
fn handle_config_command(
    action: &Option<ConfigAction>,
    current_config: &Config,
    verbose: bool,
) -> Result<(), CliError> {
    match action {
        Some(ConfigAction::View) => {
            // Display current configuration
            println!("{}", "Current Configuration:".bold().green());
            println!(
                "API Key: {}",
                match &current_config.api_key {
                    Some(key) => format!("{}...", &key[..min(8, key.len())]),
                    None => "Not set".to_string(),
                }
            );
            println!(
                "Auth Method: {}",
                current_config.auth_method.as_deref().unwrap_or("Not set")
            );
            println!(
                "Default Format: {}",
                current_config
                    .default_format
                    .as_deref()
                    .unwrap_or("Not set")
            );
            println!("Use Color: {}", current_config.use_color.unwrap_or(true));
            println!("Use Cache: {}", current_config.use_cache.unwrap_or(true));

            // Show config file location
            let config_path = config::get_config_path()?;
            println!("\n{}", "Config File Location:".bold().green());
            println!("{}", config_path.display());
        }
        Some(ConfigAction::Set { key, value }) => {
            // Create a new config based on the current one
            let mut new_config = current_config.clone();

            // Update the specified key
            match key.as_str() {
                "api_key" => {
                    new_config.api_key = Some(value.clone());
                    println!("API key updated");
                }
                "auth_method" => match value.to_lowercase().as_str() {
                    "bearer" | "url" => {
                        new_config.auth_method = Some(value.to_lowercase());
                        println!("Auth method set to: {}", value.to_lowercase());
                    }
                    _ => {
                        return Err(CliError::InvalidConfigValue(format!(
                            "Invalid auth method: {}. Valid values are 'bearer' or 'url'.",
                            value
                        )));
                    }
                },
                "default_format" => match value.to_lowercase().as_str() {
                    "text" | "json" | "csv" => {
                        new_config.default_format = Some(value.to_lowercase());
                        println!("Default format set to: {}", value.to_lowercase());
                    }
                    _ => {
                        return Err(CliError::InvalidConfigValue(format!(
                            "Invalid format: {}. Valid values are 'text', 'json', or 'csv'.",
                            value
                        )));
                    }
                },
                "use_color" => match value.to_lowercase().as_str() {
                    "true" | "yes" | "1" => {
                        new_config.use_color = Some(true);
                        println!("Color output enabled");
                    }
                    "false" | "no" | "0" => {
                        new_config.use_color = Some(false);
                        println!("Color output disabled");
                    }
                    _ => {
                        return Err(CliError::InvalidConfigValue(format!(
                            "Invalid boolean value: {}. Use 'true' or 'false'.",
                            value
                        )));
                    }
                },
                "use_cache" => match value.to_lowercase().as_str() {
                    "true" | "yes" | "1" => {
                        new_config.use_cache = Some(true);
                        println!("Caching enabled");
                    }
                    "false" | "no" | "0" => {
                        new_config.use_cache = Some(false);
                        println!("Caching disabled");
                    }
                    _ => {
                        return Err(CliError::InvalidConfigValue(format!(
                            "Invalid boolean value: {}. Use 'true' or 'false'.",
                            value
                        )));
                    }
                },
                _ => {
                    return Err(CliError::InvalidConfigKey(format!(
                        "Invalid configuration key: {}. Valid keys are 'api_key', 'auth_method', 'default_format', 'use_color', 'use_cache'.",
                        key
                    )));
                }
            }

            // Save the updated configuration
            new_config.save()?;

            if verbose {
                println!(
                    "{} Configuration updated successfully",
                    "Info:".bold().blue()
                );
            }
        }
        Some(ConfigAction::Reset) => {
            // Create a new default configuration
            let new_config = Config::default();

            // Save the default configuration
            new_config.save()?;

            println!("Configuration reset to defaults");

            if verbose {
                println!("{} Configuration reset successfully", "Info:".bold().blue());
            }
        }
        None => {
            // If no action is specified, show the current configuration
            handle_config_command(&Some(ConfigAction::View), current_config, verbose)?;
        }
    }

    Ok(())
}

use std::cmp::min;
