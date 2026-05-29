//! `worker` — the **background-processing** example: no HTTP. A `#[cron_job]`
//! enqueues work on a fixed schedule (`Scheduler`) and a `#[processor]` drains
//! the Redis-backed queue (`QueueWorker`). The composition root is [`AppModule`];
//! its feature module is crate-private.

mod app;
mod audio;

pub use app::AppModule;
