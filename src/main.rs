use tokio::io::{split, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_native_tls::native_tls::TlsConnector as NTlsConnector;
use tokio_native_tls::TlsConnector;
use tokio::sync::mpsc;
use serde::Deserialize;
use std::fs;
use colored::*;

#[derive(Deserialize)]
struct Config {
    server: String,
    port: u16,
    use_ssl: bool,
    nickname: String,
    channel: String,
    sasl_username: Option<String>,
    sasl_password: Option<String>,
    capabilities: Option<Vec<String>>
}

mod mods {
    pub mod sasl;
}
use mods::sasl::{start_sasl_auth, handle_sasl_messages};

#[tokio::main(flavor = "multi_thread", worker_threads = 12)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Loading Config...");
    let config = loaded_config().expect("Error parsing config.toml");
    println!("Config loaded!");

    if config.use_ssl {
        let tcp_stream = TcpStream::connect(format!("{}:{}", config.server, config.port)).await?;
        println!("Connected to {}!", format!("{}:{}", config.server, config.port).green());

        println!("Establishing TLS connection...");
        let tls_stream = tls_exec (&config, tcp_stream).await?;
        println!("TLS connection established!");

        handler(tls_stream, &config).await?;
    } else {
        println!("Non-SSL connection not implemented.");
    }

    Ok(())
}

fn loaded_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_contents = fs::read_to_string("config.toml")?;
    //let config_contents = fs::read_to_string("config.toml").expect("Error reading config.toml");
    let config: Config = toml::from_str(&config_contents)?;
    //let config: Config = toml::from_str(&config_contents).expect("Error parsing config.toml");
    Ok(config)
}

//async fn tls_exec(config: &Config, tcp_stream: TcpStream) -> Result<tokio_native_tls::TlsStream<TcpStream>, Box<dyn std::error::Error>> {
//    let mut tls_builder = NTlsConnector::builder();
//    tls_builder.danger_accept_invalid_certs(true);
//    let tls_connector = TlsConnector::from(tls_builder.build()?);
//    let domain = &config.server;
//    let tls_stream = tls_connector.connect(domain, tcp_stream).await?;
//    println!("TLS connection established!");
//    Ok(tls_stream)
//}

async fn tls_exec(config: &Config, tcp_stream: TcpStream) -> Result<tokio_native_tls::TlsStream<TcpStream>, Box<dyn std::error::Error>> {
    let tls_builder = NTlsConnector::builder().danger_accept_invalid_certs(true).build()?;
    let tls_connector = TlsConnector::from(tls_builder);
    Ok(tls_connector.connect(&config.server, tcp_stream).await?)
}

async fn handler(tls_stream: tokio_native_tls::TlsStream<TcpStream>, config: &Config) -> Result<(), Box<dyn std::error::Error>> {
//async fn handler(mut tls_stream: tokio_native_tls::TlsStream<TcpStream>, config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let (mut reader, mut writer) = split(tls_stream);
    let (tx, mut rx) = mpsc::channel(1000);

    let read_task = tokio::spawn(async move {
        let mut buf = vec![0; 4096];
        while let Ok(n) = reader.read(&mut buf).await {
            if n == 0 { break; } // connection killed x.x
            let msg = String::from_utf8_lossy(&buf[..n]).to_string();
            if tx.send(msg).await.is_err() { break; } // channel killed x.x
        }
    });

    //let read_task = tokio::spawn(async move {
    //    let mut buf = vec![0; 4096];
    //    loop {
    //        let n = match reader.read(&mut buf).await {
    //            Ok(0) => return, // connection killed x.x
    //            Ok(n) => n,
    //            Err(e) => {
    //                eprintln!("Error reading from socket: {:?}", e);
    //                return;
    //            },
    //        };
    //
    //        let msg = String::from_utf8_lossy(&buf[..n]).to_string();
    //        if tx.send(msg).await.is_err() {
    //            eprintln!("Error sending message to the channel");
    //            return;
    //        }
    //    }
    //});
    //
    let write_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            // new commands here 
            if msg.starts_with("PING") {
                writer.write_all(format!("PONG {}\r\n", &msg[5..]).as_bytes()).await.unwrap();
            }
        }
    });

    //let write_task = tokio::spawn(async move {
    //    while let Some(msg) = rx.recv().await {
    //        if msg.starts_with("PING") {
    //            writer.write_all(format!("PONG {}\r\n", &msg[5..]).as_bytes()).await.unwrap();
    //        } 
    //        if let Some(username) = &config.sasl_username {
    //            if let Some(password) = &config.sasl_password {
    //                handle_sasl_messages(&mut writer, &msg, username, password).await.unwrap();
    //            }
    //        }
    //    }
    //});

    let _ = tokio::try_join!(read_task, write_task);

    Ok(())
}

