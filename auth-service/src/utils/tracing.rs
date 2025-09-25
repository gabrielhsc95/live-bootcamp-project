use color_eyre::eyre::Result;
use tracing_error::ErrorLayer;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

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

pub fn init_tracing() -> Result<logfire::ShutdownGuard> {
    let logfire = logfire::configure()
        .local()
        .with_service_name("bootcamp")
        .with_environment(crate::utils::constants::ENV_NAME.to_owned())
        .with_default_level_filter(tracing::level_filters::LevelFilter::DEBUG)
        .finish()?;

    let logfire_layer = logfire.tracing_layer();

    let guard = logfire.shutdown_guard();

    let fmt_layer = fmt::layer().compact();
    let filter_layer = EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("info"))?;

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(ErrorLayer::default())
        .with(logfire_layer)
        .init();

    Ok(guard)
}
