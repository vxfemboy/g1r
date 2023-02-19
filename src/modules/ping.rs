
use std::time::{Instant};
use crate::modules::Command;
pub struct PingCommand;
impl Command for PingCommand {
    fn handle(&self, message: &str) -> Vec<String> {
        let mut response = vec![];
                
        if message.contains("PRIVMSG") && message.contains(":%ping") {
            let channel = message.split("PRIVMSG ").nth(1).and_then(|s| s.splitn(2, ' ').next()).unwrap();
            let start = Instant::now();
            let elapsed = start.elapsed();            
            response.push(format!("PRIVMSG {} :PONG: {:?}\r\n", channel, elapsed));
            }
        response
    }
}
