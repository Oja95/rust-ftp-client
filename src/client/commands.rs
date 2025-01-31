use crate::client::{data, Runtime};
use std::error::Error;
use log::info;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};

pub async fn send_command_internal<S>(stream: &mut S, command: &str) -> Result<String, Box<dyn Error>>
where
    S: AsyncReadExt + AsyncWriteExt + Unpin,
{
    stream.write_all(command.as_bytes()).await?;
    stream.write_all(b"\r\n").await?;
    stream.flush().await?;

    let mut reader = BufReader::new(stream);
    let mut response = String::new();
    let mut buffer = [0; 1024];

    loop {
        let bytes_read = reader.read(&mut buffer).await?;
        if bytes_read == 0 {
            break;
        }

        let chunk = String::from_utf8_lossy(&buffer[..bytes_read]);
        response.push_str(&chunk);

        if chunk.ends_with("\r\n") || chunk.contains("226") {
            break;
        }
    }

    info!("Server response: {}", response);
    Ok(response)
}

pub async fn eval_command(runtime: &mut Runtime, command: &str) -> Result<String, Box<dyn Error>> {
    let cmd = command.split_whitespace().nth(0).unwrap();
    match cmd.to_lowercase().as_str() {
        "list" => data::handle_list(runtime, command).await,
        "retr" => data::handle_retr(runtime, command).await,
        "epsv" => data::handle_epsv(runtime).await,
        &_ => runtime.send_command(command).await
    }
}