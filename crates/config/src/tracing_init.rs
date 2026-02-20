use tracing_subscriber::{fmt, EnvFilter};

/// Initialize the tracing subscriber with env-based filtering.
///
/// Reads `LOG_LEVEL` (or `RUST_LOG`) to set the filter.
/// Defaults to `info` if neither is set.
pub fn init_tracing(default_level: &str) {
    let filter = EnvFilter::try_from_env("RUST_LOG")
        .or_else(|_| EnvFilter::try_from_env("LOG_LEVEL"))
        .unwrap_or_else(|_| EnvFilter::new(default_level));

    fmt().with_env_filter(filter).with_target(true).init();
}
