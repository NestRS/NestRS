//! Namespaced, injectable configuration — the `registerAs` / `ConfigType` /
//! `ConfigModule.forFeature` trio, collapsed to the leverage Rust's type system
//! gives us: **the type is the token**.
//!
//! A `#[config(namespace = "database")]` struct supplies its namespace (via
//! [`Namespaced`]); the crate writes an `impl Config { fn from_env }` mapping each
//! `NESTRS_DATABASE__*` variable to a field. [`ConfigModule::for_feature::<DatabaseConfig>()`]
//! loads it once at boot and registers `Arc<DatabaseConfig>`, which any provider
//! then injects directly:
//!
//! ```ignore
//! #[module(imports = [ConfigModule::for_feature::<DatabaseConfig>()])]
//! pub struct UsersModule;
//!
//! #[injectable]
//! pub struct UsersService {
//!     #[inject] cfg: ::std::sync::Arc<DatabaseConfig>,   // ConfigType<…> + .KEY
//! }
//! ```

use std::marker::PhantomData;

use nestrs_core::{ContainerBuilder, DynamicModule};
use validator::Validate;

use crate::environment::Environment;
use crate::loader::ConfigService;
use crate::Result;

/// The env-domain segment of a config — the `<DOMAIN>` in `NESTRS_<DOMAIN>__<KEY>`.
/// Supplied by the [`config`](crate::config) macro from `#[config(namespace = "…")]`,
/// so the namespace is declared once, on the struct. A supertrait of [`Config`].
pub trait Namespaced {
    /// The namespace, e.g. `"database"` → the `NESTRS_DATABASE__` prefix.
    const NAMESPACE: &'static str;
}

/// A namespaced configuration type — the typed source of truth for one concern.
///
/// The [`config`](crate::config) macro supplies the namespace (via
/// [`Namespaced`]); the crate writes [`from_env`](Self::from_env) — the
/// **explicit** mapping from `NESTRS_<NAMESPACE>__<KEY>` variables to fields,
/// field-by-field, defaults and all. That mapping is the single place to look:
/// opening a `config.rs` shows exactly which variable feeds which field and what
/// the value is when unset. `ConfigModule` owns the *resolution* (the `.env`
/// cascade, the namespaced reader); the module owns the *mapping* (`from_env`).
///
/// ```ignore
/// #[config(namespace = "database")]
/// #[derive(Clone, Debug, Default, Validate)]
/// pub struct DatabaseConfig { pub url: String, pub max_connections: Option<u32> }
///
/// impl Config for DatabaseConfig {
///     fn from_env(env: &ConfigService) -> Result<Self> {
///         Ok(Self {
///             url: env.get("URL").unwrap_or_default(),       // NESTRS_DATABASE__URL
///             max_connections: env.parse("MAX_CONNECTIONS")?, // … else None
///         })
///     }
/// }
/// ```
pub trait Config: Namespaced + Validate + Clone + Send + Sync + Sized + 'static {
    /// Map this config from the environment, explicitly. Read each field from its
    /// `NESTRS_<NAMESPACE>__<KEY>` variable via [`ConfigService`] (`env.get`,
    /// `env.parse`, `env.flag`, `env.list`), falling back to the field's default
    /// when unset. A variable that is set but unparseable returns `Err` (naming
    /// the variable) and aborts the boot — never a silent fallback.
    fn from_env(env: &ConfigService) -> Result<Self>;

    /// Resolve and validate from the environment: build the namespaced
    /// [`ConfigService`] (which ensures the `.env` cascade is loaded), run
    /// [`from_env`](Self::from_env), then the declarative `#[validate(...)]` rules.
    /// A bad value or a violated rule aborts the boot.
    fn load() -> Result<Self> {
        let env = ConfigService::for_namespace(Self::NAMESPACE);
        let config = Self::from_env(&env)?;
        config.validate()?;
        Ok(config)
    }
}

/// The configuration module — the `ConfigModule` analog and the **sole owner of
/// config loading**. List [`ConfigModule::for_root()`](Self::for_root) **first**
/// in the root module's imports: it reads the active [`Environment`] from
/// `NESTRS_ENV`, layers the `.env` cascade into the process environment (real env
/// vars always win), and registers `Arc<Environment>` as global infrastructure,
/// so every later [`Config`] load sees the merged environment:
///
/// ```ignore
/// #[module(imports = [ConfigModule::for_root(), DatabaseModule::for_root(), ...])]
/// pub struct AppModule;
/// ```
///
/// Its other entry point, [`for_feature`](Self::for_feature), is the **generic**
/// loader a configurable module routes through: `for_feature::<C>()` reads `C`'s
/// namespace, validates, and registers `Arc<C>` for injection. `ConfigModule`
/// stays agnostic of concrete config types — the module supplies `C`.
pub struct ConfigModule;

impl ConfigModule {
    /// Establish the config system: ensure the `.env` cascade is loaded, resolve
    /// the [`Environment`], and register `Arc<Environment>`. List it **first** in
    /// the root module's imports.
    pub fn for_root() -> ConfigRoot {
        ConfigRoot
    }

    /// Register a namespaced [`Config`] so it is injectable as `Arc<C>` anywhere
    /// in the app. List the returned value in `#[module(imports = [...])]`:
    ///
    /// ```ignore
    /// #[module(imports = [ConfigModule::for_feature::<DatabaseConfig>()])]
    /// pub struct UsersModule;
    /// ```
    ///
    /// The config loads in the **factory phase** (so a malformed environment
    /// fails the boot with a clear message), and — like every factory output —
    /// becomes global infrastructure, injectable from any module without a
    /// further import. A test that seeds `C` directly (`provide`/`override_value`)
    /// wins over this factory, so it never reads the real environment.
    pub fn for_feature<C: Config>() -> ConfigFeature<C> {
        ConfigFeature(PhantomData)
    }

