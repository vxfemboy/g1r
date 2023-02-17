use std::io::prelude::*;
use std::net::TcpStream;
use std::time::{Instant, Duration};
use std::fs;
use std::thread;
use std::io::{self, Write};
use regex::Regex;
use rand::{thread_rng, Rng};
use openssl::ssl::{SslMethod, SslConnector, SslStream};
use async_openai::{Client, types::{CreateCompletionRequestArgs, ResponseFormat}};
use toml::{from_str, Value};
use serde::Deserialize;
mod modules {
    pub trait Command {
        fn handle(&self, message: &str) -> Vec<String>;
    }
    pub mod ping;
    pub mod kill;
    pub mod ai;
}

use modules::ai::Ai; // FIX THIS BS
use modules::ping::PingCommand;
use modules::kill::KillCommand; // ...
use crate::modules::Command;

#[derive(Deserialize)]
struct Config {
    server: String,
    port: u16,
    nick: String,
    password: String,
    channels: Vec<String>,
}

fn main() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

// PUT CONFIG IN A SEPRATE FILE IE: CONFIG.TOML
    // read the contents of the config file into a string
    let config_str = std::fs::read_to_string("config.toml").unwrap();

    // parse the string into a toml::Value
    let config_value = config_str.parse::<Value>().unwrap();

    // deserialize the value into a Config struct
    let config: Config = config_value.try_into().unwrap();

    let stream = TcpStream::connect(format!("{}:{}", config.server, config.port)).unwrap();; // DONT DO DRUGS YOU WILL END UP LIKE ME 
    let connector = SslConnector::builder(SslMethod::tls()).unwrap().build();
    let mut ssl_stream = connector.connect(&config.server, stream).unwrap();
    let nick_command = format!("NICK {}_\r\n", config.nick); //setup passwords
    let user_command = format!("USER {} 0 * :{}\r\n", config.nick, config.nick);
    ssl_stream.write_all(nick_command.as_bytes()).unwrap();
    ssl_stream.write_all(user_command.as_bytes()).unwrap();

    let identify_command = format!("PRIVMSG NickServ :IDENTIFY {} {}\r\n", config.nick, config.password);
    ssl_stream.write(identify_command.as_bytes()).unwrap();
    let channels = config.channels.join(",");
    let join_command = format!("JOIN {}\r\n", channels);
    
    let admin_users = vec!["s4d", "s4d[m]"]; // ADMINS
    let ignored_users = vec!["maple", "aibird", "proffesserOak"]; // IGNORED
// ... 
    ssl_stream.write_all(join_command.as_bytes()).unwrap();

    let mut buf = [0; 512];
    loop {
        match ssl_stream.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                let received = String::from_utf8_lossy(&buf[0..n]);
                let message = received.trim();

                //debug chat 
                println!("{}", received); // ADD COLORS


                // RESPOND TO PINGS
                if message.starts_with("PING") {
                    println!("[%] PONG");
                    ssl_stream.write_all("PONG ircd.chat\r\n".as_bytes()).unwrap();
                    continue; // skip processing the PING message further
                }

                // MODULES
                let ping_command = PingCommand;
                let kill_command = KillCommand;
                let ai = Ai;

                // ADMIN MODULES
                if message.starts_with(":") && message.contains(" :%") {
                    let parts: Vec<&str> = message.splitn(2, ' ').collect(); // Check if user is admin_user
                    let username = parts[0].trim_start_matches(':').split("!").next().unwrap();
                    if !admin_users.contains(&username) {
                        println!("[!] UNAUTHORIZED: {}", username);
                        continue; // ...
                    }
                    if message.contains(":%ping") {
                        for response in ping_command.handle(message) {
                            ssl_stream.write_all(response.as_bytes()).unwrap();
                        }
                    } else if message.contains(":%kill") {
                        for response in kill_command.handle(message) {
                            ssl_stream.write_all(response.as_bytes()).unwrap();
                        }
                    }
                }

                // Check if the message is user and respond via ai
                else if message.starts_with(":") && message.contains("PRIVMSG ") && message.contains("g1r") { //modify for on mention 
                    let channel = message.split("PRIVMSG ").nth(1).and_then(|s| s.splitn(2, ' ').next()).unwrap();
                    if !channels.contains(&channel) {
                        continue;
                    }
                    // extract the username from the first part and check if ignored
                    let parts: Vec<&str> = message.splitn(2, ' ').collect(); // split the message into two parts at the first space
                    let username = parts[0].trim_start_matches(':').split("!").next().unwrap();
                    if ignored_users.contains(&username) {
                        println!("[!] IGNORED: {}", username); 
                        continue;
                    }
                    for response in ai.handle(message, ) {
                        ssl_stream.write_all(response.as_bytes()).unwrap();
                    }

                }

            },
            Err(e) => {
                println!("Error: {}", e);
                break;
            },
        }
    }
}
