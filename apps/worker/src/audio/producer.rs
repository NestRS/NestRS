use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use nestrs_queue::QueueConnection;
use nestrs_schedule::{async_trait, cron_job, Scheduled};

use crate::audio::dto::{TranscodeJob, AUDIO_QUEUE};

#[cron_job(every = "5s")]
pub struct AudioProducer {
    #[inject]
    queue: Arc<QueueConnection>,
}

#[async_trait]
impl Scheduled for AudioProducer {
    async fn run(&self) -> Result<()> {
        let id = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
        let file = format!("track-{id}.mp3");
        self.queue
            .of::<TranscodeJob>(AUDIO_QUEUE)
            .push(TranscodeJob { file: file.clone() })
            .await?;
        tracing::info!(target: "worker::audio", %file, "queued transcode job");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use nestrs_core::{Container, DiscoveryService, Module};
    use nestrs_schedule::CronJobMeta;

    use crate::audio::AudioModule;

    #[test]
    fn producer_is_discovered_as_a_cron_job() {
        let container = AudioModule::register(Container::builder()).build();
        let jobs = DiscoveryService::new(&container).meta::<CronJobMeta>();
        assert!(
            jobs.iter().any(|d| d.meta.name == "AudioProducer"),
            "AudioProducer is discovered via #[cron_job]",
        );
    }
}
