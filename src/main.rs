use tokio::io::{split, AsyncRead, AsyncWrite, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_native_tls::native_tls::TlsConnector as NTlsConnector;
use tokio_native_tls::TlsConnector;
use tokio::sync::mpsc;
use serde::Deserialize;
use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use colored::*;
use tokio_socks::tcp::Socks5Stream;

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
    
    // Proxy
    use_proxy: bool,
    proxy_type: Option<String>,
    proxy_addr: Option<String>,
    proxy_port: Option<u16>,
    proxy_username: Option<String>,
    proxy_password: Option<String>,
}

mod mods {
    pub mod sasl;
    pub mod sed;
}
use mods::sasl::{start_sasl_auth, handle_sasl_messages};
use mods::sed::{SedCommand, MessageBuffer};


#[tokio::main(flavor = "multi_thread", worker_threads = 12)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tokio::spawn(async move {
    println!("Loading Config...");
    let config = loaded_config().expect("Error parsing config.toml");
    println!("Config loaded!");

    let server = format!("{}:{}", config.server, config.port);

    if config.use_ssl {
        if config.use_proxy {
            let tcp_stream = proxy_exec(&config).await;
            match tcp_stream {
                Ok(tcp_stream) => {
                    let tls_stream = tls_exec(&config, tcp_stream).await;
                    match tls_stream {
                        Ok(tls_stream) => {
                            if let Err(e) = handler(tls_stream, config).await {
                                println!("Error handling TLS connection: {}", e);
                            }
                        },
                        Err(e) => {
                            println!("Error establishing TLS connection: {}", e);
                        }
                    }
                    //handler(tls_stream, config).await.unwrap();
                },
                Err(e) => {
                    println!("Error connecting to proxy: {}", e);
                }
            }
            //let tls_stream = tls_exec(&config, tcp_stream).await
            //handler(tls_stream, config).await.unwrap();
        } else {
            let tcp_stream = TcpStream::connect(server).await.expect("Error connecting to server");
            let  tls_stream = tls_exec(&config, tcp_stream).await.expect("Error establishing TLS connection");
            handler(tls_stream, config).await.unwrap();
        }
        //let tcp_stream = TcpStream::connect(format!("{}:{}", config.server, config.port)).await;
        //println!("Connected to {}!", format!("{}:{}", config.server, config.port).green());
        //println!("Establishing TLS connection...");
        //if let Ok(tcp_stream) = TcpStream::connect(format!("{}:{}", config.server, config.port)).await {
        //    println!("TCP connection established!");
        //    let mut tls_stream = tls_exec(&config, Some(tcp_stream)).await.unwrap();
        //    println!("TLS connection established!");
        //    tls_stream.flush().await.unwrap();
        //} else {
        //    println!("TCP connection failed!");
        //};
        //let mut tls_stream = tls_exec(&config, Some(tcp_stream.unwrap())).await.unwrap();
        //println!("TLS connection established!");
        //tls_stream.flush().await.unwrap();

        //handler(tls_stream, config).await.unwrap();
    } else {
        println!("Non-SSL connection not implemented.");
    }
    }).await.unwrap(); 
    Ok(())
}

/// Load the config file
fn loaded_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_contents = fs::read_to_string("config.toml")?;
    let config: Config = toml::from_str(&config_contents)?;
    Ok(config)
}

/// Establish a TLS connection to the server
async fn tls_exec(config: &Config, tcp_stream: TcpStream) -> Result<tokio_native_tls::TlsStream<TcpStream>, Box<dyn std::error::Error + Send>> {
    let tls_builder = NTlsConnector::builder().danger_accept_invalid_certs(true).build().unwrap();
    let tls_connector = TlsConnector::from(tls_builder);
    Ok(tls_connector.connect(&config.server, tcp_stream).await.unwrap())

}

/// Establish a connection to the proxy
async fn proxy_exec(config: &Config) -> Result<TcpStream, Box<dyn std::error::Error + Send>> {
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
        let tcp_stream = Socks5Stream::connect_with_password_and_socket(proxy_stream, server, &username, &password).await.unwrap();
        tcp_stream
        
    } else {
        let tcp_stream = Socks5Stream::connect_with_socket(proxy_stream, server).await.unwrap();
        tcp_stream
    };
    let tcp_stream = tcp_stream.into_inner();

//    ok(tcp_stream)
    //Ok(tcp_stream<TcpStream>)
    Ok(tcp_stream)
}

/// Handle the connection to the server
async fn handler(tls_stream: tokio_native_tls::TlsStream<TcpStream>, config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let (reader, writer) = split(tls_stream);
    let (tx, rx) = mpsc::channel(1000);
    
    let read_task = tokio::spawn(async move {
        readmsg(reader, tx).await;
    });

    let message_buffer = MessageBuffer::new(1000);

    let write_task = tokio::spawn(async move {
        writemsg(writer, rx, &config, message_buffer).await; 
    });

    let _ = tokio::try_join!(read_task, write_task);
    
    Ok(())
}

