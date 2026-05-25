use std::sync::Arc;

use anyhow::Result;
use nestrs_queue::{async_trait, processor, Processor};

use crate::audio::dto::TranscodeJob;
use crate::audio::transcoder::Transcoder;

#[processor(queue = "audio", concurrency = 5, retries = 3)]
pub struct AudioConsumer {
    #[inject]
    transcoder: Arc<Transcoder>,
}

#[async_trait]
impl Processor for AudioConsumer {
    type Job = TranscodeJob;

    async fn process(&self, job: TranscodeJob) -> Result<()> {
        self.transcoder.transcode(&job.file).await
    }
}

#[cfg(test)]
mod tests {
    use nestrs_core::{Container, DiscoveryService, Module};
    use nestrs_queue::ProcessorMeta;

    use crate::audio::dto::AUDIO_QUEUE;
    use crate::audio::AudioModule;

    #[test]
    fn consumer_is_discovered_with_its_queue_config() {
        let container = AudioModule::register(Container::builder()).build();
        let processors = DiscoveryService::new(&container).meta::<ProcessorMeta>();
        let audio = processors
            .iter()
            .find(|d| d.meta.name == "AudioConsumer")
            .expect("AudioConsumer is discovered via #[processor]");
        assert_eq!(audio.meta.queue, AUDIO_QUEUE);
        assert_eq!(audio.meta.concurrency, 5);
        assert_eq!(audio.meta.retries, 3);
    }
}
