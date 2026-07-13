use crate::config::AppConfig;
use crate::errors::ListenerError;
use crate::policy::PolicyEngine;
use crate::session::SessionContext;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Semaphore;

pub struct LocalListener {
    listener: TcpListener,
    config: Arc<AppConfig>,
    policy: Arc<PolicyEngine>,
    semaphore: Arc<Semaphore>,
}

impl LocalListener {
    pub async fn bind(config: Arc<AppConfig>) -> Result<Self, ListenerError> {
        let addr = format!("{}:{}", config.listener.host, config.listener.port);
        let listener = TcpListener::bind(&addr)
            .await
            .map_err(ListenerError::Bind)?;
        
        let policy = Arc::new(PolicyEngine::new(config.policy.clone()));
        let semaphore = Arc::new(Semaphore::new(config.policy.max_concurrent_connections));

        tracing::info!(event = "listener_bound", addr = %addr);

        Ok(Self {
            listener,
            config,
            policy,
            semaphore,
        })
    }

    pub async fn run_accept_loop(self) -> Result<(), ListenerError> {
        loop {
            // Wait for concurrency slots
            let permit = match self.semaphore.clone().acquire_owned().await {
                Ok(p) => p,
                Err(_) => {
                    tracing::info!("Semaphore closed, shutting down accept loop.");
                    break;
                }
            };

            let (stream, addr) = match self.listener.accept().await {
                Ok((s, a)) => (s, a),
                Err(e) => {
                    tracing::error!(event = "accept_error", error = %e);
                    continue;
                }
            };

            let mut session = SessionContext::new(addr);
            tracing::debug!(
                event = "client_accepted",
                connection_id = %session.connection_id,
                client_ip = %addr
            );

            let config_clone = self.config.clone();
            let policy_clone = self.policy.clone();

            tokio::spawn(async move {
                let _permit = permit; 
                
                let mut buf = [0u8; 1];
                if let Ok(n) = stream.peek(&mut buf).await {
                    if n > 0 && buf[0] == 0x05 {
                        crate::socks5::server::handle_client(stream, session, config_clone, policy_clone).await;
                    } else {
                        crate::http_proxy::handle_session(stream, session, config_clone, policy_clone).await;
                    }
                }
            });
        }
        
        Ok(())
    }
}
