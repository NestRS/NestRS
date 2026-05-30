use std::sync::Arc;
use std::time::Duration;

use parking_lot::Mutex;

/// One Server-Timing entry: a metric name, an optional human-readable
/// description (rendered as the W3C `desc` parameter), and a duration.
#[derive(Clone, Debug)]
pub struct Entry {
    pub name: String,
    pub desc: Option<String>,
    pub dur: Duration,
}

/// Per-request accumulator for `Server-Timing` entries. Handlers grab this
/// from request extensions to record sub-step durations:
///
/// ```ignore
/// async fn handler(req: &Request) -> Response {
///     let timings = req.extensions().get::<Timings>().cloned().unwrap_or_default();
///     let start = std::time::Instant::now();
///     // ... do work ...
///     timings.record("db", start.elapsed());
/// }
/// ```
///
/// The interceptor always appends a final `total;dur=X` entry, so calling
/// `record` is optional.
#[derive(Clone, Default)]
pub struct Timings {
    inner: Arc<Mutex<Vec<Entry>>>,
}

impl Timings {
    pub fn record(&self, name: impl Into<String>, dur: Duration) {
        self.push(name, None, dur);
    }

    /// Use this when two entries share a metric name and need to be told
    /// apart in DevTools — the `desc` renders as a tooltip there.
    pub fn record_with_desc(
        &self,
        name: impl Into<String>,
        desc: impl Into<String>,
        dur: Duration,
    ) {
        self.push(name, Some(desc.into()), dur);
    }

    fn push(&self, name: impl Into<String>, desc: Option<String>, dur: Duration) {
        self.inner.lock().push(Entry {
            name: name.into(),
            desc,
            dur,
        });
    }

    pub(crate) fn drain(&self) -> Vec<Entry> {
        std::mem::take(&mut *self.inner.lock())
    }
}
