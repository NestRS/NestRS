use nestrs_core::module;

use crate::audio::AudioModule;

#[module(imports = [AudioModule])]
pub struct AppModule;
