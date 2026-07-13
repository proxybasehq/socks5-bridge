use tokio::io::{AsyncReadExt, BufReader};
use tokio::net::TcpStream;
use crate::http_proxy::request::{ProxyRequest, ConnectRequest, HttpProxyRequest};
use crate::errors::HttpError;

const MAX_HEADER_SIZE: usize = 32 * 1024;
const MAX_REQUEST_LINE: usize = 8 * 1024;

pub async fn parse_request(stream: &mut TcpStream) -> Result<ProxyRequest, HttpError> {
    // Read headers until \r\n\r\n
    let mut buf = Vec::with_capacity(4096);
    let mut reader = BufReader::new(stream);
    
    // Naively read byte by byte until double CRLF to prevent buffering too much if we want to hand off the raw socket later
    let mut matched = 0;
    while matched < 4 && buf.len() < MAX_HEADER_SIZE {
        let byte = match reader.read_u8().await {
            Ok(b) => b,
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                if buf.is_empty() {
                    return Err(HttpError::ClientClosedEarly);
                } else {
                    return Err(HttpError::Io(e));
                }
            }
            Err(e) => return Err(HttpError::Io(e)),
        };
        
        // Detect if the client is instinctively trying to speak SOCKS5 or SOCKS4 directly to us
        if buf.is_empty() && (byte == 0x05 || byte == 0x04) {
            return Err(HttpError::SocksProtocolAttempted);
        }
        
        buf.push(byte);
        if (matched == 0 && byte == b'\r') || 
           (matched == 1 && byte == b'\n') || 
           (matched == 2 && byte == b'\r') || 
           (matched == 3 && byte == b'\n') {
            matched += 1;
        } else {
            matched = if byte == b'\r' { 1 } else { 0 };
        }
    }

    if matched < 4 {
        return Err(HttpError::HeaderTooLarge);
    }

    let header_str = String::from_utf8_lossy(&buf);
    let mut lines = header_str.lines();
    
    let request_line = lines.next().ok_or_else(|| HttpError::ParseError("Empty request".into()))?;
    if request_line.len() > MAX_REQUEST_LINE {
        return Err(HttpError::ParseError("Request line too long".into()));
    }

    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() != 3 {
        return Err(HttpError::ParseError("Invalid request line".into()));
    }

    let method = parts[0];
    let uri = parts[1];
    let version = parts[2];

    let mut headers = Vec::new();
    for line in lines {
        if line.is_empty() { break; }
        if let Some((k, v)) = line.split_once(':') {
            let key = k.trim().to_string();
            // In v1 we filter Proxy-Connection and inject Connection: close later if needed.
            // But let's keep them here.
            headers.push((key, v.trim().to_string()));
        }
    }

    if method == "CONNECT" {
        // uri should be host:port
        let (host, port_str) = uri.rsplit_once(':').ok_or_else(|| HttpError::InvalidConnectTarget(uri.into()))?;
        let port = port_str.parse::<u16>().map_err(|_| HttpError::InvalidConnectTarget(uri.into()))?;
        
        Ok(ProxyRequest::Connect(ConnectRequest {
            host: host.to_string(),
            port,
            version: version.to_string(),
            headers,
        }))
    } else {
        // Parse absolute URI: http://example.com:80/path
        if !uri.starts_with("http://") {
            return Err(HttpError::ParseError("Only http:// absolute URIs are supported for standard proxying".into()));
        }
        let without_scheme = &uri[7..];
        let (authority, path_and_query) = if let Some(slash_idx) = without_scheme.find('/') {
            (&without_scheme[..slash_idx], &without_scheme[slash_idx..])
        } else {
            (without_scheme, "/")
        };

        let (host, port) = if let Some((h, p)) = authority.rsplit_once(':') {
            let port = p.parse::<u16>().map_err(|_| HttpError::ParseError("Invalid port".into()))?;
            (h.to_string(), port)
        } else {
            (authority.to_string(), 80)
        };

        Ok(ProxyRequest::Http(HttpProxyRequest {
            method: method.to_string(),
            host,
            port,
            path_and_query: path_and_query.to_string(),
            version: version.to_string(),
            headers,
        }))
    }
}
