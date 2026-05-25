use serde::{Deserialize, Serialize};

pub const AUDIO_QUEUE: &str = "audio";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscodeJob {
    pub file: String,
}
