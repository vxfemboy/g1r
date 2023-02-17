// Check if the message is user and respond via ai
use async_openai::{Client, types::{CreateCompletionRequestArgs}};
use regex::Regex;
use crate::modules::Command;
use toml::{from_str, Value};
use serde::Deserialize;
#[derive(Deserialize)]
struct Config {
    nick: String,
    channels: Vec<String>,
    openai: String,
    accents: String,
    personalities: String,
}
pub struct Ai;
   
impl Command for Ai {
    fn handle(&self, message: &str) -> Vec<String> {
        let mut responses = Vec::new();
        let config_str = std::fs::read_to_string("config.toml").unwrap();
        let config_value = config_str.parse::<Value>().unwrap();
        let config: Config = config_value.try_into().unwrap();
        if message.starts_with(":") && message.contains("PRIVMSG ") && message.contains(&config.nick) { 
            let channel = message.split("PRIVMSG ").nth(1).and_then(|s| s.splitn(2, ' ').next()).unwrap(); // set the response to varible
            let user_message = format!("The following is a chat log:\n{}\nRespond {} as you are chatting as {}: \n\n", 
                message.split(&format!("PRIVMSG {} :", channel.to_string())).nth(1).unwrap(),
                config.accents,
                config.personalities
            );                    
            let parts: Vec<&str> = message.splitn(2, ' ').collect();    
            let username = parts[0].trim_start_matches(':').split("!").next().unwrap();

            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(ai(&user_message, &username, &channel));
            responses.extend(result);
        }

        responses
    }
}
async fn ai(user_message: &str, username: &str, channel: &str) -> Vec<String> {
    let config_str = std::fs::read_to_string("config.toml").unwrap();
    let config_value = config_str.parse::<Value>().unwrap();
    let config: Config = config_value.try_into().unwrap();
    let api_key = config.openai; // set this from config

    let client = Client::new().with_api_key(api_key);
    println!("[?] PROMPT: {}: {}", username, user_message);
    let chat_request = CreateCompletionRequestArgs::default()
        .prompt(user_message)
        .max_tokens(40_u16)
        .model("text-davinci-003")
        .build()
        .unwrap();
    let chat_response = client
        .completions()
        .create(chat_request)
        .await
        .unwrap();
    println!("[+] RESPONSE: {}", chat_response.choices.first().unwrap().text);
    //modify regex for varible username ie G1R g1r GIR gir but as handle nick for bots
    let response_text = &chat_response.choices.first().unwrap().text;
    let regex = Regex::new(r#""|[gG][1iI][rR]:\s*|[mM][eE]:?\s"#).unwrap(); 
    let response_text = regex.replace_all(response_text, "").trim().to_string();
    let response_lines = response_text.split("\n").filter(|line| !line.trim().is_empty());
    let mut responses = Vec::new();
    for line in response_lines {
        responses.push(format!("PRIVMSG {} :{}: {}\r\n", channel, username, line));
    }
    
    responses
}
