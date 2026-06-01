//! [`DatabaseConfig`] — the connection settings for [`DatabaseModule`], a
//! namespaced `#[config]` whose `from_env` maps `NESTRS_DATABASE__*` to fields
//! explicitly. The single, typed source of truth: this file shows which variable
//! feeds each field and the default when unset.

use std::time::Duration;

use nestrs_config::{config, Config, ConfigService, Result};
use sea_orm::ConnectOptions;
use validator::Validate;

#[config(namespace = "database")]
#[derive(Clone, Debug, Default, Validate)]
pub struct DatabaseConfig {
    /// The database URL, e.g. `postgres://user:pass@host/db`. Empty aborts the
    /// build with a clear message.
    pub url: String,
    /// Maximum pooled connections (SeaORM default when unset).
    pub max_connections: Option<u32>,
    /// Minimum idle connections.
    pub min_connections: Option<u32>,
    /// Connection-acquire timeout in whole seconds.
    pub connect_timeout_secs: Option<u64>,
    /// Log SQL via SeaORM's `sqlx` logging.
    pub sqlx_logging: bool,
}

impl Config for DatabaseConfig {
    /// The explicit `NESTRS_DATABASE__*` → field mapping.
    fn from_env(env: &ConfigService) -> Result<Self> {
        Ok(Self {
            url: env.get("URL").unwrap_or_default(), //                NESTRS_DATABASE__URL
            max_connections: env.parse("MAX_CONNECTIONS")?, //         NESTRS_DATABASE__MAX_CONNECTIONS
            min_connections: env.parse("MIN_CONNECTIONS")?, //         NESTRS_DATABASE__MIN_CONNECTIONS
            connect_timeout_secs: env.parse("CONNECT_TIMEOUT_SECS")?, //NESTRS_DATABASE__CONNECT_TIMEOUT_SECS
            sqlx_logging: env.flag("SQLX_LOGGING", false)?, //         NESTRS_DATABASE__SQLX_LOGGING (else false)
        })
    }
}

impl DatabaseConfig {
    pub(crate) fn connect_options(&self) -> ConnectOptions {
        let mut opts = ConnectOptions::new(self.url.clone());
        if let Some(n) = self.max_connections {
            opts.max_connections(n);
        }
        if let Some(n) = self.min_connections {
            opts.min_connections(n);
        }
        if let Some(secs) = self.connect_timeout_secs {
            opts.connect_timeout(Duration::from_secs(secs));
        }
        opts.sqlx_logging(self.sqlx_logging);
        opts
    }
}
