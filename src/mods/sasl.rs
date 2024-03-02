// mods/sasl.rs
use base64::Engine;
use tokio::io::AsyncWriteExt;
/// Sends the initial commands to negotiate capabilities and start SASL authentication.
pub async fn start_sasl_auth<W: tokio::io::AsyncWriteExt + Unpin>(
//pub async fn start_sasl_auth(...) -> Result<(), Box<dyn std::error::Error>> {
    writer: &mut W,
    mechanism: &str,
    nickname: &str,
    capabilities: &[String], // Add a parameter for capabilities
) -> Result<(), Box<dyn std::error::Error>> {
    // Request a list of capabilities from the server
    writer.write_all(b"CAP LS 302\r\n").await?;

    // Send NICK and USER commands
    let nick_cmd = format!("NICK {}\r\n", nickname);
    writer.write_all(nick_cmd.as_bytes()).await?;
    let user_cmd = format!("USER {} 0 * :{}\r\n", nickname, nickname);
    writer.write_all(user_cmd.as_bytes()).await?;

    // Request specific capabilities, including 'sasl' for SASL authentication
    if !capabilities.is_empty() {
        let cap_req_cmd = format!("CAP REQ :{}\r\n", capabilities.join(" "));
        writer.write_all(cap_req_cmd.as_bytes()).await?;
    } else {
        // If no specific capabilities are requested, directly request 'sasl'
        writer.write_all(b"CAP REQ :sasl\r\n").await?;
    }

    writer.flush().await?;
    Ok(())
}

/// Continues the SASL authentication process based on the server's responses.
//pub async fn handle_sasl_messages(...) -> Result<(), Box<dyn std::error::Error>> {
pub async fn handle_sasl_messages<W: tokio::io::AsyncWriteExt + Unpin>(
    writer: &mut W,
    message: &str,
    username: &str,
    password: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if message.contains("CAP * ACK :sasl") {
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

