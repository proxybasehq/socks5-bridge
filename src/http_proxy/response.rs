use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

pub async fn send_error(stream: &mut TcpStream, code: u16, reason: &str) -> Result<(), std::io::Error> {
    let response = format!("HTTP/1.1 {} {}\r\nConnection: close\r\nContent-Length: 0\r\n\r\n", code, reason);
    stream.write_all(response.as_bytes()).await?;
    stream.flush().await?;
    Ok(())
}

pub async fn send_connection_established(stream: &mut TcpStream) -> Result<(), std::io::Error> {
    let response = "HTTP/1.1 200 Connection Established\r\n\r\n";
    stream.write_all(response.as_bytes()).await?;
    stream.flush().await?;
    Ok(())
}
