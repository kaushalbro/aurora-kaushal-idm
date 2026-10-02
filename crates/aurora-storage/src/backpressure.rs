use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Notify;

/// Disk backpressure monitor and coordinator.
#[derive(Debug, Clone)]
pub struct BackpressureController {
    current_queued_bytes: Arc<AtomicUsize>,
    max_queue_bytes: usize,
    is_throttled: Arc<AtomicBool>,
    drain_notify: Arc<Notify>,
}

impl BackpressureController {
    pub fn new(max_queue_bytes: usize) -> Self {
        Self {
            current_queued_bytes: Arc::new(AtomicUsize::new(0)),
            max_queue_bytes,
            is_throttled: Arc::new(AtomicBool::new(false)),
            drain_notify: Arc::new(Notify::new()),
        }
    }

    pub fn record_enqueue(&self, bytes: usize) {
        let prev = self.current_queued_bytes.fetch_add(bytes, Ordering::SeqCst);
        let total = prev + bytes;
        if total > self.max_queue_bytes {
            self.is_throttled.store(true, Ordering::SeqCst);
        }
    }

    pub fn record_dequeue(&self, bytes: usize) {
        let prev = self.current_queued_bytes.fetch_sub(bytes, Ordering::SeqCst);
        let remaining = prev.saturating_sub(bytes);
        // Hysteresis: resume when below 50% capacity
        if remaining < (self.max_queue_bytes / 2) && self.is_throttled.load(Ordering::SeqCst) {
            self.is_throttled.store(false, Ordering::SeqCst);
            self.drain_notify.notify_waiters();
        }
    }

    pub fn is_under_pressure(&self) -> bool {
        self.is_throttled.load(Ordering::SeqCst)
    }

    pub fn queued_bytes(&self) -> usize {
        self.current_queued_bytes.load(Ordering::Relaxed)
    }

    /// Asynchronously waits until disk queue drains below threshold if under backpressure.
    pub async fn wait_if_pressured(&self) {
        while self.is_under_pressure() {
            tokio::select! {
                _ = self.drain_notify.notified() => {},
                _ = tokio::time::sleep(Duration::from_millis(50)) => {},
            }
        }
    }
}
