//! The framework-wide environment-variable scheme and [`ConfigService`] — the
//! typed reader a config's `from_env` mapping uses.

use std::env;
use std::str::FromStr;

use crate::dotenv;
use crate::error::ConfigError;

/// Framework-wide environment-variable scheme.
///
/// **Rule:** `NESTRS_<DOMAIN>__<KEY>`. One prefix (`NESTRS_`), one domain
/// segment, then the leaf key. Domain boundaries use **double underscore**;
/// the leaf key itself stays snake_case. Nothing outside this prefix is read by
/// the framework — no `OTEL_*`/`RUST_LOG` aliasing.
///
/// **The domain is the owning crate's name** with the `nestrs-` prefix stripped
/// (`nestrs-database` → `database`, `nestrs-telemetry` → `telemetry`). A domain
/// and its crate always share a name; if they diverge, one of them is misnamed.
///
/// Domains in use today (extend the table as crates land):
///
/// | Domain      | Owner (crate)        | Example variable                   |
/// |-------------|----------------------|------------------------------------|
/// | `database`  | `nestrs-database`    | `NESTRS_DATABASE__URL`             |
/// | `queue`     | `nestrs-queue`       | `NESTRS_QUEUE__URL`                |
/// | `authn`     | `nestrs-authn`       | `NESTRS_AUTHN__PUBLIC_KEY`         |
/// | `graphql`   | `nestrs-graphql`     | `NESTRS_GRAPHQL__PLAYGROUND`       |
/// | `openapi`   | `nestrs-openapi`     | `NESTRS_OPENAPI__TITLE`            |
/// | `telemetry` | `nestrs-telemetry`   | `NESTRS_TELEMETRY__LOG_LEVEL`      |
/// | `http`      | `nestrs-http`        | `NESTRS_HTTP__TLS_KEY_FILE`        |
///
/// Each crate that owns a domain documents its full key list on its config type's
/// `from_env`. A crate maps **its own** domain; it must not read another domain's
/// vars *implicitly*. The one sanctioned exception is an **explicit fallback** in a
/// `from_env`: a config may borrow a sibling domain's variable as a default,
/// **its own variable keeping priority**, when the two genuinely share a value.
/// Build a reader for the other namespace and chain it — the borrow stays visible
/// in the borrowing config, so the precedence reads `own > borrowed > code default`:
///
/// ```ignore
/// // A future `events` module backed by the same Redis as `queue`: it reads its
/// // own URL, else falls back to queue's, else the code default. A multi-Redis
/// // setup still overrides via NESTRS_EVENTS__URL.
/// fn from_env(env: &ConfigService) -> Result<Self> {
///     let queue = ConfigService::for_namespace("queue");
///     Ok(Self {
///         url: env.get("URL")                  // NESTRS_EVENTS__URL  (own, wins)
///             .or_else(|| queue.get("URL"))    // NESTRS_QUEUE__URL   (borrowed default)
///             .unwrap_or_else(default_url),    // code default
///     })
/// }
/// ```
///
/// This is safe and conflict-free: the `.env` cascade is loaded once, in full,
/// before any `from_env` runs, so every variable that will ever exist is already
/// present — borrowing a sibling's variable is just one more read of the same flat
/// environment, with no ordering between modules. (The fallback reads the env
/// **variable**, not the other module's *resolved* config, so it does not see a
/// value the sibling pinned in code via `for_root(Config { .. })` — fine for
/// env-driven values like a URL.)
const PREFIX: &str = "NESTRS_";

/// Read a single env var, treating empty strings as unset. Use this for the few
/// `NESTRS_*` keys read outside the [`Config`](crate::Config) system (e.g. the
/// pre-build telemetry init, the HTTP TLS builder) — the empty-as-unset rule
/// prevents `FOO=` in a `.env` file from blanking out an in-code default.
pub fn env_var(name: &str) -> Option<String> {
    match env::var(name) {
        Ok(v) if !v.is_empty() => Some(v),
        _ => None,
    }
}

/// The typed reader handed to a config's [`from_env`](crate::Config::from_env).
///
/// Bound to one namespace, it resolves `NESTRS_<NAMESPACE>__<KEY>` against the
/// process environment (the `.env` cascade is already merged in — building a
/// `ConfigService` ensures it). The config's `from_env` writes the **explicit**
/// mapping field-by-field, so opening a `config.rs` shows the full correspondence:
/// which variable feeds which field, and the default when it is unset.
///
/// ```ignore
/// fn from_env(env: &ConfigService) -> Result<Self> {
///     Ok(Self {
///         url: env.get("URL").unwrap_or_default(),              // NESTRS_DATABASE__URL
///         max_connections: env.parse("MAX_CONNECTIONS")?.unwrap_or(10), // … else 10
///         sqlx_logging: env.flag("SQLX_LOGGING")?,              // … else false
///     })
/// }
/// ```
pub struct ConfigService {
    namespace: String,
}

