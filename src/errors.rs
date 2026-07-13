use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Configuration Error: {0}")]
    Config(#[from] ConfigError),
    
    #[error("Listener Error: {0}")]
    Listener(#[from] ListenerError),
    
    #[error("HTTP Error: {0}")]
    Http(#[from] HttpError),
    
    #[error("Policy Error: {0}")]
    Policy(#[from] PolicyError),
    
    #[error("Upstream Error: {0}")]
    Upstream(#[from] UpstreamError),
    
    #[error("Relay Error: {0}")]
    Relay(#[from] RelayError),
    
    #[error("Admin Error: {0}")]
    Admin(#[from] AdminError),
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to load config file (I/O error): {0}")]
    Load(#[from] std::io::Error),
    #[error("Failed to parse config file: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("Config validation error: {0}")]
    Validation(String),
    #[error("Config permission error: {0}")]
    Permission(String),
}

#[derive(Error, Debug)]
pub enum ListenerError {
    #[error("Configuration Error: Listener Bind Error: {0}")]
    Bind(std::io::Error),
    #[error("Accept Error: {0}")]
    Accept(std::io::Error),
}

#[derive(Error, Debug)]
pub enum HttpError {
    #[error("Read timeout")]
    ReadTimeout,
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Client closed connection early without sending data")]
    ClientClosedEarly,
    #[error("Client attempted to connect using SOCKS protocol, but this is an HTTP proxy listener")]
    SocksProtocolAttempted,
    #[error("Header too large")]
    HeaderTooLarge,
    #[error("Invalid CONNECT target: {0}")]
    InvalidConnectTarget(String),
    #[error("I/O Error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub enum PolicyError {
    #[error("Denied destinations on loopback")]
    DeniedLocalhost,
    #[error("Denied private network range")]
    DeniedPrivateRange,
    #[error("Unsafe bind address without override")]
    DeniedUnsafeBind,
}

#[derive(Error, Debug)]
pub enum UpstreamError {
    #[error("Upstream connect timeout")]
    ConnectTimeout,
    #[error("Upstream connection refused: {0}")]
    ConnectRefused(std::io::Error),
    #[error("Upstream socket I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("SOCKS method rejected")]
    SocksMethodRejected,
    #[error("SOCKS authentication failed")]
    SocksAuthFailed,
    #[error("SOCKS command failed: {0}")]
    SocksCommandFailed(u8),
    #[error("SOCKS reply invalid")]
    SocksReplyInvalid,
}

#[derive(Error, Debug)]
pub enum RelayError {
    #[error("Client closed connection")]
    ClientClosed,
    #[error("Upstream closed connection")]
    UpstreamClosed,
    #[error("Idle timeout")]
    IdleTimeout,
    #[error("I/O error during relay: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub enum AdminError {
    #[error("Admin listener bind error: {0}")]
    Bind(std::io::Error),
    #[error("Unauthorized access to admin API")]
    Unauthorized,
    #[error("Shutdown failed")]
    ShutdownFailed,
}
