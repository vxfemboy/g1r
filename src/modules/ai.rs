// Check if the message is user and respond via ai
use async_openai::{Client, types::{CreateCompletionRequestArgs}};
use regex::Regex;
use crate::modules::Command;

pub struct Ai;
   
impl Command for Ai {
    fn handle(&self, message: &str) -> Vec<String> {
        let mut responses = Vec::new();
        if message.starts_with(":") && message.contains("PRIVMSG ") && message.contains("g1r") { 
            let channel = message.split("PRIVMSG ").nth(1).and_then(|s| s.splitn(2, ' ').next()).unwrap();
            let user_message = "The following is a chat log:\n".to_owned() + message.split(&format!("PRIVMSG {} :", channel.to_string())).nth(1).unwrap() + "\nRespond funny, completely insane, and hyperactive as you are chatting as GIR from Invader Zim: \n\n";                    
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
    let api_key = "sk-*"; // set this from config and add rotatation

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
