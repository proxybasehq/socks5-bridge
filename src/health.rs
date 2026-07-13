use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct HealthState {
    pub ready: AtomicBool,
    pub last_probe_ok: AtomicBool,
    pub last_probe_time: AtomicU64,
}

impl Default for HealthState {
    fn default() -> Self {
        Self {
            ready: AtomicBool::new(false),
            last_probe_ok: AtomicBool::new(false),
            last_probe_time: AtomicU64::new(0),
        }
    }
}

impl HealthState {
    pub fn mark_ready(&self) {
        self.ready.store(true, Ordering::Release);
    }
    
    pub fn record_probe(&self, success: bool) {
        self.last_probe_ok.store(success, Ordering::Release);
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        self.last_probe_time.store(now, Ordering::Release);
    }
}
