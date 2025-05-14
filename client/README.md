# Exchange Rate API Client

A Rust client for the [Exchange Rate API](https://www.exchangerate-api.com/) with support for bearer token authentication.

## Features

- **Bearer Token Authentication**: Securely authenticate with your API key in the Authorization header
- **In-URL Authentication**: Alternative authentication method with API key in the URL
- **Comprehensive API Coverage**: Access all Exchange Rate API endpoints
- **Caching Support**: Built-in caching to comply with API terms of use and improve performance
- **Idiomatic Rust**: Type-safe API with proper error handling
- **Async/Await**: Built on tokio and reqwest for efficient async operations
- **Builder Pattern**: Flexible client configuration

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
client = { path = "../client" }
tokio = { version = "1.0", features = ["full"] }
```

## Usage

### Basic Example

```rust
use client::ExchangeRateClient;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get API key from environment variable
    let api_key = env::var("EXCHANGE_RATE_API_KEY")?;
    
    // Create client with bearer token authentication (default)
    let client = ExchangeRateClient::builder()
        .api_key(api_key)
        .build()?;
    
    // Get latest rates with USD as base currency
    let rates = client.get_latest_rates("USD").await?;
    
    // Convert 100 USD to EUR
    let amount_in_eur = rates.convert_from_base(100.0, "EUR").unwrap();
    println!("100 USD = {} EUR", amount_in_eur);

    Ok(())
}
```

### Authentication Methods

The client supports two authentication methods:

#### 1. Bearer Token Authentication (Default, More Secure)

```rust
let client = ExchangeRateClient::builder()
    .api_key(api_key)
    // .auth_method(AuthMethod::BearerToken) // This is the default
    .build()?;
```

#### 2. In-URL Authentication

```rust
let client = ExchangeRateClient::builder()
    .api_key(api_key)
    .auth_method(AuthMethod::InUrl)
    .build()?;
```

### API Methods

#### Get Latest Exchange Rates

```rust
// Get latest rates with USD as base currency
let rates = client.get_latest_rates("USD").await?;

// Access individual rates
let eur_rate = rates.get_rate("EUR").unwrap();
println!("1 USD = {} EUR", eur_rate);
```

#### Currency Conversion

```rust
// Convert directly using the client
let amount_in_eur = client.convert(100.0, "USD", "EUR").await?;
println!("100 USD = {} EUR", amount_in_eur);

// Or use the response object for conversions
let rates = client.get_latest_rates("USD").await?;
let amount_in_eur = rates.convert_from_base(100.0, "EUR").unwrap();
println!("100 USD = {} EUR", amount_in_eur);
```

#### Get Pair Conversion Rate

```rust
// Get direct conversion rate between two currencies
let rate = client.get_pair_conversion("GBP", "JPY").await?;
println!("1 GBP = {} JPY", rate);
```

#### Get Supported Currency Codes

```rust
// Get all supported currencies
let currencies = client.get_supported_codes().await?;
for (code, name) in currencies {
    println!("{} - {}", code, name);
}
```

## Error Handling

The client provides comprehensive error handling through the `ExchangeRateError` enum:

```rust
match client.get_latest_rates("USD").await {
    Ok(rates) => {
        // Process rates
    },
    Err(ExchangeRateError::InvalidKey) => {
        eprintln!("The API key is invalid");
    },
    Err(ExchangeRateError::QuotaReached) => {
        eprintln!("API quota has been reached");
    },
    Err(e) => {
        eprintln!("An error occurred: {}", e);
    }
}
```

## Caching

The client includes built-in caching to comply with API terms of use and improve performance:

```rust
use exchangerate_client::{ExchangeRateClient, InMemoryCache};
use std::sync::Arc;

// Create a client with in-memory caching enabled (default)
let client = ExchangeRateClient::builder()
    .api_key(api_key)
    .build()?;

// Or explicitly configure caching
let client = ExchangeRateClient::builder()
    .api_key(api_key)
    .with_cache(Arc::new(InMemoryCache::new()))
    .cache_config(CacheConfig {
        enabled: true,
        default_ttl: chrono::Duration::hours(2), // Custom TTL
    })
    .build()?;

// Disable caching if needed
let client = ExchangeRateClient::builder()
    .api_key(api_key)
    .disable_cache()
    .build()?;
```

### Cache Backends

The client supports different cache backends:

#### 1. In-Memory Cache (Default)

```rust
use exchangerate_client::{ExchangeRateClient, InMemoryCache};
use std::sync::Arc;

let client = ExchangeRateClient::builder()
    .api_key(api_key)
    .with_cache(Arc::new(InMemoryCache::new()))
    .build()?;
```

#### 2. SQLite Cache (Optional)

Enable the `sqlite-cache` feature in your Cargo.toml:

```toml
[dependencies]
client = { path = "../client", features = ["sqlite-cache"] }
```

Then use it in your code:

```rust
use exchangerate_client::{ExchangeRateClient, SqliteCache};
use std::sync::Arc;

let sqlite_cache = SqliteCache::new("exchange_rates.db")?;
let client = ExchangeRateClient::builder()
    .api_key(api_key)
    .with_cache(Arc::new(sqlite_cache))
    .build()?;
```

## Examples

See the `examples` directory for complete usage examples:

- `basic_usage.rs`: Demonstrates basic client usage with bearer token authentication
- `in_url_auth.rs`: Shows how to use the alternative in-URL authentication method
- `caching.rs`: Demonstrates the caching functionality

Run examples with:

```bash
EXCHANGE_RATE_API_KEY=your_api_key cargo run --example basic_usage
```

## Security Considerations

- Never hardcode your API key in source code
- Use environment variables or secure configuration management
- Prefer bearer token authentication over in-URL authentication when possible
- All requests use HTTPS to ensure encrypted communication

## License

This project is licensed under the MIT License - see the LICENSE file for details.