/// Read messages from the server
async fn readmsg(mut reader: tokio::io::ReadHalf<tokio_native_tls::TlsStream<TcpStream>>, tx: tokio::sync::mpsc::Sender<String>) {
    let mut buf = vec![0; 4096];
    while let Ok (n) = reader.read(&mut buf).await {
        if n == 0 { break; }
        let msg_list = String::from_utf8_lossy(&buf[..n]).to_string();
        for lines in msg_list.lines() {
            let msg = lines.to_string();
            println!("{}{}{} {}{} {}", "[".green().bold(), ">".yellow().bold(), "]".green().bold(), "DEBUG:".bold().yellow(), ":".bold().green(), msg.trim().purple());
            tx.send(msg).await.unwrap();
            if buf.len() == n {
                buf.resize(buf.len() * 2, 0);
            }
        }
    }
}



static SASL_AUTH: AtomicBool = AtomicBool::new(false);

/// Write messages to the server
async fn writemsg(mut writer: tokio::io::WriteHalf<tokio_native_tls::TlsStream<TcpStream>>, mut rx: tokio::sync::mpsc::Receiver<String>, config: &Config, mut message_buffer: MessageBuffer) {

    let username = config.sasl_username.clone().unwrap();
    let password = config.sasl_password.clone().unwrap();
    let nickname = config.nickname.clone();
    if !password.is_empty() && !SASL_AUTH.load(Ordering::Relaxed) {
        let capabilities = config.capabilities.clone();
        println!("Starting SASL auth...");
        start_sasl_auth(&mut writer, "PLAIN", &nickname, capabilities).await.unwrap();
        writer.flush().await.unwrap();
        SASL_AUTH.store(true, Ordering::Relaxed);
    } else {
        nickme(&mut writer, &nickname).await.unwrap();
        writer.flush().await.unwrap();
    }

    while let Some(msg) = rx.recv().await {
        let msg = msg.trim();
        if msg.is_empty() {
            continue;
        }
        let parts = msg.split(' ').collect::<Vec<&str>>();
        let serv = parts.first().unwrap_or(&"");
        let cmd = parts.get(1).unwrap_or(&"");

        println!("{} {} {} {} {}", "DEBUG:".bold().yellow(), "serv:".bold().green(), serv.purple(), "cmd:".bold().green(), cmd.purple());
        if *serv == "PING" { 
            let response = msg.replace("PING", "PONG") + "\r\n";
            println!("{} {} {}","[%] PONG:".bold().green(), nickname.blue(), response.purple());
            writer.write_all(response.as_bytes()).await.unwrap();
            writer.flush().await.unwrap();
            continue;
        }
        if (*cmd == "CAP" || msg.starts_with("AUTHENTICATE +") || *cmd == "903") && SASL_AUTH.load(Ordering::Relaxed) {
            println!("Handling SASL messages...");
            handle_sasl_messages(&mut writer, msg.trim(), &username, &password, &nickname).await.unwrap();
            writer.flush().await.unwrap();
        }
        if *cmd == "001" {
            println!("Setting mode");
            writer.write_all(format!("MODE {} +B\r\n", nickname).as_bytes()).await.unwrap();
            writer.flush().await.unwrap();
        }
        
        if *cmd == "376" {
            println!("Joining channel");
            writer.write_all(format!("JOIN {}\r\n", config.channel).as_bytes()).await.unwrap();
            writer.flush().await.unwrap();
        }
        if *cmd == "PRIVMSG" {
            let channel = parts[2];
            let user = parts[0].strip_prefix(':').unwrap().split_at(parts[0].find('!').unwrap()).0.strip_suffix('!').unwrap();
            let host = parts[0].split_at(parts[0].find('!').unwrap()).1;
            let msg_content = parts[3..].join(" ").replace(':', "");
            println!("{} {} {} {} {} {} {}", "DEBUG:".bold().yellow(), "channel:".bold().green(), channel.purple(), "user:".bold().green(), host.purple(), "msg:".bold().green(), msg_content.purple());
            // sed
            if msg_content.starts_with("s/") {
                println!("Sed command detected");
                if let Some(sed_command) = SedCommand::parse(&msg_content) {
                    if let Some(response) = message_buffer.apply_sed_command(&sed_command) {
                        writer.write_all(format!("PRIVMSG {} :{}: {}\r\n", channel, user, response).as_bytes()).await.unwrap();
                        writer.flush().await.unwrap();
                    }
                }
            } else {
                message_buffer.add_message(msg_content);
            }
            // other commands here
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

