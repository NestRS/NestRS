use std::sync::{Arc, Mutex};
use std::time::Duration;

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
        if let Ok(mut v) = self.inner.lock() {
            v.push(Entry {
                name: name.into(),
                desc: None,
                dur,
            });
        }
    }

    /// Use this when two entries share a metric name and need to be told
    /// apart in DevTools — the `desc` renders as a tooltip there.
    pub fn record_with_desc(
        &self,
        name: impl Into<String>,
        desc: impl Into<String>,
        dur: Duration,
    ) {
        if let Ok(mut v) = self.inner.lock() {
            v.push(Entry {
                name: name.into(),
                desc: Some(desc.into()),
                dur,
            });
        }
    }

    pub(crate) fn drain(&self) -> Vec<Entry> {
        self.inner
            .lock()
            .map(|mut v| std::mem::take(&mut *v))
            .unwrap_or_default()
    }
}
