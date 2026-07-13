use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::sync::Arc;

use crate::config::AppConfig;
use crate::policy::PolicyEngine;
use crate::session::{SessionContext, SessionState, TargetAddr};
use crate::socks5::protocol::*;
use crate::http_proxy::request::{ProxyRequest, ConnectRequest}; // We'll map SOCKS5 connect to a ProxyRequest for code reuse

pub async fn handle_client(
    mut stream: TcpStream,
    mut session: SessionContext,
    config: Arc<AppConfig>,
    policy: Arc<PolicyEngine>,
) {
    session.set_state(SessionState::ParsingClientRequest);

    // 1. Read Greeting
    let mut greeting = [0u8; 2];
    if stream.read_exact(&mut greeting).await.is_err() {
        return;
    }
    
    if greeting[0] != SOCKS_VERSION {
        return;
    }

    let n_methods = greeting[1] as usize;
    let mut methods = vec![0u8; n_methods];
    if stream.read_exact(&mut methods).await.is_err() {
        return;
    }

    // Accept NO_AUTH
    if !methods.contains(&NO_AUTH) {
        let _ = stream.write_all(&[SOCKS_VERSION, 0xFF]).await;
        return;
    }

    if stream.write_all(&[SOCKS_VERSION, NO_AUTH]).await.is_err() {
        return;
    }

    // 2. Read Command
    let mut cmd_head = [0u8; 4];
    if stream.read_exact(&mut cmd_head).await.is_err() {
        return;
    }

    if cmd_head[0] != SOCKS_VERSION || cmd_head[1] != CMD_CONNECT {
        let _ = stream.write_all(&[SOCKS_VERSION, 0x07, 0x00, 0x01, 0, 0, 0, 0, 0, 0]).await; // Command not supported
        return;
    }

    let atyp = cmd_head[3];
    let (host, port) = match atyp {
        ATYP_IPV4 => {
            let mut ip = [0u8; 4];
            if stream.read_exact(&mut ip).await.is_err() { return; }
            let mut p = [0u8; 2];
            if stream.read_exact(&mut p).await.is_err() { return; }
            (std::net::Ipv4Addr::new(ip[0], ip[1], ip[2], ip[3]).to_string(), u16::from_be_bytes(p))
        }
        ATYP_DOMAIN => {
            let mut len_buf = [0u8; 1];
            if stream.read_exact(&mut len_buf).await.is_err() { return; }
            let len = len_buf[0] as usize;
            let mut dom = vec![0u8; len];
            if stream.read_exact(&mut dom).await.is_err() { return; }
            let mut p = [0u8; 2];
            if stream.read_exact(&mut p).await.is_err() { return; }
            
            let host_str = String::from_utf8_lossy(&dom).to_string();
            (host_str, u16::from_be_bytes(p))
        }
        ATYP_IPV6 => {
            let mut ip = [0u8; 16];
            if stream.read_exact(&mut ip).await.is_err() { return; }
            let mut p = [0u8; 2];
            if stream.read_exact(&mut p).await.is_err() { return; }
            
            let mut addr = [0u16; 8];
            for i in 0..8 {
                addr[i] = u16::from_be_bytes([ip[i*2], ip[i*2+1]]);
            }
            (std::net::Ipv6Addr::from(addr).to_string(), u16::from_be_bytes(p))
        }
        _ => {
            let _ = stream.write_all(&[SOCKS_VERSION, 0x08, 0x00, 0x01, 0, 0, 0, 0, 0, 0]).await; // Address type not supported
            return;
        }
    };

    session.set_state(SessionState::ValidatingPolicy);
    if let Err(e) = policy.validate_destination(&host, port) {
        tracing::warn!(event="policy_denied", error=%e, target_host=host, target_port=port);
        let _ = stream.write_all(&[SOCKS_VERSION, 0x02, 0x00, 0x01, 0, 0, 0, 0, 0, 0]).await; // Connection not allowed by ruleset
        session.set_state(SessionState::Failed);
        return;
    }

    session.set_state(SessionState::ConnectingUpstream);
    
    // Create a mock ProxyRequest so we can reuse the socks5 client
    let request = ProxyRequest::Connect(ConnectRequest {
        host: host.clone(),
        port,
        version: "HTTP/1.1".into(),
        headers: vec![],
    });

    match crate::socks5::client::connect_and_auth(&config.upstream, &request).await {
        Ok(upstream_stream) => {
            session.set_state(SessionState::SendingClientConnectAck);
            
            // Reply success to local client
            // Just bind address 0.0.0.0:0 for simplicity
            let reply = [SOCKS_VERSION, REP_SUCCESS, 0x00, ATYP_IPV4, 0, 0, 0, 0, 0, 0];
            if stream.write_all(&reply).await.is_err() {
                 session.set_state(SessionState::Failed);
                 return;
            }

            session.set_state(SessionState::Relaying);
            match crate::relay::run_relay(stream, upstream_stream).await {
                Ok((up, down)) => {
                    session.bytes_up += up;
                    session.bytes_down += down;
                    session.set_state(SessionState::Closing);
                }
                Err(e) => {
                    tracing::error!(event="relay_error", error=%e);
                    session.set_state(SessionState::Failed);
                }
            }
        }
        Err(e) => {
            tracing::error!(event="upstream_auth_failed", error=%e);
            let _ = stream.write_all(&[SOCKS_VERSION, 0x04, 0x00, 0x01, 0, 0, 0, 0, 0, 0]).await; // Host unreachable
            session.set_state(SessionState::Failed);
        }
    }

    let ms = session.started_at.elapsed().as_millis();
    if session.bytes_up > 0 || session.bytes_down > 0 || ms > 1000 {
        tracing::info!(
            event = "session_complete",
            protocol = "socks5",
            connection_id = %session.connection_id,
            duration_ms = ms,
            bytes_up = session.bytes_up,
            bytes_down = session.bytes_down
        );
    }
}
