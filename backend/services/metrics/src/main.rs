use ovia_config::{init_tracing, AppConfig};

#[tokio::main]
async fn main() {
    init_tracing("info");

    let _config = AppConfig::from_env().expect("failed to load config");
    tracing::info!(service = "ovia-metrics", "starting");

    // Placeholder: analytics worker loop will be added in later work orders.
    tracing::info!("metrics service ready — no workers configured yet");
    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for ctrl-c");
    tracing::info!("shutting down");
}
