//! [`QueueConfig`] — the Redis connection settings for [`QueueModule`], a
//! namespaced `#[config]` whose `from_env` maps `NESTRS_QUEUE__*` to fields.

use nestrs_config::{config, Config, ConfigService, Result};
use validator::Validate;

const DEFAULT_URL: &str = "redis://127.0.0.1/";

#[config(namespace = "queue")]
#[derive(Clone, Debug, Validate)]
pub struct QueueConfig {
    /// The Redis URL backing the queues. Defaults to a local Redis when unset.
    pub url: String,
}

impl Default for QueueConfig {
    fn default() -> Self {
        Self {
            url: DEFAULT_URL.to_string(),
        }
    }
}

impl Config for QueueConfig {
    /// The explicit `NESTRS_QUEUE__*` → field mapping.
    fn from_env(env: &ConfigService) -> Result<Self> {
        Ok(Self {
            // NESTRS_QUEUE__URL, else the local-Redis default.
            url: env.get("URL").unwrap_or_else(|| DEFAULT_URL.to_string()),
        })
    }
}
