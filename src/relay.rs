use tokio::net::TcpStream;
use tokio::io::copy_bidirectional;
use crate::errors::RelayError;

pub async fn run_relay(mut client: TcpStream, mut upstream: TcpStream) -> Result<(u64, u64), RelayError> {
    // copy_bidirectional handles half-close gracefully and returns (bytes_client_to_upstream, bytes_upstream_to_client)
    let (up, down) = copy_bidirectional(&mut client, &mut upstream).await?;
    Ok((up, down))
}
