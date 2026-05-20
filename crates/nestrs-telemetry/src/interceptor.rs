use async_trait::async_trait;
use nestrs_middleware::{Interceptor, Next};
use poem::{Request, Response, Result};

#[cfg(feature = "otlp")]
use {
    opentelemetry::global,
    opentelemetry_http::HeaderExtractor,
    tracing::Instrument,
    tracing_opentelemetry::OpenTelemetrySpanExt,
};

/// Extracts the OTel context from incoming W3C `traceparent` / `tracestate`
/// headers and opens a per-request `tracing` span parented on it. Spans
/// created with `#[instrument]` inside handlers become children automatically,
/// so traces stitch across services without any handler-level wiring.
///
/// Field names follow the stable HTTP OTel semantic conventions
/// (`http.request.method`, `http.route`, `http.response.status_code`).
///
/// No-op when the `otlp` feature is off.
#[derive(Default)]
pub struct OtelHttp;

impl OtelHttp {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Interceptor for OtelHttp {
    #[allow(unused_mut, unused_variables)]
    async fn intercept(&self, mut req: Request, next: Next<'_>) -> Result<Response> {
        #[cfg(feature = "otlp")]
        {
            let span = tracing::info_span!(
                "http.request",
                otel.kind = "server",
                http.request.method = %req.method(),
                http.route = %req.uri().path(),
                http.response.status_code = tracing::field::Empty,
            );

            // The propagator lookup + HeaderExtractor walk costs a RwLock read
            // and an allocation; skip it for the common no-traceparent case.
            if req.headers().contains_key("traceparent") {
                let parent_cx = global::get_text_map_propagator(|prop| {
                    prop.extract(&HeaderExtractor(req.headers()))
                });
                let _ = span.set_parent(parent_cx);
            }

            let res = next.run(req).instrument(span.clone()).await;
            match res {
                Ok(r) => {
                    span.record("http.response.status_code", r.status().as_u16());
                    Ok(r)
                }
                Err(err) => {
                    span.record("http.response.status_code", err.status().as_u16());
                    Err(err)
                }
            }
        }
        #[cfg(not(feature = "otlp"))]
        {
            next.run(req).await
        }
    }
}
