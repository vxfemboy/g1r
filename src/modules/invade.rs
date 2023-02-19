use crate::modules::Command;

use std::io::{Write};
use std::net::TcpStream;

use openssl::ssl::{SslConnector, SslMethod};
use serde::Deserialize;
use toml::Value;

#[derive(Clone, Deserialize)]
struct Config {
    invaders: Vec<String>,
    server: String,
    port: u16,
}

pub struct InvadeCommand;

impl Command for InvadeCommand {
    fn handle(&self, message: &str) -> Vec<String> {
        let mut response = vec![];

        if message.contains("PRIVMSG") && message.contains(":%invade") {
            let parts: Vec<&str> = message.split_whitespace().collect();
            let num_invaders = parts[4].parse::<u32>().unwrap_or(1) as usize;
            let channel = parts[2];
            let invadechannel = parts[5];
            let scream = if parts.len() > 6 { parts[6] } else { "" };
            let config_str = std::fs::read_to_string("config.toml").unwrap();
            let config_value = config_str.parse::<Value>().unwrap();
            let config: Config = config_value.try_into().unwrap();

            for invader in &config.invaders[0..num_invaders] {
                let thread_channel = invadechannel.to_string();
                let thread_invader = invader.to_string();
                let config_clone = config.clone();
                let screaming = scream.to_string();

                std::thread::spawn(move || {
                    let stream = TcpStream::connect((config_clone.server.as_str(), config_clone.port)).unwrap();
                    let connector = SslConnector::builder(SslMethod::tls()).unwrap().build();
                    let mut ssl_stream = connector.connect(config_clone.server.as_str(), stream).unwrap();
                    let nick_command = format!("NICK {}\r\n", thread_invader);
                    let user_command = format!("USER {} 0 * :{}\r\n", thread_invader, thread_invader);
                    ssl_stream.write_all(nick_command.as_bytes()).unwrap();
                    ssl_stream.write_all(user_command.as_bytes()).unwrap();
                    let join_command = format!("JOIN {} \r\n", thread_channel);
                    ssl_stream.write_all(join_command.as_bytes()).unwrap();
                    let msg = format!("PRIVMSG {} :{}\r\n", thread_channel, screaming);
                    ssl_stream.write_all(msg.as_bytes()).unwrap();

                    loop {
                        let mut buffer = [0; 512];
                        match ssl_stream.ssl_read(&mut buffer) {
                            Ok(0) => break,
                            Ok(n) => {
                                let message = String::from_utf8_lossy(&buffer[..n]);
                                if message.starts_with("PING") {
                                    let response = message.replace("PING", "PONG");
                                    println!("[%] PONG {}", thread_invader);
                                    ssl_stream.write_all(response.as_bytes()).unwrap();
                                }
                            }
                            Err(e) => {
                                eprintln!("Error reading from server: {}", e);
                                break;
                            }
                        }
                    }
                });
            }

            response.push(format!("PRIVMSG {} :INVADING WITH {} INVADERS...\r\n", channel, num_invaders));
        }

        response
    }
}