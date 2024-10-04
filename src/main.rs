use colored::*;
use serde::Deserialize;
use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::io::{
    AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt,
};
use tokio::net::TcpStream;

#[derive(Deserialize, Clone)]
struct Config {
    server: String,
    port: u16,
    use_ssl: bool,
    nickname: String,
    realname: Option<String>,
    channels: Vec<String>,
    sasl_username: Option<String>,
    sasl_password: Option<String>,
    capabilities: Option<Vec<String>>,
    reconnect_delay: u64,
    reconnect_attempts: u64,
    use_proxy: bool,
    proxy_addr: Option<String>,
    proxy_port: Option<u16>,
    proxy_username: Option<String>,
    proxy_password: Option<String>,
    ascii_art: Option<String>,
    pump_delay: u64,
}

mod mods {
    pub mod ascii;
    pub mod drugs;
    pub mod handler;
    pub mod proxy;
    pub mod sasl;
    pub mod sed;
    pub mod tls;
    pub mod vomit;
    //    pub mod invade;
}

use mods::ascii::handle_ascii_command;
use mods::drugs::Drugs;
use mods::handler::handler;
use mods::proxy::proxy_exec;
use mods::sasl::{handle_sasl_messages, start_sasl_auth};
use mods::sed::{MessageBuffer, SedCommand};
use mods::tls::tls_exec;
use mods::vomit::handle_vomit_command;
//use mods::invade::{handle_invade_command};

#[tokio::main(flavor = "multi_thread", worker_threads = 12)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Loading Config...");
    let config = loaded_config().expect("Error parsing config.toml");
    println!("Config loaded!");
    let mut reconnect_attempts = 0;

    while reconnect_attempts < config.reconnect_attempts {
        let configc = config.clone();

        let server = format!("{}:{}", configc.server, configc.port);
        let connection_result = tokio::spawn(async move {
            let config = configc.clone();
            let tcp_stream = if config.use_proxy {
                match proxy_exec(&config).await {
                    Ok(stream) => stream,
                    Err(e) => {
                        println!("Error connecting to proxy: {}", e);
                        return Ok::<(), Box<dyn std::error::Error + Send>>(());
                    }
                }
            } else {
                match TcpStream::connect(server).await {
                    Ok(stream) => stream,
                    Err(e) => {
                        println!("Error connecting to server: {}", e);
                        return Ok::<(), Box<dyn std::error::Error + Send>>(());
                    }
                }
            };

            if config.use_ssl {
                println!("Connecting to SSL server...");
                match tls_exec(&config, tcp_stream).await {
                    Ok(tls_stream) => handler(tls_stream, config).await.unwrap(),
                    Err(e) => {
                        println!("Error establishing TLS connection: {}", e);
                        return Ok::<(), Box<dyn std::error::Error + Send>>(());
                    }
                }
            } else {
                println!("Connecting to Non-SSL server...");
                handler(tcp_stream, config).await.unwrap();
            }
            Ok::<(), Box<dyn std::error::Error + Send>>(())
        })
        .await
        .unwrap();

        match connection_result {
            Ok(_) => {
                println!("Connection established successfully!");
                reconnect_attempts = 0;
            }
            Err(e) => {
                println!("Error handling connection: {}", e);
                reconnect_attempts += 1;
                tokio::time::sleep(tokio::time::Duration::from_secs(config.reconnect_delay)).await;
            }
        }
    }
    println!("Reconnect attempts exceeded. Exiting...");
    Ok(())
}

fn loaded_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_contents = fs::read_to_string("config.toml")?;
    let config: Config = toml::from_str(&config_contents)?;
    Ok(config)
}

async fn readmsg<S>(mut reader: tokio::io::ReadHalf<S>, tx: tokio::sync::mpsc::Sender<String>)
where
    S: AsyncRead + Unpin,
{
    let mut buf = vec![0; 4096];
    while let Ok(n) = reader.read(&mut buf).await {
        if n == 0 {
            break;
        }
        let msg_list = String::from_utf8_lossy(&buf[..n]).to_string();
        for lines in msg_list.lines() {
            let msg = lines.to_string();
            println!(
                "{}{}{} {}{} {}",
                "[".green().bold(),
                ">".yellow().bold(),
                "]".green().bold(),
                "DEBUG:".bold().yellow(),
                ":".bold().green(),
                msg.trim().purple()
            );
            tx.send(msg).await.unwrap();
            if buf.len() == n {
                buf.resize(buf.len() * 2, 0);
            }
        }
    }
}

static SASL_AUTH: AtomicBool = AtomicBool::new(false);

