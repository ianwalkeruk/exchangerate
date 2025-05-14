mod models;
#[cfg(test)]
mod tests;

pub use models::{CurrencyCode, ExchangeRateResponse};

use std::time::Duration;
use thiserror::Error;

/// Authentication methods supported by the Exchange Rate API
#[derive(Debug, Clone, Copy)]
pub enum AuthMethod {
    /// API key is included in the URL (less secure but simpler)
    /// Example: <https://v6.exchangerate-api.com/v6/YOUR-API-KEY/latest/USD>
    InUrl,

    /// API key is passed as a bearer token in the Authorization header (more secure)
    /// Example: GET <https://v6.exchangerate-api.com/v6/latest/USD>
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

/// # Exchange Rate API Client
///
/// A Rust client for the Exchange Rate API (<https://www.exchangerate-api.com/>)
///
/// ## Authentication
///
/// This client supports two authentication methods:
///
/// 1. **Bearer Token Authentication (Default, More Secure)**
///    - API key is passed in the Authorization header
///    - Prevents API key from appearing in logs or browser history
///
/// 2. **In-URL Authentication**
///    - API key is included directly in the URL
///    - Simpler but less secure as the API key may appear in logs
///
/// ## Security Considerations
///
/// - Never hardcode your API key in source code
/// - Use environment variables or secure configuration management
/// - All requests use HTTPS to ensure encrypted communication
///
/// ## Example
///
/// ```rust,no_run
/// use client::{ExchangeRateClient, AuthMethod};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Create a client with bearer token authentication (default)
///     let client = ExchangeRateClient::builder()
///         .api_key(std::env::var("EXCHANGE_RATE_API_KEY")?)
///         .build()?;
///
///     // Get latest rates with USD as base currency
///     let rates = client.get_latest_rates("USD").await?;
///     
///     // Convert 100 USD to EUR
///     let amount_in_eur = rates.convert_from_base(100.0, "EUR").unwrap();
///     println!("100 USD = {} EUR", amount_in_eur);
///
///     Ok(())
/// }
/// ```
pub struct ExchangeRateClient {
    api_key: String,
    base_url: String,
    auth_method: AuthMethod,
    http_client: reqwest::Client,
}

/// Builder for creating an `ExchangeRateClient` with custom configuration
pub struct ExchangeRateClientBuilder {
    api_key: Option<String>,
    base_url: Option<String>,
    auth_method: AuthMethod,
    timeout: Option<Duration>,
}

impl Default for ExchangeRateClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ExchangeRateClientBuilder {
    /// Create a new builder with default settings
    #[must_use]
    pub fn new() -> Self {
        Self {
            api_key: None,
            base_url: Some("https://v6.exchangerate-api.com/v6".to_string()),
            auth_method: AuthMethod::BearerToken, // Default to more secure method
            timeout: Some(Duration::from_secs(30)),
        }
    }

    /// Set the API key for authentication
    #[must_use]
    pub fn api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Set the authentication method
    #[must_use]
    pub const fn auth_method(mut self, auth_method: AuthMethod) -> Self {
        self.auth_method = auth_method;
        self
    }

    /// Set a custom base URL (useful for testing or if the API URL changes)
    #[must_use]
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    /// Set a custom timeout for HTTP requests
    #[must_use]
    pub const fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Build the client with the configured settings
    ///
    /// # Errors
    ///
    /// Returns an error if the API key is not provided or if the HTTP client cannot be created
    pub fn build(self) -> Result<ExchangeRateClient, ExchangeRateError> {
        let api_key = self.api_key.ok_or(ExchangeRateError::MissingApiKey)?;

        // Create HTTP client with appropriate timeout
        let mut client_builder = reqwest::Client::builder();
        if let Some(timeout) = self.timeout {
            client_builder = client_builder.timeout(timeout);
        }

        let http_client = client_builder
            .build()
            .map_err(ExchangeRateError::HttpClientError)?;

        Ok(ExchangeRateClient {
            api_key,
            base_url: self
                .base_url
                .unwrap_or_else(|| "https://v6.exchangerate-api.com/v6".to_string()),
            auth_method: self.auth_method,
            http_client,
        })
    }
}

impl ExchangeRateClient {
    /// Creates a new client builder
    #[must_use]
    pub fn builder() -> ExchangeRateClientBuilder {
        ExchangeRateClientBuilder::new()
    }

