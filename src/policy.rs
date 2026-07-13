use crate::config::PolicyConfig;
use crate::errors::PolicyError;
use std::net::SocketAddr;

pub struct PolicyEngine {
    config: PolicyConfig,
}

impl PolicyEngine {
    pub fn new(config: PolicyConfig) -> Self {
        Self { config }
    }

    pub fn validate_client(&self, _client_addr: &SocketAddr) -> Result<(), PolicyError> {
        // v1 doesn't actively block remote clients at connection time via policy,
        // it relies on the bind address. Just a placeholder for actual source filtering if needed.
        Ok(())
    }

    pub fn validate_destination(&self, host: &str, _port: u16) -> Result<(), PolicyError> {
        // A full implementation would resolve `host` if it's an IP string, and block it.
        // For now we check basic strings.
        if !self.config.allow_localhost_destinations {
            if host == "localhost" || host == "127.0.0.1" || host == "::1" {
                return Err(PolicyError::DeniedLocalhost);
            }
        }

        if !self.config.allow_private_destinations {
            // Very naive checks for IPs for demonstration.
            // A perfect check would parse to IpAddr and use `is_private()`.
            if host.starts_with("10.") || host.starts_with("192.168.") {
                return Err(PolicyError::DeniedPrivateRange);
            }
            if host.starts_with("172.") {
                let parts: Vec<&str> = host.split('.').collect();
                if parts.len() == 4 {
                    if let Ok(second) = parts[1].parse::<u8>() {
                        if (16..=31).contains(&second) {
                            return Err(PolicyError::DeniedPrivateRange);
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
