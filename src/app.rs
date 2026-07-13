use crate::config::AppConfig;
use crate::errors::AppError;
use std::sync::Arc;
use crate::health::HealthState;
use crate::metrics::Metrics;
use crate::admin_api::AdminState;
use std::time::Instant;

pub struct App;

impl App {
    pub async fn run(config: AppConfig) -> Result<(), AppError> {
        let config = Arc::new(config);
        let health = Arc::new(HealthState::default());
        let metrics = Arc::new(Metrics::default());
        
        health.mark_ready(); // In a full implementation, we'd wait for a successful probe.

        let admin_state = AdminState {
            config: config.clone(),
            health: health.clone(),
            metrics: metrics.clone(),
            start_time: Instant::now(),
        };

        tokio::spawn(async move {
            if let Err(e) = crate::admin_api::start_admin_server(admin_state).await {
                tracing::error!("Admin server error: {}", e);
            }
        });

        tracing::info!("Starting socks5-bridge HTTP listener...");
        let listener = crate::listener::LocalListener::bind(config).await?;
        listener.run_accept_loop().await?;

        Ok(())
    }
}
