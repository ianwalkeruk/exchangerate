mod models;

pub use models::{CurrencyCode, ExchangeRateResponse};

use std::time::Duration;
use thiserror::Error;

/// Authentication methods supported by the Exchange Rate API
#[derive(Debug, Clone, Copy)]
pub enum AuthMethod {
    /// API key is included in the URL (less secure but simpler)
    /// Example: https://v6.exchangerate-api.com/v6/YOUR-API-KEY/latest/USD
    InUrl,

    /// API key is passed as a bearer token in the Authorization header (more secure)
    /// Example: GET https://v6.exchangerate-api.com/v6/latest/USD
    /// With header: Authorization: Bearer YOUR-API-KEY
    BearerToken,
}

impl Default for AuthMethod {
    fn default() -> Self {
        // Default to the more secure method
        Self::BearerToken
    }
}

/// Errors that can occur when using the Exchange Rate API client
#[derive(Debug, Error)]
pub enum ExchangeRateError {
    #[error("Missing API key")]
    MissingApiKey,

    #[error("Unsupported currency code")]
    UnsupportedCode,

    #[error("Malformed request")]
    MalformedRequest,

    #[error("Invalid API key")]
    InvalidKey,

    #[error("Inactive account")]
    InactiveAccount,

    #[error("API quota reached")]
    QuotaReached,

    #[error("HTTP client error: {0}")]
    HttpClientError(#[from] reqwest::Error),

    #[error("HTTP error: {0}")]
    HttpError(reqwest::StatusCode),

    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),
}
