use std::{
    error::Error,
    sync::{atomic::AtomicU64, Arc},
};

use serenity::prelude::TypeMapKey;

/// A connection to the database, representing the stored "state" of the app
pub struct AppState {
    pub start_time: std::time::Instant,
    pub num_connected: Arc<AtomicU64>,
}

impl AppState {
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            start_time: std::time::Instant::now(),
            num_connected: Arc::new(AtomicU64::new(0)),
        })
    }
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState").finish()
    }
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        Self {
            start_time: self.start_time,
            num_connected: self.num_connected.clone(),
        }
    }
}

impl TypeMapKey for AppState {
    type Value = AppState;
}
