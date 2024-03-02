// mods/sasl.rs
use base64::Engine;
pub async fn start_sasl_auth<W: tokio::io::AsyncWriteExt + Unpin>(
    writer: &mut W,
    mechanism: &str,
    nickname: &str,
    capabilities: Option<Vec<String>>) -> Result<(), Box<dyn std::error::Error>> {
    writer.write_all(b"CAP LS 302\r\n").await?;

    let nick_cmd = format!("NICK {}\r\n", nickname);
    writer.write_all(nick_cmd.as_bytes()).await?;
    let user_cmd = format!("USER {} 0 * :{}\r\n", nickname, nickname);
    writer.write_all(user_cmd.as_bytes()).await?;

    if let Some(caps) = capabilities {
        if !caps.is_empty() {
            let cap_req_cmd = format!("CAP REQ :{}\r\n", caps.join(" "));
            writer.write_all(cap_req_cmd.as_bytes()).await?;
        }
    } else {
        writer.write_all(b"CAP REQ :sasl\r\n").await?;
    }

    writer.flush().await?;
    Ok(())
}

pub async fn handle_sasl_messages<W: tokio::io::AsyncWriteExt + Unpin>(
    writer: &mut W,
    message: &str,
    username: &str,
    password: &str,
    nickname: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let nick = format!("CAP {} ACK :sasl", nickname.to_string());
    if message.contains(&nick) {
        writer.write_all(b"AUTHENTICATE PLAIN\r\n").await?;
    } else if message.starts_with("AUTHENTICATE +") {
        let auth_string = format!("\0{}\0{}", username, password);
        let encoded = base64::engine::general_purpose::STANDARD.encode(auth_string);
        writer.write_all(format!("AUTHENTICATE {}\r\n", encoded).as_bytes()).await?;
    } else if message.contains("903 * :SASL authentication successful") {
        writer.write_all(b"CAP END\r\n").await?;
    }
    writer.flush().await?;
    Ok(())
}

