use nestrs_core::module;

use crate::audio::consumer::AudioConsumer;
use crate::audio::producer::AudioProducer;
use crate::audio::service::Transcoder;

#[module(providers = [Transcoder, AudioConsumer, AudioProducer])]
pub struct AudioModule;
