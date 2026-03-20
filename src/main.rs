use krillin_rs::config::Config;
use krillin_rs::router::build_router;
use krillin_rs::service::Service;
use krillin_rs::storage::task_store::TaskStore;
use krillin_rs::storage::BinPaths;
use krillin_rs::AppState;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    // Load configuration
    let config = Config::load()?;
    let addr = format!("{}:{}", config.server.host, config.server.port);

    // Detect external tool paths
    let bin_paths = BinPaths::detect();
    tracing::info!("ffmpeg: {}", bin_paths.ffmpeg);
    tracing::info!("yt-dlp: {}", bin_paths.ytdlp);

    // Initialize service from config
    let service = Service::from_config(&config);

    // Build shared state
    let state = Arc::new(AppState {
        config: RwLock::new(config),
        task_store: TaskStore::new(),
        bin_paths: RwLock::new(bin_paths),
        service: RwLock::new(service),
        config_updated: AtomicBool::new(false),
    });

    // Build router and start server
    let app = build_router(state);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Server starting on {addr}");
    axum::serve(listener, app).await?;

    Ok(())
}
