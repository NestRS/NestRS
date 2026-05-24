use nestrs_core::module;

use crate::audio::AudioModule;

// The worker's root module. It only imports feature modules — no controllers,
// no resolvers; this app exists to run scheduled producers and queue consumers.
#[module(imports = [AudioModule])]
pub struct AppModule;
