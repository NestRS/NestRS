//! [`GraphqlConfig`] — the GraphQL endpoint settings, a namespaced `#[config]`
//! loaded from `NESTRS_GRAPHQL__*` (and the `.env` cascade). Every field has a
//! **production-safe default** (playground off, SDL emit off) — the framework
//! defaults to production everywhere, as a safety. A dev run opts the tooling in
//! through `.env.development` (the development overrides), so `app.rs` carries no
//! config literal.

use std::path::PathBuf;

use nestrs_config::{config, Config, ConfigService, Result};
use validator::Validate;

pub(crate) const DEFAULT_PATH: &str = "/graphql";

#[config(namespace = "graphql")]
#[derive(Clone, Debug, Validate)]
pub struct GraphqlConfig {
    /// HTTP path the schema is served at (`POST` for operations, `GET` for the
    /// playground). Default `/graphql`.
    pub path: String,
    /// Serve the GraphQL playground on `GET <path>`. Default `false`
    /// (production-safe); a dev run enables it via `NESTRS_GRAPHQL__PLAYGROUND=true`.
    pub playground: bool,
    /// Where the committed SDL lives — written on boot when `emit_sdl` is `true`.
    /// Default `schema.graphql` (cwd-relative).
    pub schema_path: PathBuf,
    /// (Re)write `schema_path` from the live schema once at boot. A write failure
    /// is logged, never fatal. Default `false`.
    pub emit_sdl: bool,
}

impl Default for GraphqlConfig {
    fn default() -> Self {
        Self {
            path: DEFAULT_PATH.into(),
            playground: false,
            schema_path: "schema.graphql".into(),
            emit_sdl: false,
        }
    }
}

impl Config for GraphqlConfig {
    /// The explicit `NESTRS_GRAPHQL__*` → field mapping (defaults are production-safe).
    fn from_env(env: &ConfigService) -> Result<Self> {
        let d = Self::default();
        Ok(Self {
            path: env.get("PATH").unwrap_or(d.path), //          NESTRS_GRAPHQL__PATH
            playground: env.flag("PLAYGROUND", d.playground)?, //NESTRS_GRAPHQL__PLAYGROUND (else off)
            schema_path: env //                                  NESTRS_GRAPHQL__SCHEMA_PATH
                .get("SCHEMA_PATH")
                .map(PathBuf::from)
                .unwrap_or(d.schema_path),
            emit_sdl: env.flag("EMIT_SDL", d.emit_sdl)?, //      NESTRS_GRAPHQL__EMIT_SDL (else off)
        })
    }
}