    /// Constructs the appropriate URL based on the authentication method
    fn build_url(&self, endpoint: &str, params: &[&str]) -> String {
        match self.auth_method {
            AuthMethod::InUrl => {
                // Include API key in URL
                format!(
                    "{}/{}/{}/{}",
                    self.base_url,
                    self.api_key,
                    endpoint,
                    params.join("/")
                )
            }
            AuthMethod::BearerToken => {
                // Omit API key from URL
                format!("{}/{}/{}", self.base_url, endpoint, params.join("/"))
            }
        }
    }

    /// Get latest exchange rates for a base currency
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails, the response cannot be parsed,
    /// or the API returns an error response
    pub async fn get_latest_rates(
        &self,
        base_code: &str,
    ) -> Result<ExchangeRateResponse, ExchangeRateError> {
        let url = self.build_url("latest", &[base_code]);

        let mut request_builder = self.http_client.get(&url);

        // Add authorization header if using bearer token auth
        if let AuthMethod::BearerToken = self.auth_method {
            request_builder = request_builder.header(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", self.api_key),
            );
        }

        let response = request_builder
            .send()
            .await
            .map_err(ExchangeRateError::HttpClientError)?;

        // Check for HTTP errors
        if !response.status().is_success() {
            return Err(ExchangeRateError::HttpError(response.status()));
        }

        // Parse the response
        let exchange_rate_response = response
            .json::<ExchangeRateResponse>()
            .await
            .map_err(ExchangeRateError::HttpClientError)?;

        Ok(exchange_rate_response)
    }

    /// Convert an amount from one currency to another
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails, the response cannot be parsed,
    /// the API returns an error response, or if the target currency is not supported
    pub async fn convert(
        &self,
        amount: f64,
        from_currency: &str,
        to_currency: &str,
    ) -> Result<f64, ExchangeRateError> {
        // Get the latest rates with from_currency as base
        let rates = self.get_latest_rates(from_currency).await?;

        // Get the conversion rate for to_currency
        let rate = rates
            .get_rate(to_currency)
            .ok_or(ExchangeRateError::UnsupportedCode)?;

        // Calculate the converted amount
        Ok(amount * rate)
    }

    /// Get pair conversion rate (direct conversion between two currencies)
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails, the response cannot be parsed,
    /// or the API returns an error response
    pub async fn get_pair_conversion(
        &self,
        from_currency: &str,
        to_currency: &str,
    ) -> Result<f64, ExchangeRateError> {
        // Define the response structure at the beginning of the function
        #[derive(serde::Deserialize)]
        struct PairConversionResponse {
            conversion_rate: f64,
        }
        
        let url = self.build_url("pair", &[from_currency, to_currency]);

        let mut request_builder = self.http_client.get(&url);

        // Add authorization header if using bearer token auth
        if let AuthMethod::BearerToken = self.auth_method {
            request_builder = request_builder.header(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", self.api_key),
            );
        }

        let response = request_builder
            .send()
            .await
            .map_err(ExchangeRateError::HttpClientError)?;

        // Check for HTTP errors
        if !response.status().is_success() {
            return Err(ExchangeRateError::HttpError(response.status()));
        }

        // Parse the response to get the conversion rate
        let pair_response = response
            .json::<PairConversionResponse>()
            .await
            .map_err(ExchangeRateError::HttpClientError)?;

        Ok(pair_response.conversion_rate)
    }

    /// Get supported currency codes
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails, the response cannot be parsed,
    /// or the API returns an error response
    pub async fn get_supported_codes(&self) -> Result<Vec<(String, String)>, ExchangeRateError> {
        // Define the response structure at the beginning of the function
        #[derive(serde::Deserialize)]
        struct SupportedCodesResponse {
            supported_codes: Vec<Vec<String>>,
        }
        
        let url = self.build_url("codes", &[]);

        let mut request_builder = self.http_client.get(&url);

        // Add authorization header if using bearer token auth
        if let AuthMethod::BearerToken = self.auth_method {
            request_builder = request_builder.header(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", self.api_key),
            );
        }

        let response = request_builder
            .send()
            .await
            .map_err(ExchangeRateError::HttpClientError)?;

        // Check for HTTP errors
        if !response.status().is_success() {
            return Err(ExchangeRateError::HttpError(response.status()));
        }

        // Parse the response to get the supported codes
        let codes_response = response
            .json::<SupportedCodesResponse>()
            .await
            .map_err(ExchangeRateError::HttpClientError)?;

        // Convert the nested Vec<Vec<String>> to Vec<(String, String)>
        let codes = codes_response
            .supported_codes
            .into_iter()
            .filter_map(|code_pair| {
                if code_pair.len() >= 2 {
                    Some((code_pair[0].clone(), code_pair[1].clone()))
                } else {
                    None
                }
            })
            .collect();

        Ok(codes)
    }
}
