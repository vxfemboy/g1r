use tokio::io::{split, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_native_tls::native_tls::TlsConnector as NTlsConnector;
use tokio_native_tls::TlsConnector;
use tokio::sync::mpsc;
use serde::Deserialize;
use tokio_rustls::rustls::Writer;
use std::fs;
use std::future::IntoFuture;
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
    capabilities: Option<Vec<String>>,
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
        let mut tls_stream = tls_exec (&config, tcp_stream).await?;
        println!("TLS connection established!");
        tls_stream.flush().await?;

        handler(tls_stream, config).await?;
    } else {
        println!("Non-SSL connection not implemented.");
    }

    Ok(())
}
/// Load the config file
fn loaded_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_contents = fs::read_to_string("config.toml")?;
    let config: Config = toml::from_str(&config_contents)?;
    Ok(config)
}

/// Establish a TLS connection to the server
async fn tls_exec(config: &Config, tcp_stream: TcpStream) -> Result<tokio_native_tls::TlsStream<TcpStream>, Box<dyn std::error::Error>> {
    let tls_builder = NTlsConnector::builder().danger_accept_invalid_certs(true).build()?;
    let tls_connector = TlsConnector::from(tls_builder);
    Ok(tls_connector.connect(&config.server, tcp_stream).await?)
}


/// Handle the connection to the server
async fn handler(tls_stream: tokio_native_tls::TlsStream<TcpStream>, config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let (reader, writer) = split(tls_stream);
    let (tx, rx) = mpsc::channel(1000);


    
    let read_task = tokio::spawn(async move {
        readmsg(reader, tx).await;
    });


    let write_task = tokio::spawn(async move {
        writemsg(writer, rx, &config).await; 
    });

    let _ = tokio::try_join!(read_task, write_task);
    
    Ok(())
}

/// Read messages from the server
async fn readmsg(mut reader: tokio::io::ReadHalf<tokio_native_tls::TlsStream<TcpStream>>, tx: tokio::sync::mpsc::Sender<String>) {
    let mut buf = vec![0; 4096];
    while let Ok (n) = reader.read(&mut buf).await {
        if n == 0 { break; }
        let msg = String::from_utf8_lossy(&buf[..n]).to_string();
        // must pretty this up later
        println!{"{}{}{} {}{} {}", "[".green().bold(), ">".yellow().bold(), "]".green().bold(), "DEBUG:".bold().yellow(), ":".bold().green(), msg.purple()};
   
        tx.send(msg).await.unwrap();
    }
}

/// Write messages to the server
async fn writemsg(mut writer: tokio::io::WriteHalf<tokio_native_tls::TlsStream<TcpStream>>, mut rx: tokio::sync::mpsc::Receiver<String>, config: &Config) {
    // sasl auth 
    let capabilities = config.capabilities.clone();
    let username = config.sasl_username.clone().unwrap();
    let password = config.sasl_password.clone().unwrap();
    let nickname = config.nickname.clone();


    if !password.is_empty() {
        println!("Starting SASL auth...");
        start_sasl_auth(&mut writer, "PLAIN", &nickname, capabilities).await.unwrap();
        writer.flush().await.unwrap();
    } else {
        nickme(&mut writer, &nickname).await.unwrap();
    }

    writer.flush().await.unwrap();
    // THIS NEEDS TO BE REBUILT TO BE MORE MODULAR AND SECURE 
    while let Some(msg) = rx.recv().await {

        if msg.starts_with("PING") {
            let response = msg.replace("PING", "PONG");
            println!("{} {} {}","[%] PONG:".bold().green(), nickname.blue(), response.purple());
            writer.write_all(response.as_bytes()).await.unwrap();
            writer.flush().await.unwrap();
            //continue;
        }
        // handle sasl auth
        if !password.is_empty(){
            println!("Handling SASL messages...");
            handle_sasl_messages(&mut writer, &msg, &username, &password, &nickname).await.unwrap();
            //continue;
            writer.flush().await.unwrap();
        } 

        // new commands here
        if msg.contains("001") {
            println!("Setting mode");
            writer.write_all(format!("MODE {} +B\r\n", nickname).as_bytes()).await.unwrap();
            writer.flush().await.unwrap();
        }


        if msg.contains("433") {
            println!("Nickname already in use, appending _ to nickname");
            let new_nick = format!("{}_", nickname);
            nickme(&mut writer, &new_nick).await.unwrap();
            writer.flush().await.unwrap();
        }
        if msg.contains("376") {
            println!("Joining channel");
            writer.write_all(format!("JOIN {}\r\n", config.channel).as_bytes()).await.unwrap();
            writer.flush().await.unwrap();
        }

    }
}

async fn nickme<W: tokio::io::AsyncWriteExt + Unpin>(writer: &mut W, nickname: &str) -> Result<(), Box<dyn std::error::Error>> {
    writer.write_all(format!("NICK {}\r\n", nickname).as_bytes()).await?;
    writer.flush().await?;
    writer.write_all(format!("USER {} 0 * :{}\r\n", nickname, nickname).as_bytes()).await?;
    writer.flush().await?;
    Ok(())
}
