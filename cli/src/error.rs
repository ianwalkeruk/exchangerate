use std::fmt;

#[derive(Debug)]
pub enum CliError {
    ApiError(String),
    InvalidCurrency(String),
    InvalidFormat(String),
    MissingApiKey,
    NetworkError(String),
    UnexpectedError(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::ApiError(msg) => write!(f, "API Error: {}", msg),
            CliError::InvalidCurrency(code) => write!(f, "Invalid currency code: {}", code),
            CliError::InvalidFormat(format) => write!(f, "Invalid output format: {}", format),
            CliError::MissingApiKey => write!(
                f,
                "API key not provided. Use --api-key option or set EXCHANGE_RATE_API_KEY environment variable"
            ),
            CliError::NetworkError(msg) => write!(f, "Network Error: {}", msg),
            CliError::UnexpectedError(msg) => write!(f, "Unexpected Error: {}", msg),
        }
    }
}

impl std::error::Error for CliError {}

impl From<client::Error> for CliError {
    fn from(err: client::Error) -> Self {
        match err {
            client::Error::ApiError(msg) => CliError::ApiError(msg),
            client::Error::InvalidCurrency(code) => CliError::InvalidCurrency(code),
            client::Error::NetworkError(msg) => CliError::NetworkError(msg),
            _ => CliError::UnexpectedError(err.to_string()),
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
