use axum::{
    routing::get,
    Router, Json, extract::State,
};
use serde::Serialize;
use std::sync::Arc;
use tokio::net::TcpListener;
use std::time::Instant;

use crate::config::AppConfig;
use crate::health::HealthState;
use crate::metrics::{Metrics, StatsSnapshot};

#[derive(Clone)]
pub struct AdminState {
    pub config: Arc<AppConfig>,
    pub health: Arc<HealthState>,
    pub metrics: Arc<Metrics>,
    pub start_time: Instant,
}

#[derive(Serialize)]
pub struct HealthResponse {
    ok: bool,
    service: String,
    uptime_sec: u64,
}

#[derive(Serialize)]
pub struct ReadyResponse {
    ready: bool,
    last_probe_ok: bool,
    last_probe_time: u64,
}

pub async fn start_admin_server(state: AdminState) -> Result<(), std::io::Error> {
    if !state.config.health.enable_admin_api {
        return Ok(());
    }

    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/ready", get(ready_handler))
        .route("/stats", get(stats_handler))
        .with_state(state.clone());

    let addr = format!("{}:{}", state.config.health.admin_host, state.config.health.admin_port);
    let listener = TcpListener::bind(&addr).await?;
    
    tracing::info!(event = "admin_api_started", addr = %addr);
    
    axum::serve(listener, app).await
}

async fn health_handler(State(state): State<AdminState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        ok: true,
        service: "socks5-bridge".to_string(),
        uptime_sec: state.start_time.elapsed().as_secs(),
    })
}

async fn ready_handler(State(state): State<AdminState>) -> Json<ReadyResponse> {
    use std::sync::atomic::Ordering;
    Json(ReadyResponse {
        ready: state.health.ready.load(Ordering::Acquire),
        last_probe_ok: state.health.last_probe_ok.load(Ordering::Acquire),
        last_probe_time: state.health.last_probe_time.load(Ordering::Acquire),
    })
}

async fn stats_handler(State(state): State<AdminState>) -> Json<StatsSnapshot> {
    Json(state.metrics.snapshot())
}
