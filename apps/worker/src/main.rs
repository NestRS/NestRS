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