    /// Wire a configurable module's `C` into `builder`, honouring an optional pin:
    /// `None` loads `C` from the environment (the [`for_feature`](Self::for_feature)
    /// path); `Some(cfg)` **provides `cfg` directly** so it wins over the
    /// environment (the value an app passes to `Module::for_root(config)`). The
    /// single helper every configurable module's `for_root` routes through.
    pub fn provide_feature<C: Config>(
        pinned: Option<C>,
        builder: ContainerBuilder,
    ) -> ContainerBuilder {
        match pinned {
            None => Self::for_feature::<C>().collect(builder),
            Some(config) => builder.provide(config),
        }
    }
}

/// The configured form of [`ConfigModule`] for one [`Config`] type, produced by
/// [`ConfigModule::for_feature`]. A [`DynamicModule`] whose only job is to queue
/// the config-loading factory in the collect phase.
pub struct ConfigFeature<C>(PhantomData<fn() -> C>);

impl<C: Config> DynamicModule for ConfigFeature<C> {
    // Loading is synchronous, but it is fallible, and `register` cannot return an
    // error — so the load is wrapped in a factory queued here and awaited by the
    // build, where a returned `Err` aborts the boot. (The same path the database
    // pool takes; config is one more piece of shared infrastructure.) The error
    // already names the offending variable (`ConfigError::Parse`) or the broken
    // rule, so it surfaces the misconfiguration directly.
    fn collect(&self, builder: ContainerBuilder) -> ContainerBuilder {
        builder
            .provide_factory::<C, _, _>(|_| async move { C::load().map_err(anyhow::Error::from) })
    }
}

/// The configured form of [`ConfigModule::for_root`]. A [`DynamicModule`] that
/// ensures the `.env` cascade is loaded and registers `Arc<Environment>`. The work
/// runs in the **collect phase** (a sync side effect + a direct `provide`), so it
/// completes before any [`Config`] factory reads the environment.
pub struct ConfigRoot;

impl DynamicModule for ConfigRoot {
    fn collect(&self, builder: ContainerBuilder) -> ContainerBuilder {
        crate::dotenv::ensure_env_loaded();
        builder.provide(Environment::from_env())
    }
}

#[cfg(test)]
// `figment::Jail`'s closure returns figment's large `Result` — a fixed signature
// we cannot change, so the lint is unactionable on these tests.
#[allow(clippy::result_large_err)]
mod tests {
    use super::*;
    use crate::ConfigError;

    // A hand-written `impl Config` rather than the `#[config]` macro: the macro
    // emits `::nestrs_config::Config`, which a crate cannot resolve against
    // itself. The end-to-end macro + DI wiring is covered in `nestrs-testing`.
    #[derive(Clone, Validate, PartialEq, Debug)]
    struct DbCfg {
        url: String,
        #[validate(range(min = 1))]
        max_connections: u32,
    }
    impl Namespaced for DbCfg {
        const NAMESPACE: &'static str = "testdb";
    }
    impl Config for DbCfg {
        fn from_env(env: &ConfigService) -> Result<Self> {
            Ok(Self {
                url: env.get("URL").unwrap_or_default(),
                max_connections: env.parse("MAX_CONNECTIONS")?.unwrap_or(10),
            })
        }
    }

    #[test]
    fn load_maps_each_field_from_its_variable() {
        figment::Jail::expect_with(|jail| {
            jail.set_env("NESTRS_TESTDB__URL", "postgres://localhost/app");
            jail.set_env("NESTRS_TESTDB__MAX_CONNECTIONS", "5");
            let cfg = DbCfg::load().expect("config loads from NESTRS_TESTDB__*");
            assert_eq!(
                cfg,
                DbCfg {
                    url: "postgres://localhost/app".into(),
                    max_connections: 5,
                }
            );
            Ok(())
        });
    }

    #[test]
    fn load_falls_back_to_defaults_when_unset() {
        figment::Jail::expect_with(|jail| {
            jail.set_env("NESTRS_TESTDB__URL", "postgres://localhost/app");
            // MAX_CONNECTIONS unset → the in-mapping default (10).
            let cfg = DbCfg::load().expect("config loads with defaults");
            assert_eq!(cfg.max_connections, 10);
            Ok(())
        });
    }

    #[test]
    fn load_validates_on_the_way_in() {
        figment::Jail::expect_with(|jail| {
            jail.set_env("NESTRS_TESTDB__MAX_CONNECTIONS", "0");
            let err = DbCfg::load().expect_err("max_connections = 0 violates min = 1");
            assert!(matches!(err, ConfigError::Validation(_)));
            Ok(())
        });
    }

    #[test]
    fn load_fails_loudly_on_an_unparseable_value() {
        figment::Jail::expect_with(|jail| {
            jail.set_env("NESTRS_TESTDB__MAX_CONNECTIONS", "lots");
            let err = DbCfg::load().expect_err("non-numeric must abort the boot");
            assert!(
                matches!(err, ConfigError::Parse { ref var, .. } if var == "NESTRS_TESTDB__MAX_CONNECTIONS")
            );
            Ok(())
        });
    }
}
