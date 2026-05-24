//! The [`QueueWorker`] transport: runs an apalis worker for every discovered
//! `#[processor]`, all on one [`Monitor`] sharing the app's Redis connection.

use anyhow::{Context, Result};
use apalis::prelude::Monitor;
use async_trait::async_trait;
use nestrs_core::{Container, DiscoveryService, Transport};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

use crate::connection::QueueConnection;
use crate::processor::ProcessorMeta;

/// A [`Transport`] that consumes every `#[processor]` discovered in the module
/// tree. Attach it in `main` alongside the others:
///
/// ```ignore
/// App::builder()
///     .provide_factory(|_| QueueConnection::connect("redis://127.0.0.1/"))
///     .module::<AppModule>()
///     .build().await?
///     .transport(HttpTransport::new().bind("0.0.0.0:3002"))
///     .transport(QueueWorker::new())
///     .run().await
/// ```
///
/// At [`configure`](Transport::configure) it reads every [`ProcessorMeta`] from
/// the fully-assembled container and resolves the shared [`QueueConnection`];
/// [`serve`](Transport::serve) builds an apalis [`Monitor`] with one worker per
/// processor and runs it until shutdown is signalled.
pub struct QueueWorker {
    processors: Vec<Arc<ProcessorMeta>>,
    container: Option<Container>,
}

impl QueueWorker {
    pub fn new() -> Self {
        Self {
            processors: Vec::new(),
            container: None,
        }
    }
}

impl Default for QueueWorker {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Transport for QueueWorker {
    async fn configure(&mut self, container: &Container) -> Result<()> {
        let discovery = DiscoveryService::new(container);
        self.processors = discovery
            .meta::<ProcessorMeta>()
            .into_iter()
            .map(|d| d.meta)
            .collect();

        // Fail fast at boot if there are processors to run but nothing seeded the
        // shared connection — an app with no processors attaches this harmlessly.
        if !self.processors.is_empty() {
            container.get::<QueueConnection>().context(
                "QueueWorker found #[processor]s but no QueueConnection in the container — \
                 seed one with App::builder().provide_factory(|_| QueueConnection::connect(url))",
            )?;
            for p in &self.processors {
                tracing::info!(
                    target: "nestrs::queue",
                    processor = p.name,
                    queue = p.queue,
                    concurrency = p.concurrency,
                    retries = p.retries,
                    "registered queue processor",
                );
            }
        }

        self.container = Some(container.clone());
        Ok(())
    }

    async fn serve(self: Box<Self>, cancel: CancellationToken) -> Result<()> {
        // No processors: idle until shutdown rather than returning, so this
        // transport doesn't race the app down when it is the only one attached.
        if self.processors.is_empty() {
            cancel.cancelled().await;
            return Ok(());
        }

        let container = self
            .container
            .expect("QueueWorker::configure must run before serve");
        // Present because configure checked it whenever processors is non-empty.
        let connection = container
            .get::<QueueConnection>()
            .expect("QueueConnection presence is verified in configure");

        let mut monitor = Monitor::new();
        for meta in &self.processors {
            monitor = (meta.register)(monitor, (*connection).clone(), container.clone(), meta);
        }

        monitor
            .run_with_signal(async move {
                cancel.cancelled().await;
                Ok(())
            })
            .await?;
        Ok(())
    }
}
