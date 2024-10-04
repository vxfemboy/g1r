// mods/proxy.rs
use crate::Config;
use tokio::net::TcpStream;
use tokio_socks::tcp::Socks5Stream;

/// Establish a connection to the proxy
pub async fn proxy_exec(config: &Config) -> Result<TcpStream, Box<dyn std::error::Error + Send>> {
    let proxy_addr = match config.proxy_addr.as_ref() {
        Some(addr) => addr,
        None => "127.0.0.1",
    };
    let proxy_port = config.proxy_port.unwrap_or(9050);
    let proxy = format!("{}:{}", proxy_addr, proxy_port);
    let server = format!("{}:{}", config.server, config.port);
    let proxy_stream = TcpStream::connect(proxy).await.unwrap();
    let username = config.proxy_username.clone().unwrap();
    let password = config.proxy_password.clone().unwrap();
    let tcp_stream = if !&username.is_empty() && !password.is_empty() {
        Socks5Stream::connect_with_password_and_socket(proxy_stream, server, &username, &password)
            .await
            .unwrap()
    } else {
        Socks5Stream::connect_with_socket(proxy_stream, server)
            .await
            .unwrap()
    };
    let tcp_stream = tcp_stream.into_inner();

    Ok(tcp_stream)
}
