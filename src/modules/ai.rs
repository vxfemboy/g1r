use async_openai::{Client, types::{CreateChatCompletionRequestArgs, ChatCompletionRequestMessageArgs, Role}};
use crate::modules::Command;
use toml::Value;
use serde::Deserialize;
use regex::Regex;

#[derive(Deserialize)]
struct Config {
    nick: String,
    openai: String,
    model: String,
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
            let channel = message.split("PRIVMSG ").nth(1).and_then(|s| s.splitn(2, ' ').next()).unwrap();
            let cleaned_message = message.replace(&format!("{} ", config.nick), "");
            println!("{}", cleaned_message);
            let user_message = cleaned_message.split(&format!("PRIVMSG {} :", channel.to_string())).nth(1).unwrap();

            let parts: Vec<&str> = message.splitn(2, ' ').collect();
            let username = parts[0].trim_start_matches(':').split("!").next().unwrap();


            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(ai(&user_message, &username, &channel, &config));
            responses.extend(result);
        }

        responses
    }
}



async fn ai(user_message: &str, username: &str, channel: &str, config: &Config) -> Vec<String> {
    let api_key = &config.openai;
    let client = Client::new().with_api_key(api_key);


    let request = CreateChatCompletionRequestArgs::default()
        .model(&config.model)
        .messages([
            ChatCompletionRequestMessageArgs::default()
                .role(Role::System)
                .content(format!("Respond {} as you are chatting as {}, The following is a chat message to you from {} dont mention that you are who you are they can see that:", config.accents, config.personalities, username))
                .build()
                .unwrap(),
            ChatCompletionRequestMessageArgs::default()
                .role(Role::User)
                .content(user_message.to_string())
                .build()
                .unwrap(),
        ])
        .max_tokens(256_u16)
        .build()
        .unwrap();

    let response = client.chat().create(request).await.unwrap();
    let response_text = response.choices.first().unwrap().message.content.trim().to_string();
    let regex = Regex::new(r#""|[gG][1iI][rR]\s"#).unwrap(); // THIS IS FUCKING UP EVERYTHING

    let response_text = regex.replace_all(&response_text, "").trim().to_string();
    println!("{}", response_text);
    let response_lines = response_text.split("\n").filter(|line| !line.trim().is_empty());
    let mut responses = Vec::new();
    let mut line_count = 0;



    for line in response_lines {
        if line.contains("*") {
            let parts: Vec<&str> = line.split("*").collect();
            for (i, part) in parts.iter().enumerate() {
                if i % 2 == 1 {
                    let action = part.to_lowercase();
                    responses.push(format!("PRIVMSG {} :\x01ACTION {}\x01\r\n", channel, action));
                } else {
                    let message = part.trim().to_string();
                    responses.push(format!("PRIVMSG {} :{}\r\n", channel, message));
                }
            }
        } else {
            let response_line = if line_count == 0 && !line.contains(&username) {
                format!("PRIVMSG {} :\x0313{}\x0f: {}\r\n", channel, username, line)
            } else {
                format!("PRIVMSG {} :{}\r\n", channel, line)
            };
            responses.push(response_line);
        }
    
        line_count += 1;
    }
    

    responses
}