async fn writemsg<S>(
    mut writer: tokio::io::WriteHalf<S>,
    mut rx: tokio::sync::mpsc::Receiver<String>,
    config: &Config,
    mut message_buffer: MessageBuffer,
) where
    S: AsyncWrite + Unpin,
{
    let username = config.sasl_username.clone().unwrap();
    let password = config.sasl_password.clone().unwrap();
    let nickname = config.nickname.clone();
    let realname = config.realname.clone().unwrap_or(nickname.clone());
    if !password.is_empty() && !SASL_AUTH.load(Ordering::Relaxed) {
        let capabilities = config.capabilities.clone();
        println!("Starting SASL auth...");
        start_sasl_auth(&mut writer, "PLAIN", &nickname, &realname, capabilities)
            .await
            .unwrap();
        writer.flush().await.unwrap();
        SASL_AUTH.store(true, Ordering::Relaxed);
    } else {
        nickme(&mut writer, &nickname, &realname).await.unwrap();
        writer.flush().await.unwrap();
    }

    let mut drugs = Drugs::new();

    while let Some(msg) = rx.recv().await {
        let msg = msg.trim();
        if msg.is_empty() {
            continue;
        }
        let parts = msg.split(' ').collect::<Vec<&str>>();
        let serv = parts.first().unwrap_or(&"");
        let cmd = parts.get(1).unwrap_or(&"");

        println!(
            "{} {} {} {} {}",
            "DEBUG:".bold().yellow(),
            "serv:".bold().green(),
            serv.purple(),
            "cmd:".bold().green(),
            cmd.purple()
        );
        if *serv == "PING" {
            let response = msg.replace("PING", "PONG") + "\r\n";
            println!(
                "{} {} {}",
                "[%] PONG:".bold().green(),
                nickname.blue(),
                response.purple()
            );
            writer.write_all(response.as_bytes()).await.unwrap();
            writer.flush().await.unwrap();
            continue;
        }
        if (*cmd == "CAP" || msg.starts_with("AUTHENTICATE +") || *cmd == "903")
            && SASL_AUTH.load(Ordering::Relaxed)
        {
            println!("Handling SASL messages...");
            handle_sasl_messages(&mut writer, msg.trim(), &username, &password, &nickname)
                .await
                .unwrap();
            writer.flush().await.unwrap();
        }
        if *cmd == "001" {
            println!("Setting mode");
            writer
                .write_all(format!("MODE {} +B\r\n", nickname).as_bytes())
                .await
                .unwrap();
            writer.flush().await.unwrap();
        }

        if *cmd == "376" {
            println!("Joining channels");
            for channel in &config.channels {
                writer
                    .write_all(format!("JOIN {}\r\n", channel).as_bytes())
                    .await
                    .unwrap();
                writer.flush().await.unwrap();
            }
        }
        if *cmd == "KICK" {
            let channel = parts.get(2).unwrap_or(&"");
            let userme = parts.get(3).unwrap_or(&"");
            if *userme == nickname {
                writer
                    .write_all(format!("JOIN {}\r\n", channel).as_bytes())
                    .await
                    .unwrap();
                writer.flush().await.unwrap();
            }
        }
        if *cmd == "PRIVMSG" {
            let channel = &parts.get(2).to_owned().unwrap_or(&"");
            let user = parts[0]
                .strip_prefix(':')
                .and_then(|user_with_host| user_with_host.split('!').next())
                .unwrap_or("unknown_user");
            let host = parts[0].split('@').nth(1).unwrap_or("unknown_host");
            let msg_content = if parts.len() > 3 {
                let remainder = &parts[3..].join(" ");
                if let Some(pos) = remainder.find(':') {
                    let (first_part, last_part) = remainder.split_at(pos);
                    format!("{}{}", first_part, &last_part[1..])
                } else {
                    remainder.to_string()
                }
            } else {
                "".to_string()
            };
            println!(
                "{} {} {} {} {} {} {} {} {}",
                "DEBUG:".bold().yellow(),
                "channel:".bold().green(),
                channel.purple(),
                "user:".bold().green(),
                user.purple(),
                "host:".bold().green(),
                host.purple(),
                "msg:".bold().green(),
                msg_content.yellow()
            );

            if msg_content.starts_with("s/") {
                if let Some(sed_command) = SedCommand::parse(&msg_content.clone()) {
                    if let Some(response) = message_buffer.apply_sed_command(&sed_command) {
                        writer
                            .write_all(format!("PRIVMSG {} :{}\r\n", channel, response).as_bytes())
                            .await
                            .unwrap();
                        writer.flush().await.unwrap();
                    }
                }
            } else {
                message_buffer.add_message(msg_content.clone().to_string());
            }

            if msg_content.starts_with("%ascii") {
                let _ = handle_ascii_command(&mut writer, config, &msg_content, channel).await;
            }

            if msg_content.starts_with("%vomit") {
                let _ = handle_vomit_command(&mut writer, config, &msg_content, channel).await;
            }

            if [
                "%chug", "%smoke", "%toke", "%100", "%extendo", "%fatfuck", "%beer",
            ]
            .iter()
            .any(|&prefix| msg_content.starts_with(prefix))
            {
                drugs
                    .handle_drugs_command(&mut writer, config, &msg_content, channel)
                    .await
                    .unwrap_or_else(|e| eprintln!("Error handling drugs command: {}", e));
            }
        }
    }
}

async fn nickme<W: tokio::io::AsyncWriteExt + Unpin>(
    writer: &mut W,
    nickname: &str,
    realname: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    writer
        .write_all(format!("NICK {}\r\n", nickname).as_bytes())
        .await?;
    writer.flush().await?;
    writer
        .write_all(format!("USER {} 0 * :{}\r\n", nickname, realname).as_bytes())
        .await?;
    writer.flush().await?;
    Ok(())
}
