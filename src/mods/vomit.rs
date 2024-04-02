
use rand::prelude::*;
use tokio::io::AsyncWriteExt;
use tokio::time;
use crate::Config;

async fn generate_random_unicode() -> char {
    let codepoint: u32 = thread_rng().gen_range(0..=0x10FFFF);
    std::char::from_u32(codepoint).unwrap_or(' ')
}

async fn generate_random_color() -> u8 {
    thread_rng().gen_range(0..=15)
}

async fn generate_random_format() -> char {
    match thread_rng().gen_range(0..=2) {
        0 => '\x02', // Bold
        1 => '\x1D', // Italic
        _ => '\x1F', // Underline
    }
}

async fn generate_vomit(length: usize) -> String {
    let mut irc_text = String::new();
    for _ in 0..length {
        let char = generate_random_unicode().await;
        let color_code = generate_random_color().await;
        let format_code = generate_random_format().await;
        irc_text.push_str(&format!("\x03{:02}{}", color_code, format_code));
        irc_text.push(char);
    }
    irc_text
}

fn split_into_chunks(s: &str, max_chunk_size: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current_chunk = String::new();

    for char in s.chars() {
        if current_chunk.len() + char.len_utf8() > max_chunk_size {
            chunks.push(current_chunk.clone()); 
            current_chunk.clear();
        }
        current_chunk.push(char);
    }

    if !current_chunk.is_empty() {
        chunks.push(current_chunk);
    }

    chunks
}


const CHUNK_SIZE: usize = 400;
// Function to handle the vomit command
pub async fn handle_vomit_command<W: AsyncWriteExt + Unpin>(
    writer: &mut W,
    config: &Config,
    command: &str,
    channel: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let parts: Vec<&str> = command.split_whitespace().collect();
    let length = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(1000);

    let vomit = generate_vomit(length).await; // Correctly awaited
    let chunks = split_into_chunks(&vomit, CHUNK_SIZE); // Adjust if split_into_chunks is async

    for chunk in chunks {
        writer.write_all(format!("PRIVMSG {} :{}\r\n", channel, chunk).as_bytes()).await?;
        writer.flush().await?;
        time::sleep(tokio::time::Duration::from_secs(config.pump_delay)).await;
    }

    Ok(())
}
