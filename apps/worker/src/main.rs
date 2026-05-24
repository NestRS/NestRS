//! A headless worker microservice — the Kubernetes "listen and process" pod. It
//! runs no HTTP: only the `Scheduler` (which drives the `AudioProducer` cron) and
//! the `QueueWorker` (which drives the `AudioConsumer`). The producer fills the
//! `audio` queue every few seconds and the consumer drains it, so the full
//! produce → Redis → consume loop runs inside this one process — yet stays
//! decoupled through Redis, so the producer could instead live in another pod.

mod app;
mod audio;

use anyhow::Result;
use nestrs_core::App;
use nestrs_queue::{QueueConnection, QueueWorker};
use nestrs_schedule::Scheduler;
use nestrs_telemetry::Telemetry;

use crate::app::AppModule;

#[tokio::main]
async fn main() -> Result<()> {
    let _telemetry = Telemetry::init("worker")?;

    // The Redis connection is async, so it is built once at the composition root;
    // the QueueWorker transport and the AudioProducer cron both inject it.
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());

    App::builder()
        .provide_factory::<QueueConnection, _, _>(move |_| async move {
            QueueConnection::connect(&redis_url).await
        })
        .module::<AppModule>()
        .build()
        .await?
        .transport(Scheduler::new())
        .transport(QueueWorker::new())
        .run()
        .await
}
