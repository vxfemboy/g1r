// mods/tls.rs 
use tokio::net::TcpStream;
use tokio_native_tls::{TlsConnector, native_tls::TlsConnector as NTlsConnector};
use crate::Config;

// Establish a TLS connection to the server
pub async fn tls_exec(config: &Config, tcp_stream: TcpStream) -> Result<tokio_native_tls::TlsStream<TcpStream>, Box<dyn std::error::Error + Send>> {
    let tls_builder = NTlsConnector::builder().danger_accept_invalid_certs(true).build().unwrap();
    let tls_connector = TlsConnector::from(tls_builder);
    Ok(tls_connector.connect(&config.server, tcp_stream).await.unwrap())

}
