use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents the response from the Exchange Rate API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeRateResponse {
    /// Indicates if the API request was successful
    pub result: String,

    /// URL to the API documentation
    pub documentation: String,

    /// URL to the terms of use
    #[serde(rename = "terms_of_use")]
    pub terms_of_use: String,

    /// Unix timestamp of when rates were last updated
    #[serde(rename = "time_last_update_unix")]
    pub time_last_update_unix: u64,

    /// UTC timestamp of when rates were last updated
    #[serde(rename = "time_last_update_utc")]
    pub time_last_update_utc: String,

    /// Unix timestamp of when rates will next be updated
    #[serde(rename = "time_next_update_unix")]
    pub time_next_update_unix: u64,

    /// UTC timestamp of when rates will next be updated
    #[serde(rename = "time_next_update_utc")]
    pub time_next_update_utc: String,

    /// The base currency code used for the exchange rates
    #[serde(rename = "base_code")]
    pub base_code: String,

    /// Map of currency codes to their exchange rates relative to the base currency
    #[serde(rename = "conversion_rates")]
    pub conversion_rates: HashMap<String, f64>,
}

/// Currency code type alias for better readability
pub type CurrencyCode = String;

/// Extension methods for ExchangeRateResponse
impl ExchangeRateResponse {
    /// Get the exchange rate for a specific currency
    pub fn get_rate(&self, currency_code: &str) -> Option<f64> {
        self.conversion_rates.get(currency_code).copied()
    }

    /// Convert an amount from the base currency to another currency
    pub fn convert_from_base(&self, amount: f64, to_currency: &str) -> Option<f64> {
        self.get_rate(to_currency).map(|rate| amount * rate)
    }

    /// Convert an amount from one currency to another
    pub fn convert(&self, amount: f64, from_currency: &str, to_currency: &str) -> Option<f64> {
        if from_currency == self.base_code {
            return self.convert_from_base(amount, to_currency);
        }

        let from_rate = self.get_rate(from_currency)?;
        let to_rate = self.get_rate(to_currency)?;

        // Convert to base currency first, then to target currency
        Some(amount / from_rate * to_rate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_sample_response() {
        // This test would verify that our model can parse the sample JSON correctly
        // when running with cargo test
        let sample_json = r#"{
            "result":"success",
            "documentation":"https://www.exchangerate-api.com/docs",
            "terms_of_use":"https://www.exchangerate-api.com/terms",
            "time_last_update_unix":1747180802,
            "time_last_update_utc":"Wed, 14 May 2025 00:00:02 +0000",
            "time_next_update_unix":1747267202,
            "time_next_update_utc":"Thu, 15 May 2025 00:00:02 +0000",
            "base_code":"USD",
            "conversion_rates":{
                "USD":1,
                "EUR":0.8961,
                "GBP":0.7538,
                "JPY":147.6780
            }
        }"#;

        // Parse the JSON into our model
        let response: ExchangeRateResponse = serde_json::from_str(sample_json).unwrap();

        // Verify the parsed data
        assert_eq!(response.result, "success");
        assert_eq!(response.base_code, "USD");
        assert_eq!(response.get_rate("EUR"), Some(0.8961));

        // Test conversion methods
        assert_eq!(response.convert_from_base(100.0, "EUR"), Some(89.61));
        assert_eq!(
            response
                .convert(100.0, "EUR", "JPY")
                .map(|v| (v * 100.0).round() / 100.0),
            Some(164.80)
        );
    }
}
