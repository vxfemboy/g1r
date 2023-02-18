use crate::modules::Command;
use std::io::{Write};
use std::net::TcpStream;
use openssl::ssl::{SslConnector, SslMethod};
use serde::Deserialize;
use toml::Value;
use std::thread;

#[derive(Clone, Deserialize)]
struct Config {
    invaders: Vec<String>,
    server: String,
    port: u16,
}

pub struct InvadersCommand;

impl Command for InvadersCommand {
    fn handle(&self, message: &str) -> Vec<String> {
        let mut response = vec![];

        if message.contains("PRIVMSG") && message.contains(":%invaders") {
            let parts: Vec<&str> = message.split_whitespace().collect();
            let scream = parts[1];

            let config_str = std::fs::read_to_string("config.toml").unwrap();
            let config_value = config_str.parse::<Value>().unwrap();
            let config: Config = config_value.try_into().unwrap();

            for invader in &config.invaders {
                let thread_channel = parts[2].to_string();
                let thread_invader = invader.to_string();
                let screaming = scream.to_string();

                std::thread::spawn(move || {
                    let stream = TcpStream::connect((config.server.as_str(), config.port)).unwrap();
                    let connector = SslConnector::builder(SslMethod::tls()).unwrap().build();
                    let mut ssl_stream = connector.connect(config.server.as_str(), stream).unwrap();

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

            response.push(format!("PRIVMSG {} :Screaming \"{}\" through {} invaders..\r\n", parts[2], scream, config.invaders.len()));
        }

        response
    }
}