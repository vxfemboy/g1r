
use crate::modules::Command;

pub struct KillCommand;
impl Command for KillCommand {
    fn handle(&self, message: &str) -> Vec<String> {
        let mut response = vec![];
                
        if message.contains("PRIVMSG") && message.contains(":%kill") {
            let channel = message.split("PRIVMSG ").nth(1).and_then(|s| s.splitn(2, ' ').next()).unwrap();
            response.push(format!("PRIVMSG {} :SELF DESTRUCTING...\r\n", channel));
            println!("[!] KILLING!");
            std::process::exit(0);
        }
        response
    }
}
