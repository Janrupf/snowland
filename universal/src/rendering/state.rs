use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Thread safe container for renderer state synchronization.
#[derive(Debug)]
pub struct SharedRendererState {
    shutdown: AtomicBool,
}

impl SharedRendererState {
    /// Creates a new state instance ready to be shared using the Arc.
    pub fn new() -> Arc<Self> {
        assert_send_sync::<Self>();

        Arc::new(Self {
            shutdown: AtomicBool::new(false),
        })
    }

    /// Atomically tests whether the renderer should shut down.
    pub fn should_shutdown(&self) -> bool {
        self.shutdown.load(Ordering::Acquire)
    }

    /// Atomically sets the shutdown flag.
    pub fn initiate_shutdown(&self) {
        self.shutdown.store(true, Ordering::Release)
    }
}

fn assert_send_sync<T: Send + Sync>() {}
