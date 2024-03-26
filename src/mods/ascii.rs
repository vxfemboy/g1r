use tokio::io::{AsyncWriteExt, BufReader};
use tokio::fs::File;
use tokio::time::{self, Duration};
use std::fs;
use rand::Rng;
use tokio::io::AsyncBufReadExt;
use std::error::Error;
use crate::Config;

const CHUNK_SIZE: usize = 4096;

async fn send_ansi_art<W: AsyncWriteExt + Unpin>(writer: &mut W, file_path: &str, pump_delay: u64, channel: &str) -> Result<(), Box<dyn Error>> {
    let file = File::open(file_path).await?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    let mut line_count = 0;
    let lines_stream = &mut lines;
    while let Ok(Some(_)) = lines_stream.next_line().await {
        line_count += 1;
    }
    let mut pump_delay = Duration::from_millis(pump_delay);
    if line_count > 500 && pump_delay < Duration::from_millis(100){
        pump_delay = Duration::from_millis(100);
    }
    let file = File::open(file_path).await?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    while let Some(line) = lines.next_line().await? {

        if line.len() > CHUNK_SIZE {
            for chunk in line.as_bytes().chunks(CHUNK_SIZE) {
                writer.write_all(format!("PRIVMSG {} :{}\r\n", channel, String::from_utf8_lossy(chunk)).as_bytes()).await?;
                writer.flush().await?;
                time::sleep(pump_delay).await;
            }
        } else {

            writer.write_all(format!("PRIVMSG {} :{}\r\n", channel, line).as_bytes()).await?;
            writer.flush().await?;
            time::sleep(pump_delay).await;
        }
    }
    Ok(())
}

fn select_random_file(dir: &str) -> Option<String> {
    let files = fs::read_dir(dir).ok()?.filter_map(|entry| {
        let path = entry.ok()?.path();
        if path.is_file() {
            path.to_str().map(ToString::to_string)
        } else {
            None
        }
    }).collect::<Vec<String>>();

    if files.is_empty() {
        None
    } else {
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..files.len());
        files.get(index).cloned()
    }
}

pub async fn handle_ascii_command<W: AsyncWriteExt + Unpin>(
    writer: &mut W,
    config: &Config, 
    command: &str,
    channel: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let parts: Vec<&str> = command.split_whitespace().collect();
    let command_type = parts.get(1).unwrap_or(&"");

    if *command_type == "random" && parts.len() == 2 {
        handle_random(writer, config, channel).await?;
    } else if *command_type == "list"{
        handle_list(writer, config, channel, Some(parts.get(2).unwrap_or(&""))).await?;
    } else {
        handle_specific_file(writer, config, channel, &parts).await?;
    }

    Ok(())
}

async fn handle_random<W: AsyncWriteExt + Unpin>(
    writer: &mut W,
    config: &Config,
    channel: &str,
) -> Result<(), Box<dyn Error>> {
    if let Some(dir) = config.ascii_art.as_ref() {
        if let Some(random_file) = select_random_file(dir) {
            send_ansi_art(writer, &random_file, config.pump_delay, channel).await?;
        } else {
            writer.write_all(format!("PRIVMSG {} :No files found\r\n", channel).as_bytes()).await?;
        }
    }
    Ok(())
}

async fn handle_list<W: AsyncWriteExt + Unpin>(
    writer: &mut W,
    config: &Config,
    channel: &str,
    parts: Option<&str>
) -> Result<(), Box<dyn Error>> {
    let base_dir = config.ascii_art.clone().unwrap_or_else(|| "ascii_art".to_string());

    let dir = if let Some(subdir) = parts {
        format!("{}/{}", base_dir, subdir)
    } else {
        base_dir
    };

    let entries = fs::read_dir(&dir)
        .map_err(|_| "Failed to read directory")?
        .filter_map(|entry| entry.ok())
        .map(|entry| {
            let path = entry.path();
            let display_name = path.file_name().unwrap_or_default().to_string_lossy().into_owned();
            if path.is_dir() {
                format!("{}/", display_name)
            } else {
                display_name.strip_suffix(".txt").unwrap_or(&display_name).to_string()
            }
        })
        .collect::<Vec<String>>()
        .join(", ");

    if entries.is_empty() {
        writer.write_all(format!("PRIVMSG {} :No files or directories found\r\n", channel).as_bytes()).await?;
    } else {
        writer.write_all(format!("PRIVMSG {} :{}\r\n", channel, entries).as_bytes()).await?;
    }

    Ok(())
}

async fn handle_specific_file<W: AsyncWriteExt + Unpin>(
    writer: &mut W,
    config: &Config,
    channel: &str,
    parts: &[&str],
) -> Result<(), Box<dyn Error>> {
    println!("{:?}", parts);
    let file_name = if parts.len() > 2 {
        parts[1..].join(" ").replace(' ', "/")
    } else {
        parts.get(1).unwrap_or(&"").to_string()
    };
    println!("{:?}", file_name);

    let file_path = format!("{}/{}.txt", config.ascii_art.clone().unwrap_or_else(|| "ascii_art".to_string()), file_name);
    println!("{:?}", file_path);
    
    send_ansi_art(writer, &file_path, config.pump_delay, channel).await
}


