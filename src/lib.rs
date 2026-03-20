pub mod config;
pub mod dto;
pub mod error;
pub mod handler;
pub mod provider;
pub mod router;
pub mod service;
pub mod storage;
pub mod types;
pub mod util;

use crate::storage::task_store::TaskStore;
use crate::storage::BinPaths;
use std::sync::atomic::AtomicBool;
use tokio::sync::RwLock;

pub struct AppState {
    pub config: RwLock<config::Config>,
    pub task_store: TaskStore,
    pub bin_paths: RwLock<BinPaths>,
    pub service: RwLock<service::Service>,
    pub config_updated: AtomicBool,
}
