// mods/handler.rs
use crate::{readmsg, writemsg, Config, MessageBuffer};
use tokio::io::{split, AsyncRead, AsyncWrite};
use tokio::sync::mpsc;

/// Handle the connection to the server
pub async fn handler<S>(stream: S, config: Config) -> Result<(), Box<dyn std::error::Error>>
where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    let (reader, writer) = split(stream);
    let (tx, rx) = mpsc::channel(1000);

    let read_task = tokio::spawn(async move {
        readmsg(reader, tx).await;
    });

    let message_buffer = MessageBuffer::new(1000);

    let write_task = tokio::spawn(async move {
        writemsg(writer, rx, &config, message_buffer).await;
    });

    //let _ = tokio::try_join!(read_task, write_task);
    tokio::try_join!(read_task, write_task)
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    Ok(())
}
