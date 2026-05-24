use nestrs_core::module;

use crate::audio::consumer::AudioConsumer;
use crate::audio::producer::AudioProducer;
use crate::audio::transcoder::Transcoder;

// The audio feature: the `Transcoder` service, the `AudioConsumer` that drains
// the queue, and the `AudioProducer` cron that fills it. Producer and consumer
// live in the same app here, but are decoupled through Redis — the producer
// could just as well be the `api` service in another pod.
#[module(providers = [Transcoder, AudioConsumer, AudioProducer])]
pub struct AudioModule;
