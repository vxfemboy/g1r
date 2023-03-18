use crate::modules::Command;

use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use openssl::ssl::{SslConnector, SslMethod};
use serde::Deserialize;
use toml::{Value, to_string};
use colored::*;

//use anyhow::Result;
//use socks5_proxy::{client, Addr};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

// add better error handling
// add ai invasion
// add proxy support

#[derive(Clone, Deserialize)]
struct Config {
    //invaders: Vec<String>,
    server: String,
    port: u16,
    
    proxy_server: String,
    proxy_port: u16,
    
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
            let scream = if parts.len() > 6 { parts[6] } else { "" }; // read entire message
            //message.split(&format!("PRIVMSG {} :", channel.to_string())).nth(1).unwrap(),
            //let channel = message.split("PRIVMSG ").nth(1).and_then(|s| s.splitn(2, ' ').next()).unwrap();
            let config_str = std::fs::read_to_string("config.toml").unwrap();
            let config_value = config_str.parse::<Value>().unwrap();
            let config: Config = config_value.try_into().unwrap();


            for _invader in 1..=num_invaders {//&config.invaders[1..num_invaders] { //1..=20 {
                let thread_channel = invadechannel.to_string();
                let config_clone = config.clone();
                let screaming = scream.to_string();
                let command_channel = channel.to_string();
                let thread_invader = random_word::gen(); // change to leetspeak on nick collision
                
                std::thread::spawn(move || {

                    let stream = TcpStream::connect((config_clone.server.as_str(), config_clone.port)).unwrap();
                    let connector = SslConnector::builder(SslMethod::tls()).unwrap().build();
                    let mut ssl_stream = connector.connect(config_clone.server.as_str(), stream).unwrap();

                    let nick_command = format!("NICK {}\r\n", thread_invader);
                    let user_command = format!("USER {} 0 * :{}\r\n", thread_invader, thread_invader);
                    ssl_stream.write_all(nick_command.as_bytes()).unwrap();
                    ssl_stream.write_all(user_command.as_bytes()).unwrap();
                    let join_command = format!("JOIN {} \r\n", thread_channel);
                    let commander = format!("JOIN {} \r\n", command_channel);
                    ssl_stream.write_all(commander.as_bytes()).unwrap();
                    ssl_stream.write_all(join_command.as_bytes()).unwrap();
                    let msg = format!("PRIVMSG {} :{}\r\n", thread_channel, screaming);
                    ssl_stream.write_all(msg.as_bytes()).unwrap();

                    loop {


                        
                        let mut buf = [0; 512];
                        match ssl_stream.ssl_read(&mut buf) {
                            Ok(0) => break,
                            Ok(n) => {
                                let received = String::from_utf8_lossy(&buf[0..n]);
                                let message = received.trim();
                
                                //debug chat 
                                println!("{} {} {}","[%] DEBUG:".bold().green(), thread_invader.green(), received.blue());

                                if message.starts_with("PING") {
                                    let response = message.replace("PING", "PONG");
                                    println!("{} {}","[%] PONG:".bold().green(), thread_invader.blue());
                                    ssl_stream.write_all(response.as_bytes()).unwrap();
                                }
                            // turn to mods
                                // setup so these will only run from the server admin to avoid handle/host conflicts 
                                let commandi = format!("PRIVMSG {} :%%",command_channel); // & check for admin and verify with server 
                                
                                if message.contains(&commandi) && message.contains(":%%") {
                                    if message.contains("PRIVMSG") && message.contains(":%%join") { // fix so commands get picked up faster
                                        let parts: Vec<&str> = message.splitn(3, ":%%join ").collect();
                                        let invade_channel = parts[1];
                                        let response = format!("JOIN {} \r\n", invade_channel);
                                        ssl_stream.write_all(response.as_bytes()).unwrap();
                                    }
                                    if message.contains("PRIVMSG") && message.contains(":%%leave") {
                                        let parts: Vec<&str> = message.splitn(3, ":%%leave ").collect();
                                        let invade_channel = parts[1];
                                        let response = format!("PART {} \r\n", invade_channel);
                                        ssl_stream.write_all(response.as_bytes()).unwrap();
                                    }
                                    if message.contains("PRIVMSG") && message.contains(":%%scream") {
                                        let parts: Vec<&str> = message.splitn(3, ":%%scream ").collect();
                                        let invade_channel = parts[1];
                                        if parts.len() == 2 {
                                            let scream = parts[1];
                                            let response = format!("PRIVMSG {} :{}\r\n", invade_channel, scream);
                                            ssl_stream.write_all(response.as_bytes()).unwrap();
                                        }
                                    }
                                }
                            // ...1
                            }
                            Err(e) => {
                                eprintln!("{} {}","[!] ERROR FROM SERVER:".on_red(), e);
                                break;
                            }
                        }
                    }
                });
            }

            response.push(format!("PRIVMSG {} :\x0304,01[!] INVADING {} WITH {} INVADERS...\x0f\r\n", channel, invadechannel, num_invaders));
        }

        response
    }
}