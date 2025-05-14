use crate::models::ExchangeRateResponse;
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use thiserror::Error;

/// Errors that can occur when working with the cache
#[derive(Debug, Error)]
pub enum CacheError {
    /// The requested item was not found in the cache
    #[error("Item not found in cache")]
    NotFound,

    /// The cached item has expired
    #[error("Cached item has expired")]
    Expired,

    /// Error with the cache backend
    #[error("Cache backend error: {0}")]
    Backend(String),

    /// Error serializing or deserializing cache data
    #[error("Cache serialization error: {0}")]
    Serialization(String),
}

/// A cached response with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedResponse<T = ExchangeRateResponse> {
    /// The original API response
    pub response: T,

    /// When the response was cached
    pub cached_at: DateTime<Utc>,

    /// When the response expires (based on time_next_update_unix from API)
    pub expires_at: DateTime<Utc>,
}

impl<T> CachedResponse<T> {
    /// Create a new cached response
    #[must_use]
    pub fn new(response: T) -> Self {
        let cached_at = Utc::now();
        let expires_at = cached_at + Duration::hours(24);

        Self {
            response,
            cached_at,
            expires_at,
        }
    }
}

impl CachedResponse<ExchangeRateResponse> {
    /// Create a new cached response for ExchangeRateResponse with expiration based on API data
    #[must_use]
    pub fn new_with_api_expiration(response: ExchangeRateResponse) -> Self {
        let cached_at = Utc::now();

        // Use the time_next_update_unix from the response as the expiration time
        // If it's not available, cache for 24 hours by default
        let expires_at = if response.time_next_update_unix > 0 {
            DateTime::from_timestamp(response.time_next_update_unix as i64, 0)
                .unwrap_or_else(|| cached_at + Duration::hours(24))
        } else {
            cached_at + Duration::hours(24)
        };

        Self {
            response,
            cached_at,
            expires_at,
        }
    }
}

impl<T> CachedResponse<T> {
    /// Check if the cached response has expired
    #[must_use]
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}

/// Cache configuration options
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Whether caching is enabled
    pub enabled: bool,

    /// Default TTL for cached responses if not specified by the API
    pub default_ttl: Duration,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_ttl: Duration::hours(24),
        }
    }
}

/// Trait for cache backends
#[async_trait]
pub trait CacheBackend: Send + Sync {
    /// Get a cached response by key for ExchangeRateResponse
    async fn get_exchange_rate(
        &self,
        key: &str,
    ) -> Result<CachedResponse<ExchangeRateResponse>, CacheError>;

    /// Set a cached response for ExchangeRateResponse
    async fn set_exchange_rate(
        &self,
        key: &str,
        response: CachedResponse<ExchangeRateResponse>,
    ) -> Result<(), CacheError>;

    /// Get a cached response as raw JSON string
    async fn get_raw(
        &self,
        key: &str,
    ) -> Result<(String, DateTime<Utc>, DateTime<Utc>), CacheError>;

    /// Set a cached response as raw JSON string
    async fn set_raw(
        &self,
        key: &str,
        json: String,
        cached_at: DateTime<Utc>,
        expires_at: DateTime<Utc>,
    ) -> Result<(), CacheError>;

    /// Invalidate a cached response
    async fn invalidate(&self, key: &str) -> Result<(), CacheError>;

    /// Clear all cached responses
    async fn clear_all(&self) -> Result<(), CacheError>;
}

/// In-memory cache implementation
#[derive(Debug, Clone, Default)]
pub struct InMemoryCache {
    cache: Arc<RwLock<HashMap<String, String>>>, // Store serialized JSON
    expiry: Arc<RwLock<HashMap<String, DateTime<Utc>>>>, // Store expiration times separately
}

