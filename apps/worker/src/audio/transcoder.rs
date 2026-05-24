//! The service the consumer delegates to. A banal stand-in for real audio work,
//! it is an `#[injectable]` so `AudioConsumer` receives it from the container.

use std::time::Duration;

use anyhow::Result;
use nestrs_core::injectable;

#[injectable]
#[derive(Default)]
pub struct Transcoder;

impl Transcoder {
    pub async fn transcode(&self, file: &str) -> Result<()> {
        // Stand in for the CPU/IO of real transcoding.
        tokio::time::sleep(Duration::from_millis(300)).await;
        tracing::info!(target: "worker::audio", file, "transcoded");
        Ok(())
    }
}
