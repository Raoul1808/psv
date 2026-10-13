use std::sync::{Arc, atomic::AtomicBool};

#[derive(Debug, Clone)]
pub struct CancellationToken {
    token: Arc<AtomicBool>,
}

impl CancellationToken {
    pub fn new() -> Self {
        Self {
            token: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn cancel(&self) {
        self.token.store(true, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn is_cancelled(&self) -> bool {
        self.token.load(std::sync::atomic::Ordering::Relaxed)
    }
}