impl InMemoryCache {
    /// Create a new in-memory cache
    #[must_use]
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            expiry: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl CacheBackend for InMemoryCache {
    async fn get_exchange_rate(
        &self,
        key: &str,
    ) -> Result<CachedResponse<ExchangeRateResponse>, CacheError> {
        // Check if the key exists and is not expired
        let cache = self
            .cache
            .read()
            .map_err(|e| CacheError::Backend(e.to_string()))?;
        let expiry = self
            .expiry
            .read()
            .map_err(|e| CacheError::Backend(e.to_string()))?;

        let json = cache.get(key).ok_or(CacheError::NotFound)?;
        let expires_at = expiry.get(key).ok_or(CacheError::NotFound)?;

        // Check if expired
        if Utc::now() > *expires_at {
            return Err(CacheError::Expired);
        }

        // Deserialize the cached response
        let cached_response: CachedResponse<ExchangeRateResponse> =
            serde_json::from_str(json).map_err(|e| CacheError::Serialization(e.to_string()))?;

        Ok(cached_response)
    }

    async fn set_exchange_rate(
        &self,
        key: &str,
        response: CachedResponse<ExchangeRateResponse>,
    ) -> Result<(), CacheError> {
        // Serialize the response to JSON
        let json = serde_json::to_string(&response)
            .map_err(|e| CacheError::Serialization(e.to_string()))?;

        // Store the serialized response and expiration time
        let mut cache = self
            .cache
            .write()
            .map_err(|e| CacheError::Backend(e.to_string()))?;
        let mut expiry = self
            .expiry
            .write()
            .map_err(|e| CacheError::Backend(e.to_string()))?;

        cache.insert(key.to_string(), json);
        expiry.insert(key.to_string(), response.expires_at);

        Ok(())
    }

    async fn get_raw(
        &self,
        key: &str,
    ) -> Result<(String, DateTime<Utc>, DateTime<Utc>), CacheError> {
        // Check if the key exists and is not expired
        let cache = self
            .cache
            .read()
            .map_err(|e| CacheError::Backend(e.to_string()))?;
        let expiry = self
            .expiry
            .read()
            .map_err(|e| CacheError::Backend(e.to_string()))?;

        let json = cache.get(key).ok_or(CacheError::NotFound)?;
        let expires_at = expiry.get(key).ok_or(CacheError::NotFound)?;

        // Check if expired
        if Utc::now() > *expires_at {
            return Err(CacheError::Expired);
        }

        // Get the cached_at time from the JSON
        let cached_at = Utc::now(); // Fallback value

        Ok((json.clone(), cached_at, *expires_at))
    }

    async fn set_raw(
        &self,
        key: &str,
        json: String,
        _cached_at: DateTime<Utc>,
        expires_at: DateTime<Utc>,
    ) -> Result<(), CacheError> {
        // Store the serialized response and expiration time
        let mut cache = self
            .cache
            .write()
            .map_err(|e| CacheError::Backend(e.to_string()))?;
        let mut expiry = self
            .expiry
            .write()
            .map_err(|e| CacheError::Backend(e.to_string()))?;

        cache.insert(key.to_string(), json);
        expiry.insert(key.to_string(), expires_at);

        Ok(())
    }

    async fn invalidate(&self, key: &str) -> Result<(), CacheError> {
        let mut cache = self
            .cache
            .write()
            .map_err(|e| CacheError::Backend(e.to_string()))?;
        let mut expiry = self
            .expiry
            .write()
            .map_err(|e| CacheError::Backend(e.to_string()))?;

        cache.remove(key);
        expiry.remove(key);

        Ok(())
    }

    async fn clear_all(&self) -> Result<(), CacheError> {
        let mut cache = self
            .cache
            .write()
            .map_err(|e| CacheError::Backend(e.to_string()))?;
        let mut expiry = self
            .expiry
            .write()
            .map_err(|e| CacheError::Backend(e.to_string()))?;

        cache.clear();
        expiry.clear();

        Ok(())
    }
}

#[cfg(feature = "sqlite-cache")]
pub mod sqlite {
    use super::*;
    use rusqlite::{Connection, Result as SqliteResult, params};
    use std::path::Path;
    use std::sync::Mutex;

    /// SQLite cache implementation
    pub struct SqliteCache {
        conn: Arc<Mutex<Connection>>,
    }

    impl SqliteCache {
        /// Create a new SQLite cache
        pub fn new(db_path: impl AsRef<Path>) -> Result<Self, CacheError> {
            let conn = Connection::open(db_path).map_err(|e| {
                CacheError::Backend(format!("Failed to open SQLite database: {}", e))
            })?;

            // Create the cache table if it doesn't exist
            conn.execute(
                "CREATE TABLE IF NOT EXISTS exchange_rate_cache (
                    key TEXT PRIMARY KEY,
                    response TEXT NOT NULL,
                    cached_at TEXT NOT NULL,
                    expires_at TEXT NOT NULL,
                    response_type TEXT NOT NULL
                )",
                [],
            )
            .map_err(|e| CacheError::Backend(format!("Failed to create cache table: {}", e)))?;

            Ok(Self {
                conn: Arc::new(Mutex::new(conn)),
            })
        }
    }

    #[async_trait]
    impl CacheBackend for SqliteCache {
        async fn get_exchange_rate(
            &self,
            key: &str,
        ) -> Result<CachedResponse<ExchangeRateResponse>, CacheError> {
            let conn = self
                .conn
                .lock()
                .map_err(|e| CacheError::Backend(e.to_string()))?;

            let mut stmt = conn
                .prepare(
                    "SELECT response, cached_at, expires_at FROM exchange_rate_cache WHERE key = ?",
                )
                .map_err(|e| CacheError::Backend(e.to_string()))?;

            let result: SqliteResult<Option<(String, String, String)>> = stmt
                .query_row(params![key], |row| {
                    Ok((row.get(0)?, row.get(1)?, row.get(2)?))
                })
                .map(Some)
                .or_else(|e| {
                    if e == rusqlite::Error::QueryReturnedNoRows {
                        Ok(None)
                    } else {
                        Err(e)
                    }
                });

            match result {
                Ok(Some((response_json, cached_at_str, expires_at_str))) => {
                    // Parse the response JSON
                    let response: ExchangeRateResponse = serde_json::from_str(&response_json)
                        .map_err(|e| CacheError::Serialization(e.to_string()))?;

                    // Parse the timestamps
                    let cached_at = DateTime::parse_from_rfc3339(&cached_at_str)
                        .map_err(|e| CacheError::Serialization(e.to_string()))?
                        .with_timezone(&Utc);

                    let expires_at = DateTime::parse_from_rfc3339(&expires_at_str)
                        .map_err(|e| CacheError::Serialization(e.to_string()))?
                        .with_timezone(&Utc);

                    let cached = CachedResponse {
                        response,
                        cached_at,
                        expires_at,
                    };

                    if Utc::now() > expires_at {
                        return Err(CacheError::Expired);
                    }

                    Ok(cached)
                }
                Ok(None) => Err(CacheError::NotFound),
                Err(e) => Err(CacheError::Backend(e.to_string())),
            }
        }

