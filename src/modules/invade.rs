use crate::modules::Command;

use std::cell::RefCell;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::rc::Rc;

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
            let num_bots = parts[4].parse::<u32>().unwrap_or(1) as usize;
            let channel = parts[2];
            let invadechannel = parts[5];
            let scream = parts[6];
            let config_str = std::fs::read_to_string("config.toml").unwrap();
            let config_value = config_str.parse::<Value>().unwrap();
            let config: Config = config_value.try_into().unwrap();

            for invader in &config.invaders[0..num_bots] {
                let thread_channel = invadechannel.to_string();
                let thread_invader = invader.to_string();
                let config_clone = config.clone();
                let screaming = scream.to_string();
                

                std::thread::spawn(move || {
                    let stream = TcpStream::connect(format!("{}:{}", config_clone.server, config_clone.port)).unwrap();
                    let connector = SslConnector::builder(SslMethod::tls()).unwrap().build();
                    let ssl_stream = connector.connect(&config_clone.server, stream).unwrap();
                    let ssl_stream = Rc::new(RefCell::new(ssl_stream));
                    let nick_command = format!("NICK {}\r\n", thread_invader);
                    let user_command = format!("USER {} 0 * :{}\r\n", thread_invader, thread_invader);
                    ssl_stream.borrow_mut().write_all(nick_command.as_bytes()).unwrap();
                    ssl_stream.borrow_mut().write_all(user_command.as_bytes()).unwrap();
                    let join_command = format!("JOIN {} \r\n", thread_channel);
                    ssl_stream.borrow_mut().write_all(join_command.as_bytes()).unwrap();
                    let msg = format!("PRIVMSG {} :{}\r\n", thread_channel, screaming);
                    ssl_stream.borrow_mut().write_all(msg.as_bytes()).unwrap();


                    loop {
                        let mut ssl_stream_ref = ssl_stream.borrow_mut();
                        let mut reader = BufReader::new(&mut *ssl_stream_ref);
                                        
                        let mut message = String::new();
                        match reader.read_line(&mut message) {
                            Ok(0) => break,
                            Ok(_) => {
                                if message.starts_with("PING") {
                                    let response = message.replace("PING", "PONG");
                                    ssl_stream.borrow_mut().write_all(response.as_bytes()).unwrap();
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

            response.push(format!("PRIVMSG {} :INVADING with {} bots..\r\n", channel, num_bots));
        }

        response
    }
}
