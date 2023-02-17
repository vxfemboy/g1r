use std::io::prelude::*;
use std::net::TcpStream;
use std::time::{Instant, Duration};
use std::fs;
use std::thread;
use toml::Value;
use std::io::{self, Write};
use regex::Regex;
use rand::{thread_rng, Rng};
use openssl::ssl::{SslMethod, SslConnector, SslStream};
use async_openai::{Client, types::{CreateCompletionRequestArgs, ResponseFormat}};

mod modules {
    pub trait Command {
        fn handle(&self, message: &str) -> Vec<String>;
    }
    pub mod ping;
    pub mod kill;
    pub mod ai;
}
use modules::ai::Ai;
use modules::ping::PingCommand;
use modules::kill::KillCommand;
use crate::modules::Command;
fn main() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    let connector = SslConnector::builder(SslMethod::tls()).unwrap().build();
// PUT CONFIG IN A SEPRATE FILE IE: YAML, JSON, CONFIG, TOML12
    let stream = TcpStream::connect("ircd.chat:6697").unwrap(); // setup tor & custom masking
    let mut ssl_stream = connector.connect("ircd.chat", stream).unwrap();

    let nick_command = "NICK g1r\r\n"; // set SASL Passwords User Nicks
    let user_command = "USER g1r 0 * :g1r\r\n";


    let channels = vec!["#tcpdirect", "#macros"]; // CHANNELS
    let join_command = format!("JOIN {}\r\n", channels.join(","));

    let admin_users = vec!["s4d", "s4d[m]"]; // ADMINS
    let ignored_users = vec!["maple", "aibird", "proffesserOak"]; // IGNORED
// ... 
    ssl_stream.write_all(nick_command.as_bytes()).unwrap();
    ssl_stream.write_all(user_command.as_bytes()).unwrap();
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
