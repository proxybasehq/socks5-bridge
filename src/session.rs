use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

static CONNECTION_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone)]
pub enum SessionState {
    Accepted,
    ParsingClientRequest,
    ValidatingPolicy,
    ConnectingUpstream,
    AuthenticatingUpstream,
    IssuingSocksConnect,
    SendingClientConnectAck,
    Relaying,
    Closing,
    Closed,
    Failed,
}

#[derive(Debug, Clone)]
pub enum TargetAddr {
    Domain { host: String, port: u16 },
    Ip(SocketAddr),
}

pub struct SessionContext {
    pub connection_id: String,
    pub client_addr: SocketAddr,
    pub state: SessionState,
    pub target: Option<TargetAddr>,
    pub started_at: Instant,
    pub bytes_up: u64,
    pub bytes_down: u64,
}

impl SessionContext {
    pub fn new(client_addr: SocketAddr) -> Self {
        let id_num = CONNECTION_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        Self {
            connection_id: format!("c_{:06}", id_num),
            client_addr,
            state: SessionState::Accepted,
            target: None,
            started_at: Instant::now(),
            bytes_up: 0,
            bytes_down: 0,
        }
    }

    pub fn set_state(&mut self, state: SessionState) {
        self.state = state;
    }
}
