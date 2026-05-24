//! The shared Redis connection and the [`Queue`] producer handle.
//!
//! [`QueueConnection`] is built once at boot (an async [`App::builder`] factory)
//! and injected wherever a producer enqueues. A producer asks it for a typed
//! handle by queue name — `conn.of::<WelcomeEmail>("welcome-email")` — and pushes
//! jobs onto it. The name is supplied at the call site and must match the
//! `#[processor(queue = "...")]` that consumes it.

use apalis::prelude::Storage;
use apalis_redis::{Config, ConnectionManager, RedisStorage};

use crate::processor::Job;

/// A cheaply-cloneable handle to the queues' shared Redis connection. Provided
/// once via `App::builder().provide_factory(|_| QueueConnection::connect(url))`,
/// then injected as `#[inject] queue: Arc<QueueConnection>` by any producer.
#[derive(Clone)]
pub struct QueueConnection {
    conn: ConnectionManager,
}

impl QueueConnection {
    /// Connect to Redis. Run this in an `App::builder` factory so the connection
    /// is in the container before the module tree (and the `QueueWorker`
    /// transport) is wired.
    pub async fn connect(redis_url: &str) -> anyhow::Result<Self> {
        let conn = apalis_redis::connect(redis_url).await?;
        Ok(Self { conn })
    }

    /// A typed [`Queue`] handle on the named queue, for enqueueing jobs.
    pub fn of<J: Job>(&self, queue: &str) -> Queue<J> {
        Queue {
            storage: self.storage(queue),
        }
    }

    /// The producer-side storage for a queue's namespace.
    pub(crate) fn storage<J: Job>(&self, queue: &str) -> RedisStorage<J> {
        RedisStorage::new_with_config(self.conn.clone(), Config::default().set_namespace(queue))
    }

    /// The consumer-side storage: same namespace as the producer, with the fetch
    /// buffer set to the processor's concurrency (the in-flight-job ceiling).
    pub(crate) fn consumer_storage<J: Job>(
        &self,
        queue: &str,
        concurrency: usize,
    ) -> RedisStorage<J> {
        RedisStorage::new_with_config(
            self.conn.clone(),
            Config::default()
                .set_namespace(queue)
                .set_buffer_size(concurrency.max(1)),
        )
    }
}

/// A typed producer handle for one queue. Obtain it from
/// [`QueueConnection::of`] and [`push`](Queue::push) jobs onto it.
pub struct Queue<J: Job> {
    storage: RedisStorage<J>,
}

impl<J: Job> Queue<J> {
    /// Enqueue a job. It is persisted in Redis and picked up by whichever
    /// `#[processor]` consumes this queue — in this process or another.
    pub async fn push(&self, job: J) -> anyhow::Result<()> {
        // `push` takes `&mut self`; the storage is a cheap connection handle, so
        // we clone per call rather than force callers to hold it mutably.
        let mut storage = self.storage.clone();
        storage.push(job).await?;
        Ok(())
    }
}
