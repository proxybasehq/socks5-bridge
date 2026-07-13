#[derive(Debug)]
pub enum ProxyRequest {
    Http(HttpProxyRequest),
    Connect(ConnectRequest),
}

#[derive(Debug)]
pub struct HttpProxyRequest {
    pub method: String,
    pub host: String,
    pub port: u16,
    pub path_and_query: String,
    pub version: String,
    pub headers: Vec<(String, String)>,
}

#[derive(Debug)]
pub struct ConnectRequest {
    pub host: String,
    pub port: u16,
    pub version: String,
    pub headers: Vec<(String, String)>,
}

impl ProxyRequest {
    pub fn host(&self) -> &str {
        match self {
            Self::Http(r) => &r.host,
            Self::Connect(r) => &r.host,
        }
    }

    pub fn port(&self) -> u16 {
        match self {
            Self::Http(r) => r.port,
            Self::Connect(r) => r.port,
        }
    }
}
