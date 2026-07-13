use std::sync::atomic::{AtomicU64, Ordering};
use serde::Serialize;

#[derive(Default, Debug)]
pub struct Metrics {
    pub active_connections: AtomicU64,
    pub accepted_total: AtomicU64,
    pub successful_tunnels_total: AtomicU64,
    pub http_requests_total: AtomicU64,
    pub auth_failures_total: AtomicU64,
    pub upstream_connect_failures_total: AtomicU64,
    pub target_connect_failures_total: AtomicU64,
    pub idle_timeouts_total: AtomicU64,
}

#[derive(Serialize)]
pub struct StatsSnapshot {
    pub active_connections: u64,
    pub accepted_total: u64,
    pub successful_tunnels_total: u64,
    pub http_requests_total: u64,
    pub auth_failures_total: u64,
    pub upstream_connect_failures_total: u64,
    pub target_connect_failures_total: u64,
    pub idle_timeouts_total: u64,
}

impl Metrics {
    pub fn snapshot(&self) -> StatsSnapshot {
        StatsSnapshot {
            active_connections: self.active_connections.load(Ordering::Relaxed),
            accepted_total: self.accepted_total.load(Ordering::Relaxed),
            successful_tunnels_total: self.successful_tunnels_total.load(Ordering::Relaxed),
            http_requests_total: self.http_requests_total.load(Ordering::Relaxed),
            auth_failures_total: self.auth_failures_total.load(Ordering::Relaxed),
            upstream_connect_failures_total: self.upstream_connect_failures_total.load(Ordering::Relaxed),
            target_connect_failures_total: self.target_connect_failures_total.load(Ordering::Relaxed),
            idle_timeouts_total: self.idle_timeouts_total.load(Ordering::Relaxed),
        }
    }
}