        async fn set_exchange_rate(
            &self,
            key: &str,
            response: CachedResponse<ExchangeRateResponse>,
        ) -> Result<(), CacheError> {
            let conn = self
                .conn
                .lock()
                .map_err(|e| CacheError::Backend(e.to_string()))?;

            let response_json = serde_json::to_string(&response.response)
                .map_err(|e| CacheError::Serialization(e.to_string()))?;

            let cached_at_str = response.cached_at.to_rfc3339();
            let expires_at_str = response.expires_at.to_rfc3339();
            let response_type = "ExchangeRateResponse";

            conn.execute(
                "INSERT OR REPLACE INTO exchange_rate_cache (key, response, cached_at, expires_at, response_type) VALUES (?, ?, ?, ?, ?)",
                params![key, response_json, cached_at_str, expires_at_str, response_type],
            )
            .map_err(|e| CacheError::Backend(e.to_string()))?;

            Ok(())
        }

        async fn get_raw(
            &self,
            key: &str,
        ) -> Result<(String, DateTime<Utc>, DateTime<Utc>), CacheError> {
            let conn = self
                .conn
                .lock()
                .map_err(|e| CacheError::Backend(e.to_string()))?;

            let mut stmt = conn
                .prepare(
                    "SELECT response, cached_at, expires_at FROM exchange_rate_cache WHERE key = ?",
                )
                .map_err(|e| CacheError::Backend(e.to_string()))?;

            let result: SqliteResult<Option<(String, String, String)>> = stmt
                .query_row(params![key], |row| {
                    Ok((row.get(0)?, row.get(1)?, row.get(2)?))
                })
                .map(Some)
                .or_else(|e| {
                    if e == rusqlite::Error::QueryReturnedNoRows {
                        Ok(None)
                    } else {
                        Err(e)
                    }
                });

            match result {
                Ok(Some((response_json, cached_at_str, expires_at_str))) => {
                    // Parse the timestamps
                    let cached_at = DateTime::parse_from_rfc3339(&cached_at_str)
                        .map_err(|e| CacheError::Serialization(e.to_string()))?
                        .with_timezone(&Utc);

                    let expires_at = DateTime::parse_from_rfc3339(&expires_at_str)
                        .map_err(|e| CacheError::Serialization(e.to_string()))?
                        .with_timezone(&Utc);

                    if Utc::now() > expires_at {
                        return Err(CacheError::Expired);
                    }

                    Ok((response_json, cached_at, expires_at))
                }
                Ok(None) => Err(CacheError::NotFound),
                Err(e) => Err(CacheError::Backend(e.to_string())),
            }
        }

        async fn set_raw(
            &self,
            key: &str,
            json: String,
            cached_at: DateTime<Utc>,
            expires_at: DateTime<Utc>,
        ) -> Result<(), CacheError> {
            let conn = self
                .conn
                .lock()
                .map_err(|e| CacheError::Backend(e.to_string()))?;

            let cached_at_str = cached_at.to_rfc3339();
            let expires_at_str = expires_at.to_rfc3339();
            let response_type = "Raw";

            conn.execute(
                "INSERT OR REPLACE INTO exchange_rate_cache (key, response, cached_at, expires_at, response_type) VALUES (?, ?, ?, ?, ?)",
                params![key, json, cached_at_str, expires_at_str, response_type],
            )
            .map_err(|e| CacheError::Backend(e.to_string()))?;

            Ok(())
        }

        async fn invalidate(&self, key: &str) -> Result<(), CacheError> {
            let conn = self
                .conn
                .lock()
                .map_err(|e| CacheError::Backend(e.to_string()))?;

            conn.execute(
                "DELETE FROM exchange_rate_cache WHERE key = ?",
                params![key],
            )
            .map_err(|e| CacheError::Backend(e.to_string()))?;

            Ok(())
        }

        async fn clear_all(&self) -> Result<(), CacheError> {
            let conn = self
                .conn
                .lock()
                .map_err(|e| CacheError::Backend(e.to_string()))?;

            conn.execute("DELETE FROM exchange_rate_cache", [])
                .map_err(|e| CacheError::Backend(e.to_string()))?;

            Ok(())
        }
    }
}

/// Create a cache key for a request
#[must_use]
pub fn create_cache_key(endpoint: &str, params: &[&str]) -> String {
    format!("{}:{}", endpoint, params.join(":"))
}
