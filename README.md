# Exchange Rate CLI

A command-line interface for the [Exchange Rate API](https://www.exchangerate-api.com/), providing easy access to currency conversion and exchange rate information.

## Features

- Get latest exchange rates for a base currency
- Convert amounts between currencies
- Get direct conversion rates between currency pairs
- List all supported currency codes
- Multiple output formats (text, JSON, CSV)
- Colored output for better readability
- Caching support to reduce API calls

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/ianwalkeruk/exchangerate.git
cd exchangerate

# Build the CLI
cargo build --release

# The binary will be available at target/release/exchangerate-cli
```

## Usage

```bash
# Set your API key as an environment variable (recommended)
export EXCHANGE_RATE_API_KEY="your-api-key"

# Or pass it directly (less secure)
exchangerate-cli --api-key "your-api-key" [COMMAND]
```

### Commands

#### Get Latest Rates

```bash
# Get latest rates with USD as base currency
exchangerate-cli latest USD

# Get latest rates with EUR as base currency in JSON format
exchangerate-cli --format json latest EUR
```

#### Convert Currency

```bash
# Convert 100 USD to EUR
exchangerate-cli convert 100 USD EUR

# Convert 50 GBP to JPY in CSV format
exchangerate-cli --format csv convert 50 GBP JPY
```

#### Get Currency Pair Rate

```bash
# Get the exchange rate from USD to EUR
exchangerate-cli pair USD EUR
```

#### List Currency Codes

```bash
# List all supported currency codes
exchangerate-cli codes

# List all supported currency codes in JSON format
exchangerate-cli --format json codes
```

### Options

- `--api-key <API_KEY>`: API key for the Exchange Rate API
- `--auth-method <METHOD>`: Authentication method (bearer or url, default: bearer)
- `--format <FORMAT>`: Output format (text, json, csv, default: text)
- `--no-color`: Disable colored output
- `--no-cache`: Disable caching

## Environment Variables

- `EXCHANGE_RATE_API_KEY`: Your Exchange Rate API key

## License

This project is licensed under the MIT License - see the LICENSE file for details.
