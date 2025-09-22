use axum::{extract::Request, middleware::Next, response::Response};
use opentelemetry::{KeyValue, metrics::Counter};
use std::sync::LazyLock;

static REQUEST_COUNTER: LazyLock<Counter<u64>> = LazyLock::new(|| {
    logfire::u64_counter("http_requests_total")
        .with_description("Total number of HTTP requests")
        .with_unit("{request}")
        .build()
});

pub async fn metrics_middleware(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();

    let response = next.run(request).await;

    // Increment request counter with labels
    REQUEST_COUNTER.add(
        1,
        &[
            KeyValue::new("method", method.to_string()),
            KeyValue::new("status_code", response.status().as_u16() as i64),
            KeyValue::new("route", uri.path().to_string()),
        ],
    );

    response
}
