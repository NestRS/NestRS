use nestrs_core::module;

use crate::audio::processor::AudioProcessor;
use crate::audio::producer::AudioProducer;
use crate::audio::service::Transcoder;

#[module(providers = [Transcoder, AudioProcessor, AudioProducer])]
pub struct AudioModule;
