//! Background job registry.
//!
//! Keeps track of running jobs so the UI can cancel them. Job state is also
//! persisted in the `jobs` table for history.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use parking_lot::RwLock;
use uuid::Uuid;

#[derive(Clone)]
pub struct CancelToken {
    inner: Arc<AtomicBool>,
}

impl CancelToken {
    pub fn new() -> Self {
        Self { inner: Arc::new(AtomicBool::new(false)) }
    }
    pub fn cancel(&self) {
        self.inner.store(true, Ordering::Relaxed);
    }
    pub fn is_cancelled(&self) -> bool {
        self.inner.load(Ordering::Relaxed)
    }
}

pub struct JobRegistry {
    running: RwLock<HashMap<String, CancelToken>>,
}

impl JobRegistry {
    pub fn new() -> Self {
        Self { running: RwLock::new(HashMap::new()) }
    }

    pub fn start(&self) -> String {
        let id = Uuid::new_v4().to_string();
        let token = CancelToken::new();
        self.running.write().insert(id.clone(), token);
        id
    }

    pub fn cancel(&self, id: &str) -> bool {
        if let Some(t) = self.running.read().get(id) {
            t.cancel();
            true
        } else {
            false
        }
    }

    pub fn is_cancelled(&self, id: &str) -> bool {
        self.running
            .read()
            .get(id)
            .map(|t| t.is_cancelled())
            .unwrap_or(false)
    }

    pub fn complete(&self, id: &str) {
        self.running.write().remove(id);
    }
}

impl Default for JobRegistry {
    fn default() -> Self {
        Self::new()
    }
}
