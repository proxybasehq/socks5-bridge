use serde::{Deserialize, Serialize};
use std::path::Path;
use std::fs;
use crate::errors::{ConfigError, AppError};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AppConfig {
    #[serde(default)]
    pub listener: ListenerConfig,
    pub upstream: UpstreamConfig,
    #[serde(default)]
    pub policy: PolicyConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub health: HealthConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ListenerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    pub port: u16,
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

impl Default for ListenerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: 8899,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UpstreamConfig {
    pub r#type: String,
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    #[serde(default = "default_connect_timeout")]
    pub connect_timeout_ms: u64,
    #[serde(default = "default_auth_timeout")]
    pub auth_timeout_ms: u64,
    #[serde(default = "default_command_timeout")]
    pub command_timeout_ms: u64,
    #[serde(default = "default_remote_dns")]
    pub remote_dns: bool,
}

fn default_connect_timeout() -> u64 { 8000 }
fn default_auth_timeout() -> u64 { 5000 }
fn default_command_timeout() -> u64 { 8000 }
fn default_remote_dns() -> bool { true }

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PolicyConfig {
    #[serde(default = "default_true")]
    pub allow_loopback_only: bool,
    #[serde(default = "default_false")]
    pub allow_private_destinations: bool,
    #[serde(default = "default_false")]
    pub allow_localhost_destinations: bool,
    #[serde(default = "default_max_conn")]
    pub max_concurrent_connections: usize,
    #[serde(default = "default_idle_timeout")]
    pub idle_timeout_ms: u64,
    #[serde(default = "default_shutdown_timeout")]
    pub graceful_shutdown_timeout_ms: u64,
}

fn default_true() -> bool { true }
fn default_false() -> bool { false }
fn default_max_conn() -> usize { 256 }
fn default_idle_timeout() -> u64 { 60000 }
fn default_shutdown_timeout() -> u64 { 5000 }

impl Default for PolicyConfig {
    fn default() -> Self {
        Self {
            allow_loopback_only: true,
            allow_private_destinations: false,
            allow_localhost_destinations: false,
            max_concurrent_connections: 256,
            idle_timeout_ms: 60000,
            graceful_shutdown_timeout_ms: 5000,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default = "default_log_format")]
    pub format: String,
    pub file: Option<String>,
    #[serde(default = "default_true")]
    pub redact_credentials: bool,
}

fn default_log_level() -> String { "info".to_string() }
fn default_log_format() -> String { "json".to_string() }

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            format: default_log_format(),
            file: None,
            redact_credentials: true,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HealthConfig {
    #[serde(default = "default_true")]
    pub enable_admin_api: bool,
    #[serde(default = "default_host")]
    pub admin_host: String,
    #[serde(default = "default_admin_port")]
    pub admin_port: u16,
    #[serde(default = "default_probe_host")]
    pub probe_host: String,
    #[serde(default = "default_probe_port")]
    pub probe_port: u16,
    #[serde(default = "default_probe_interval")]
    pub probe_interval_ms: u64,
}

fn default_admin_port() -> u16 { 8898 }
fn default_probe_host() -> String { "example.com".to_string() }
fn default_probe_port() -> u16 { 443 }
fn default_probe_interval() -> u64 { 30000 }

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            enable_admin_api: true,
            admin_host: default_host(),
            admin_port: 8898,
            probe_host: default_probe_host(),
            probe_port: 443,
            probe_interval_ms: 30000,
        }
    }
}

impl AppConfig {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, AppError> {
        let contents = fs::read_to_string(&path)
            .map_err(ConfigError::Load)?;
        
        let config: AppConfig = toml::from_str(&contents)
            .map_err(ConfigError::Parse)?;
        
        config.validate()?;
        
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.listener.host != "127.0.0.1" && self.policy.allow_loopback_only {
            return Err(ConfigError::Validation("Listener host is not loopback, but policy.allow_loopback_only is true".to_string()));
        }
        if self.listener.port == self.health.admin_port {
            return Err(ConfigError::Validation("Listener port and admin API port cannot be the same".to_string()));
        }

        if self.upstream.r#type != "socks5" {
            return Err(ConfigError::Validation("Upstream type must be 'socks5' for v1".to_string()));
        }

        if self.upstream.host.is_empty() {
            return Err(ConfigError::Validation("Upstream host must not be empty".to_string()));
        }
        if self.upstream.port == 0 {
            return Err(ConfigError::Validation("Upstream port must not be zero".to_string()));
        }
        if self.upstream.username.is_some() != self.upstream.password.is_some() {
            return Err(ConfigError::Validation("Upstream username and password must both be provided or both be absent".to_string()));
        }
        if self.upstream.username.is_none() {
            return Err(ConfigError::Validation("Upstream credentials are required for v1".to_string()));
        }

        Ok(())
    }
}
