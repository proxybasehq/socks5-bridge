pub mod parser;
pub mod request;
pub mod response;

use std::sync::Arc;
use tokio::net::TcpStream;
use crate::config::AppConfig;
use crate::policy::PolicyEngine;
use crate::session::{SessionContext, SessionState};
use crate::http_proxy::request::ProxyRequest;

pub async fn handle_session(
    mut stream: TcpStream,
    mut session: SessionContext,
    config: Arc<AppConfig>,
    policy: Arc<PolicyEngine>,
) {
    session.set_state(SessionState::ParsingClientRequest);
    
    match parser::parse_request(&mut stream).await {
        Ok(request) => {
            session.set_state(SessionState::ValidatingPolicy);
            
            let host = request.host();
            let port = request.port();

            if let Err(e) = policy.validate_destination(host, port) {
                tracing::warn!(event="policy_denied", error=%e, target_host=host, target_port=port);
                let _ = response::send_error(&mut stream, 403, "Forbidden").await;
                session.set_state(SessionState::Failed);
                return;
            }

            session.set_state(SessionState::ConnectingUpstream);
            match crate::socks5::client::connect_and_auth(&config.upstream, &request).await {
                Ok(mut upstream_stream) => {
                    // Tell client we are done
                    session.set_state(SessionState::SendingClientConnectAck);
                    
                    if matches!(request, ProxyRequest::Connect(_)) {
                        if let Err(e) = response::send_connection_established(&mut stream).await {
                             tracing::error!(event="send_client_ack_failed", error=%e);
                             session.set_state(SessionState::Failed);
                             return;
                        }
                    } else if let ProxyRequest::Http(http) = &request {
                        // Forward HTTP request into upstream stream
                        let req_line = format!("{} {} {}\r\n", http.method, http.path_and_query, http.version);
                        let mut headers_str = String::new();
                        for (k, v) in &http.headers {
                            if k.eq_ignore_ascii_case("proxy-connection") { continue; }
                            headers_str.push_str(&format!("{}: {}\r\n", k, v));
                        }
                        headers_str.push_str("Connection: close\r\n\r\n");
                        
                        use tokio::io::AsyncWriteExt;
                        if let Err(e) = upstream_stream.write_all(format!("{}{}", req_line, headers_str).as_bytes()).await {
                            tracing::error!(event="forward_http_failed", error=%e);
                            session.set_state(SessionState::Failed);
                            return;
                        }
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
                    let _ = response::send_error(&mut stream, 502, "Bad Gateway").await;
                    session.set_state(SessionState::Failed);
                }
            }
        }
        Err(crate::errors::HttpError::ClientClosedEarly) => {
            tracing::debug!(event="client_closed_early", connection_id=%session.connection_id);
            session.set_state(SessionState::Closed);
        }
        Err(crate::errors::HttpError::SocksProtocolAttempted) => {
            tracing::error!(
                event="client_used_wrong_protocol",
                connection_id=%session.connection_id,
                message="Chrome attempted to connect natively using SOCKS (e.g. socks5://127.0.0.1:8899). The local listener expects HTTP proxy protocol. Configure Chrome with --proxy-server=\"http://127.0.0.1:8899\" instead."
            );
            session.set_state(SessionState::Failed);
        }
        Err(e) => {
            tracing::error!(event="client_parse_failure", error=%e, connection_id=%session.connection_id);
            let _ = response::send_error(&mut stream, 400, "Bad Request").await;
            session.set_state(SessionState::Failed);
        }
    }
    
    // Final log
    tracing::info!(
        event = "session_complete",
        connection_id = %session.connection_id,
        duration_ms = session.started_at.elapsed().as_millis(),
        bytes_up = session.bytes_up,
        bytes_down = session.bytes_down
    );
}
