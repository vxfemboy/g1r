use colored::*;
use openssl::ssl::{SslConnector, SslMethod, SslVerifyMode};
use serde::Deserialize;
use std::fs;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::TcpStream;
use tokio_openssl::SslStream;
use std::pin::Pin;

#[derive(Deserialize)]
struct Config {
    server: String,
    port: u16,
    use_ssl: bool,
    nickname: String,
    channel: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<(dyn std::error::Error)>> {
    println!("Loading Config...");
    let config_contents = fs::read_to_string("config.toml").expect("Error reading config.toml");
    let config: Config = toml::from_str(&config_contents).expect("Error parsing config.toml");
    println!("Config loaded!");

    let addr = format!("{}:{}", config.server, config.port);
    println!("Connecting to {}...", addr.green());
    let tcp_stream = TcpStream::connect(&addr).await.unwrap();
    println!("Connected to {}!", addr.green());

    if config.use_ssl {
        println!("Establishing SSL connection...");
        let mut connector = SslConnector::builder(SslMethod::tls()).unwrap().build().configure().unwrap().into_ssl(&addr).unwrap();
        connector.set_verify(SslVerifyMode::NONE);
        let mut ssl_stream = SslStream::new(connector, tcp_stream).unwrap();

        // Perform the SSL handshake
        match Pin::new(&mut ssl_stream).connect().await {
            Ok(_) => println!("SSL connection established!"),
            Err(e) => {
                println!("Error establishing SSL connection: {:?}", e);
                return Err(Box::new(e) as Box<dyn std::error::Error>);
            }
        };

        println!("Sending NICK and USER commands...");
        ssl_stream.write_all(format!("NICK {}\r\n", config.nickname).as_bytes()).await.unwrap();
        ssl_stream.write_all(format!("USER {} 0 * :{}\r\n", config.nickname, config.nickname).as_bytes()).await.unwrap();
        ssl_stream.write_all(format!("JOIN {}\r\n", config.channel).as_bytes()).await.unwrap();


        let (read_half, write_half) = tokio::io::split(ssl_stream);

        // split the stream and then transfer this for non-ssl
        let mut reader = BufReader::new(read_half);
        let mut writer = BufWriter::new(write_half);
        let mut lines = reader.lines();

        while let Some(result) = lines.next_line().await.unwrap() {

            let received = String::from_utf8_lossy(result.as_bytes()).trim().to_string();
            println!("{} {}","[%] DEBUG:".bold().green(), received.purple());

            let message = received.trim();
            if message.starts_with("PING") {
                println!("Sending PONG...");
                let response = message.replace("PING", "PONG");
                println!("{} {}","[%] PONG:".bold().green(), config.nickname.blue());
                writer.write_all(response.as_bytes()).await.unwrap();
                continue;
            }
        }
    } else {
        println!("Non-SSL connection not implemented.");
    }

    Ok(())
}