impl ConfigService {
    /// Build a reader for `namespace`, ensuring the `.env` cascade is loaded first
    /// (idempotent). The namespace is the config's `NAMESPACE` — callers normally
    /// get a `ConfigService` from the framework, not by name.
    pub fn for_namespace(namespace: &str) -> Self {
        dotenv::ensure_env_loaded();
        Self {
            namespace: namespace.to_ascii_uppercase(),
        }
    }

    /// The full variable name for `key` under this namespace, e.g. `URL` →
    /// `NESTRS_DATABASE__URL`. Use it when building an error message.
    pub fn var(&self, key: &str) -> String {
        format!("{PREFIX}{}__{}", self.namespace, key.to_ascii_uppercase())
    }

    /// The raw string value of `NESTRS_<NS>__<KEY>`, or `None` when unset/empty.
    pub fn get(&self, key: &str) -> Option<String> {
        env_var(&self.var(key))
    }

    /// Parse `NESTRS_<NS>__<KEY>` into `T`. `Ok(None)` when unset; `Err` (naming
    /// the variable) when set but unparseable — a boot-fatal misconfiguration, not
    /// a silent fallback to the default.
    pub fn parse<T>(&self, key: &str) -> Result<Option<T>, ConfigError>
    where
        T: FromStr,
        T::Err: std::fmt::Display,
    {
        match self.get(key) {
            None => Ok(None),
            Some(raw) => raw
                .parse::<T>()
                .map(Some)
                .map_err(|e| ConfigError::parse(self.var(key), e.to_string())),
        }
    }

    /// A boolean flag: `true` for `1`/`true`/`yes`/`on` (case-insensitive), `false`
    /// for `0`/`false`/`no`/`off`, the `default` when unset, `Err` otherwise.
    pub fn flag(&self, key: &str, default: bool) -> Result<bool, ConfigError> {
        match self.get(key) {
            None => Ok(default),
            Some(raw) => match raw.trim().to_ascii_lowercase().as_str() {
                "1" | "true" | "yes" | "on" => Ok(true),
                "0" | "false" | "no" | "off" => Ok(false),
                other => Err(ConfigError::parse(
                    self.var(key),
                    format!("expected a boolean, got `{other}`"),
                )),
            },
        }
    }

    /// A comma-separated list (`a,b,c` → `["a","b","c"]`), trimmed, empties
    /// dropped. Empty when unset. The plain shape for `Vec<String>` config (e.g.
    /// OAuth scopes) without quoting ceremony.
    pub fn list(&self, key: &str) -> Vec<String> {
        self.get(key)
            .map(|raw| {
                raw.split(',')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(ToOwned::to_owned)
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[cfg(test)]
// `figment::Jail`'s closure returns figment's large `Result` — a fixed signature
// we cannot change, so the lint is unactionable on these tests.
#[allow(clippy::result_large_err)]
mod tests {
    use super::*;

    #[test]
    fn var_builds_the_namespaced_name() {
        let env = ConfigService::for_namespace("database");
        assert_eq!(env.var("URL"), "NESTRS_DATABASE__URL");
        assert_eq!(
            env.var("max_connections"),
            "NESTRS_DATABASE__MAX_CONNECTIONS"
        );
    }

    #[test]
    fn parse_reports_the_variable_on_failure() {
        figment::Jail::expect_with(|jail| {
            jail.set_env("NESTRS_TESTDB__MAX", "not-a-number");
            let env = ConfigService::for_namespace("testdb");
            let err = env.parse::<u32>("MAX").expect_err("non-numeric must fail");
            assert!(
                matches!(err, ConfigError::Parse { ref var, .. } if var == "NESTRS_TESTDB__MAX")
            );
            Ok(())
        });
    }

    #[test]
    fn parse_is_none_when_unset() {
        figment::Jail::expect_with(|_| {
            let env = ConfigService::for_namespace("testdb");
            assert!(env
                .parse::<u32>("UNSET_KEY")
                .expect("unset is Ok(None)")
                .is_none());
            Ok(())
        });
    }

    #[test]
    fn flag_reads_common_spellings() {
        figment::Jail::expect_with(|jail| {
            jail.set_env("NESTRS_TESTF__ON", "yes");
            jail.set_env("NESTRS_TESTF__OFF", "false");
            let env = ConfigService::for_namespace("testf");
            assert!(env.flag("ON", false).unwrap());
            assert!(!env.flag("OFF", true).unwrap());
            assert!(env.flag("MISSING", true).unwrap()); // default
            Ok(())
        });
    }

    #[test]
    fn list_splits_on_commas() {
        figment::Jail::expect_with(|jail| {
            jail.set_env("NESTRS_TESTL__SCOPES", "read:user, write , ,admin");
            let env = ConfigService::for_namespace("testl");
            assert_eq!(env.list("SCOPES"), vec!["read:user", "write", "admin"]);
            Ok(())
        });
    }
}
