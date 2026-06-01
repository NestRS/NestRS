//! `QueueModule` — owns the shared Redis [`QueueConnection`](crate::QueueConnection).
//!
//! Configured at its import site with **`QueueModule::for_root()`** (no bare form):
//! it routes the load through [`ConfigModule::for_feature`] (`NESTRS_QUEUE__*` +
//! the `.env` cascade) and registers a [`QueueConnection`].
//!
//! The connection is async, so it is built in the **collect phase** (a queued
//! factory `await`ed before the module tree is wired) — so the `QueueWorker`
//! transport and every producer inject it regardless of import order.
//!
//! ```ignore
//! #[module(imports = [QueueModule::for_root(), AudioModule])]
//! pub struct AppModule;
//! ```

use nestrs_config::ConfigModule;
use nestrs_core::{ContainerBuilder, DynamicModule};

use crate::config::QueueConfig;
use crate::QueueConnection;

/// The queue module. Wire it with `QueueModule::for_root()` (env-driven).
pub struct QueueModule;

impl QueueModule {
    /// Configure the queue. Pass `None` to load [`QueueConfig`] from
    /// `NESTRS_QUEUE__*` (the `.env` cascade), or a `QueueConfig` to pin it in code
    /// (wins over the environment).
    pub fn for_root(config: impl Into<Option<QueueConfig>>) -> QueueSetup {
        QueueSetup {
            pinned: config.into(),
        }
    }
}

/// The configured form of [`QueueModule`]. Resolves its config through
/// [`ConfigModule::provide_feature`] (env, or the pinned value), then opens the
/// connection.
pub struct QueueSetup {
    pinned: Option<QueueConfig>,
}

impl DynamicModule for QueueSetup {
    fn collect(&self, builder: ContainerBuilder) -> ContainerBuilder {
        let builder = ConfigModule::provide_feature(self.pinned.clone(), builder);
        builder.provide_factory::<QueueConnection, _, _>(|container| async move {
            let config = container
                .get::<QueueConfig>()
                .expect("QueueConfig is resolved by ConfigModule::provide_feature");
            QueueConnection::connect(&config.url).await
        })
    }
}
