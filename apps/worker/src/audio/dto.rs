use serde::{Deserialize, Serialize};

/// The queue name, mirroring NestJS's `@Processor('audio')` /
/// `registerQueue({ name: 'audio' })`. The producer uses this const; the consumer
/// repeats the literal in `#[processor(queue = "audio")]` because attribute
/// arguments can't reference a const. The consumer's test pins the two together.
pub const AUDIO_QUEUE: &str = "audio";

/// The job payload — a file to transcode. The NestJS docs enqueue a `transcode`
/// job with data like `{ file }`; the payload type is that data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscodeJob {
    pub file: String,
}
