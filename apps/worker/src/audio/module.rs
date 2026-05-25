use nestrs_core::module;

use crate::audio::consumer::AudioConsumer;
use crate::audio::producer::AudioProducer;
use crate::audio::transcoder::Transcoder;

#[module(providers = [Transcoder, AudioConsumer, AudioProducer])]
pub struct AudioModule;
