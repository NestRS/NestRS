//! [`AuthnModule`] / [`OAuth2Module`] — make a configured [`JwtService`] /
//! [`OAuth2Client`] injectable everywhere. The analog of NestJS's
//! `JwtModule.register({ ... })`.
//!
//! Each is configured at its import site with **`for_root()`** (no bare form): it
//! routes the load of [`JwtConfig`] / [`OAuth2Config`] through
//! [`ConfigModule::for_feature`] (`NESTRS_AUTHN__*` + the `.env` cascade) and
//! provides its value as global infrastructure (injectable regardless of import
//! order).

use nestrs_config::ConfigModule;
use nestrs_core::{ContainerBuilder, DynamicModule};

use crate::jwt::{JwtConfig, JwtService};
use crate::oauth::{OAuth2Client, OAuth2Config};

/// Provides the app's [`JwtService`], env-driven via `AuthnModule::for_root()`
/// (loads [`JwtConfig`] from `NESTRS_AUTHN__*`, like `DatabaseModule`). An app signs
/// or only verifies depending on the keys its environment carries and the methods it
/// calls — there is no module-level "issuer" vs "resource server" mode.
pub struct AuthnModule;

impl AuthnModule {
    /// Configure JWT. Pass `None` to load [`JwtConfig`] from `NESTRS_AUTHN__*`
    /// (the `.env` cascade), or a `JwtConfig` to pin the keys in code (wins over
    /// the environment). The signing mode is inferred from the keys present.
    pub fn for_root(config: impl Into<Option<JwtConfig>>) -> AuthnSetup {
        AuthnSetup {
            pinned: config.into(),
        }
    }
}

/// The configured form of [`AuthnModule`]. Provides the [`JwtService`] through the
/// factory phase (global infrastructure, like the database/queue connections).
pub struct AuthnSetup {
    pinned: Option<JwtConfig>,
}

impl DynamicModule for AuthnSetup {
    fn collect(&self, builder: ContainerBuilder) -> ContainerBuilder {
        let builder = ConfigModule::provide_feature(self.pinned.clone(), builder);
        builder.provide_factory::<JwtService, _, _>(|container| async move {
            let config = container
                .get::<JwtConfig>()
                .expect("JwtConfig is resolved by ConfigModule::provide_feature");
            let options = (*config)
                .clone()
                .into_options()
                .map_err(anyhow::Error::new)?;
            JwtService::new(options).map_err(anyhow::Error::new)
        })
    }
}

/// Provides a configured [`OAuth2Client`] for a single provider, injectable as
/// `Arc<OAuth2Client>` by an OAuth [`Strategy`](crate::Strategy).
///
/// The flat container keys by type, so one app currently wires one
/// [`OAuth2Client`]; multiple providers would need per-provider newtypes (a
/// future addition).
pub struct OAuth2Module;

impl OAuth2Module {
    /// Configure the OAuth2 provider. Pass `None` to load [`OAuth2Config`] from
    /// `NESTRS_AUTHN__*` (the `.env` cascade), or an `OAuth2Config` to pin it in
    /// code (wins over the environment).
    pub fn for_root(config: impl Into<Option<OAuth2Config>>) -> OAuth2Setup {
        OAuth2Setup {
            pinned: config.into(),
        }
    }
}

/// The configured form of [`OAuth2Module`].
pub struct OAuth2Setup {
    pinned: Option<OAuth2Config>,
}

impl DynamicModule for OAuth2Setup {
    fn collect(&self, builder: ContainerBuilder) -> ContainerBuilder {
        let builder = ConfigModule::provide_feature(self.pinned.clone(), builder);
        builder.provide_factory::<OAuth2Client, _, _>(|container| async move {
            let config = container
                .get::<OAuth2Config>()
                .expect("OAuth2Config is resolved by ConfigModule::provide_feature");
            OAuth2Client::new((*config).clone()).map_err(anyhow::Error::new)
        })
    }
}
