use std::fmt;

/// Errors that can occur when using the CLI
#[derive(Debug)]
pub enum CliError {
    /// Error returned from the Exchange Rate API
    ApiError(String),
    /// Invalid currency code provided
    InvalidCurrency(String),
    /// Invalid output format specified
    InvalidFormat(String),
    /// API key not provided
    MissingApiKey,
    /// Network-related error
    NetworkError(String),
    /// Unexpected error that doesn't fit other categories
    UnexpectedError(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::ApiError(msg) => write!(
                f,
                "API Error: {}. Please check your request parameters.",
                msg
            ),
            CliError::InvalidCurrency(code) => write!(
                f,
                "Invalid currency code: '{}'. Currency codes must be 3 uppercase letters (e.g., USD, EUR, GBP).",
                code
            ),
            CliError::InvalidFormat(format) => write!(
                f,
                "Invalid output format: '{}'. Supported formats are: text, json, csv.",
                format
            ),
            CliError::MissingApiKey => write!(
                f,
                "API key not provided. Use --api-key option or set EXCHANGE_RATE_API_KEY environment variable.\nGet your API key at https://www.exchangerate-api.com/"
            ),
            CliError::NetworkError(msg) => write!(
                f,
                "Network Error: {}. Please check your internet connection and try again.",
                msg
            ),
            CliError::UnexpectedError(msg) => write!(
                f,
                "Unexpected Error: {}. Please report this issue if it persists.",
                msg
            ),
        }
    }
}

impl std::error::Error for CliError {}

impl From<client::ExchangeRateError> for CliError {
    fn from(err: client::ExchangeRateError) -> Self {
        match err {
            client::ExchangeRateError::MissingApiKey => CliError::MissingApiKey,
            client::ExchangeRateError::UnsupportedCode => {
                CliError::InvalidCurrency("Unsupported currency code".to_string())
            }
            client::ExchangeRateError::InvalidKey => CliError::ApiError(
                "Invalid API key. Please check your API key and try again".to_string(),
            ),
            client::ExchangeRateError::InactiveAccount => CliError::ApiError(
                "Your account is inactive. Please check your subscription status".to_string(),
            ),
            client::ExchangeRateError::QuotaReached => CliError::ApiError(
                "API quota reached. Please upgrade your plan or try again later".to_string(),
            ),
            client::ExchangeRateError::MalformedRequest => {
                CliError::ApiError("Malformed request. This is likely a bug in the CLI".to_string())
            }
            client::ExchangeRateError::HttpClientError(e) => {
                if e.is_timeout() {
                    CliError::NetworkError(
                        "Request timed out. The server might be busy, please try again later"
                            .to_string(),
                    )
                } else if e.is_connect() {
                    CliError::NetworkError(
                        "Connection failed. Please check your internet connection".to_string(),
                    )
                } else {
                    CliError::NetworkError(format!("HTTP client error: {}", e))
                }
            }
            client::ExchangeRateError::HttpError(status) => match status.as_u16() {
                429 => CliError::ApiError("Too many requests. Please try again later".to_string()),
                403 => {
                    CliError::ApiError("Access forbidden. Please check your API key".to_string())
                }
                404 => CliError::ApiError(
                    "Resource not found. The requested endpoint might not exist".to_string(),
                ),
                500..=599 => CliError::ApiError(
                    "Server error. The API service might be experiencing issues".to_string(),
                ),
                _ => CliError::NetworkError(format!("HTTP error: {}", status)),
            },
            client::ExchangeRateError::JsonError(e) => {
                CliError::UnexpectedError(format!("Failed to parse API response: {}", e))
            }
            client::ExchangeRateError::CacheError(e) => {
                CliError::UnexpectedError(format!("Cache error: {}. Try using --no-cache", e))
            }
            _ => CliError::UnexpectedError(format!("Unexpected error: {}", err)),
        }
    }
}

impl From<std::env::VarError> for CliError {
    fn from(_: std::env::VarError) -> Self {
        CliError::MissingApiKey
    }
}

impl From<serde_json::Error> for CliError {
    fn from(err: serde_json::Error) -> Self {
        CliError::UnexpectedError(format!("JSON Error: {}", err))
    }
}

impl From<std::io::Error> for CliError {
    fn from(err: std::io::Error) -> Self {
        CliError::UnexpectedError(format!("IO Error: {}", err))
    }
}
