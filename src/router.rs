use crate::handler::{config_handler, file_handler, subtitle_task};
use crate::AppState;
use axum::routing::{get, post};
use axum::Router;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

pub fn build_router(state: Arc<AppState>) -> Router {
    let api = Router::new()
        .route("/api/capability/subtitleTask", post(subtitle_task::start_task))
        .route("/api/capability/subtitleTask", get(subtitle_task::get_task))
        .route("/api/file", post(file_handler::upload_file))
        .route("/api/file/*filepath", get(file_handler::download_file))
        .route("/api/config", get(config_handler::get_config))
        .route("/api/config", post(config_handler::update_config));

    Router::new()
        .merge(api)
        .layer(CorsLayer::permissive())
        .with_state(state)
}
