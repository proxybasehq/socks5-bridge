use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::test]
async fn test_fake_socks5_server() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    // Spawn fake SOCKS5 server
    tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        
        // 1. Read greeting
        let mut greeting = [0u8; 3];
        stream.read_exact(&mut greeting).await.unwrap();
        assert_eq!(greeting[0], 0x05); // Version 5
        
        // Reply with User/Pass Auth Selected
        stream.write_all(&[0x05, 0x02]).await.unwrap();

        // 2. Read Auth
        let mut auth_head = [0u8; 2];
        stream.read_exact(&mut auth_head).await.unwrap();
        assert_eq!(auth_head[0], 0x01); // Auth version
        let ulen = auth_head[1] as usize;
        let mut user = vec![0u8; ulen];
        stream.read_exact(&mut user).await.unwrap();
        
        let mut plen_buf = [0u8; 1];
        stream.read_exact(&mut plen_buf).await.unwrap();
        let plen = plen_buf[0] as usize;
        let mut pass = vec![0u8; plen];
        stream.read_exact(&mut pass).await.unwrap();

        assert_eq!(&user, b"testuser");
        assert_eq!(&pass, b"testpass");

        // Reply Auth Success
        stream.write_all(&[0x01, 0x00]).await.unwrap();

        // 3. Read Connect
        let mut cmd_head = [0u8; 4];
        stream.read_exact(&mut cmd_head).await.unwrap();
        assert_eq!(cmd_head[0], 0x05);
        assert_eq!(cmd_head[1], 0x01); // CONNECT
        
        let atyp = cmd_head[3];
        match atyp {
            0x01 => {
                let mut ip = [0u8; 4];
                stream.read_exact(&mut ip).await.unwrap();
            }
            0x03 => {
                let mut dlen = [0u8; 1];
                stream.read_exact(&mut dlen).await.unwrap();
                let mut dom = vec![0u8; dlen[0] as usize];
                stream.read_exact(&mut dom).await.unwrap();
            }
            0x04 => {
                let mut ip = [0u8; 16];
                stream.read_exact(&mut ip).await.unwrap();
            }
            _ => panic!("Unknown atyp"),
        }
        let mut cmd_port = [0u8; 2];
        stream.read_exact(&mut cmd_port).await.unwrap();

        // Reply Connect Success
        stream.write_all(&[0x05, 0x00, 0x00, 0x01, 127, 0, 0, 1, 0, 80]).await.unwrap();

        // 4. Relay dummy data
        let mut buf = [0u8; 1024];
        let n = stream.read(&mut buf).await.unwrap();
        assert!(n > 0);
        
        let response = "HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nOK";
        stream.write_all(response.as_bytes()).await.unwrap();
    });

    // Run client connection against our fake server through the bridge
    // Note: For a true full integration test, we would start the App::run and send an HTTP request to the bridge listener.
    // Given the complexity of this setup in a one-off test, we just test the SOCKS5 client logic here.
    
    use socks5_bridge::config::UpstreamConfig;
    use socks5_bridge::http_proxy::request::{ProxyRequest, HttpProxyRequest};

    let config = UpstreamConfig {
        r#type: "socks5".to_string(),
        host: "127.0.0.1".to_string(),
        port,
        username: Some("testuser".to_string()),
        password: Some("testpass".to_string()),
        connect_timeout_ms: 1000,
        auth_timeout_ms: 1000,
        command_timeout_ms: 1000,
        remote_dns: true,
    };

    let req = ProxyRequest::Http(HttpProxyRequest {
        method: "GET".to_string(),
        host: "example.com".to_string(),
        port: 80,
        path_and_query: "/".to_string(),
        version: "HTTP/1.1".to_string(),
        headers: vec![],
    });

    let mut upstream = socks5_bridge::socks5::client::connect_and_auth(&config, &req).await.expect("Failed to connect to fake SOCKS5");
    
    // Test relay
    let payload = b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";
    upstream.write_all(payload).await.unwrap();

    let mut response = [0u8; 1024];
    let n = upstream.read(&mut response).await.unwrap();
    let resp_str = String::from_utf8_lossy(&response[..n]);
    assert!(resp_str.contains("200 OK"));
}
