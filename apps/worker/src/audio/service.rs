use std::time::Duration;

use anyhow::Result;
use nestrs_core::injectable;

#[injectable]
#[derive(Default)]
pub struct Transcoder;

impl Transcoder {
    pub async fn transcode(&self, file: &str) -> Result<()> {
        tokio::time::sleep(Duration::from_millis(300)).await;
        tracing::info!(target: "worker::audio", file, "transcoded");
        Ok(())
    }
}
