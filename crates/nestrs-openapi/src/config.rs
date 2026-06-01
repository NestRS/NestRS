//! [`OpenApiConfig`] — the OpenAPI document `info` block, a namespaced `#[config]`
//! whose `from_env` maps `NESTRS_OPENAPI__*` to fields. An app sets its identity
//! (`NESTRS_OPENAPI__TITLE`, `…__VERSION`, `…__DESCRIPTION`) in the `.env` cascade.

use nestrs_config::{config, Config, ConfigService, Result};
use validator::Validate;

#[config(namespace = "openapi")]
#[derive(Clone, Debug, Validate)]
pub struct OpenApiConfig {
    /// `info.title`.
    pub title: String,
    /// `info.version`.
    pub version: String,
    /// `info.description`, omitted when `None`.
    pub description: Option<String>,
}

impl Default for OpenApiConfig {
    fn default() -> Self {
        Self {
            title: "nestrs API".into(),
            version: "0.1.0".into(),
            description: None,
        }
    }
}

impl Config for OpenApiConfig {
    /// The explicit `NESTRS_OPENAPI__*` → field mapping.
    fn from_env(env: &ConfigService) -> Result<Self> {
        let d = Self::default();
        Ok(Self {
            title: env.get("TITLE").unwrap_or(d.title), //       NESTRS_OPENAPI__TITLE
            version: env.get("VERSION").unwrap_or(d.version), // NESTRS_OPENAPI__VERSION
            description: env.get("DESCRIPTION"), //              NESTRS_OPENAPI__DESCRIPTION (else None)
        })
    }
}
