use crate::dto::ApiResponse;
use crate::AppState;
use axum::extract::State;
use axum::Json;
use std::sync::Arc;

pub async fn get_config(
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let config = state.config.read().await;
    let resp = ApiResponse::success(config.clone());
    Json(serde_json::to_value(resp).unwrap())
}

pub async fn update_config(
    State(state): State<Arc<AppState>>,
    Json(new_config): Json<crate::config::Config>,
) -> Json<serde_json::Value> {
    if let Err(e) = new_config.validate() {
        let resp = ApiResponse::<()>::error(&format!("Validation failed: {e}"));
        return Json(serde_json::to_value(resp).unwrap());
    }

    if let Err(e) = new_config.save() {
        let resp = ApiResponse::<()>::error(&format!("Failed to save config: {e}"));
        return Json(serde_json::to_value(resp).unwrap());
    }

    let mut config = state.config.write().await;
    *config = new_config;
    state.config_updated.store(true, std::sync::atomic::Ordering::SeqCst);

    let resp = ApiResponse::<()>::ok();
    Json(serde_json::to_value(resp).unwrap())
}
