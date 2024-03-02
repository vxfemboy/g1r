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
    let config_contents = fs::read_to_string("config.toml").expect("Error reading config.toml");
    let config: Config = toml::from_str(&config_contents).expect("Error parsing config.toml");
    println!("Config loaded!");

    let addr = format!("{}:{}", config.server, config.port);
    println!("Connecting to {}...", addr.green());
    let tcp_stream = TcpStream::connect(&addr).await?;
    println!("Connected to {}!", addr.green());

    if config.use_ssl {
        println!("Establishing TLS connection...");
        let mut tls_builder = NTlsConnector::builder();
        tls_builder.danger_accept_invalid_certs(true);
        let tls_connector = TlsConnector::from(tls_builder.build()?);
        let domain = &config.server;
        let tls_stream = tls_connector.connect(domain, tcp_stream).await?;
        println!("TLS connection established!");

        let (reader, writer) = split(tls_stream);
        let (tx, mut rx) = mpsc::channel(1000);
        let read_task = tokio::spawn(async move {
            let mut reader = reader;
            let mut buf = vec![0; 4096];
            loop {
                let n = match reader.read(&mut buf).await {
                    Ok(0) => return, // connection has been killed x.x
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("Error reading from socket: {:?}", e);
                        return;
                    }
                };

                let msg = String::from_utf8_lossy(&buf[..n]).to_string();
                if tx.send(msg).await.is_err() {
                    eprintln!("Error sending message to the channel");
                    return;
                }
                if msg.contains("AUTHENTICATE") || msg.contains("903") {
                    handle_sasl_messages(&mut writer, &msg, &config.sasl_username.unwrap(), &config.sasl_password.unwrap()).await.unwrap();
                }
            }
            //let msg = String::from_utf8_lossy(&buf[..n]).to_string();
            //if msg.contains("AUTHENTICATE") || msg.contains("903") {
            //    handle_sasl_messages(&mut writer, &msg, &sasl_username.unwrap(), &sasl_password.unwrap()).await;
            //}
        });
        // capabilities
        let mut capabilities = config.capabilities.clone().unwrap_or_else(Vec::new);
        //if config.capabilities.is_some() && config.sasl_password.is_some() && config.sasl_username.is_some() {
        if config.sasl_username.is_some() && config.sasl_password.is_some() && !capabilities.contains(&"sasl".to_string()) {
            capabilities.push("sasl".to_string());
        }

        let write_task = tokio::spawn(async move {
            let mut writer = writer;

            // capabilities
            //let capabilities = config.capabilities.clone().unwrap_or_else(Vec::new);
            if !capabilities.is_empty() {
                let cap_req = format!("CAP REQ :{}\r\n", capabilities.join(" "));
                writer.write_all(cap_req.as_bytes()).await.unwrap();
                // handle that CAP ACK yo  
            } // proceeding with sasl if creds are availble
            if let (Some(sasl_username), Some(sasl_password)) = (&config.sasl_username, &config.sasl_password) {
                start_sasl_auth(&mut writer, "PLAIN", &config.nickname, &capabilities).await.unwrap();
            } else {
                writer.write_all(format!("NICK {}\r\n", config.nickname).as_bytes()).await.unwrap();
                writer.write_all(format!("USER {} 0 * :{}\r\n", config.nickname, config.nickname).as_bytes()).await.unwrap();
            }
            writer.flush().await.unwrap();

            while let Some(msg) = rx.recv().await {
                // handle messages better 
                println!("{} {}", "[%] DEBUG:".bold().green(), msg.purple());

                if msg.starts_with("PING") {
                    writer.write_all(format!("PONG {}\r\n", &msg[5..]).as_bytes()).await.unwrap();
                }
                // super dirty auto-rejoin on kick REWRITE THIS
                if let Some(pos) = msg.find(" KICK ") {
                    let parts: Vec<&str> = msg[pos..].split_whitespace().collect();
                    if parts.len() > 3 && parts[2] == config.nickname {
                        writer.write_all(format!("JOIN {}\r\n", config.channel).as_bytes()).await.unwrap();
                    }
                }
            }
        });

        let _ = tokio::try_join!(read_task, write_task);
    } else {
        println!("Non-SSL connection not implemented.");
    }

    Ok(())
}
