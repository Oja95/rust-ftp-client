use crate::client::{data, Runtime};
use std::error::Error;
use log::info;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream;

pub async fn send_command_tls(stream: &mut TlsStream<TcpStream>, command: &str) -> Result<String, Box<dyn Error>> {
    stream.write_all(command.as_bytes()).await?;
    stream.write_all(b"\r\n").await?;
    stream.flush().await.unwrap();

    let mut reader = BufReader::new(stream);
    let mut response = String::new();
    reader.read_line(&mut response).await?;

    info!("Server response: {}", response);
    Ok(response)
}

pub async fn send_command_plain(stream: &mut TcpStream, command: &str) -> Result<String, Box<dyn Error>> {
    stream.write_all(command.as_bytes()).await?;
    stream.write_all(b"\r\n").await?;
    stream.flush().await?;

    let mut reader = BufReader::new(stream);
    let mut response = String::new();
    reader.read_line(&mut response).await?;
    info!("Server response: {}", response);
    Ok(response)
}

pub async fn eval_command(runtime: &mut Runtime, command: &str) -> Result<String, Box<dyn Error>> {
    let cmd = command.split_whitespace().nth(0).unwrap();
    match cmd.to_lowercase().as_str() {
        "list" => data::handle_list(runtime).await,
        "retr" => data::handle_retr(runtime, command).await,
        "epsv" => data::handle_epsv(runtime).await,
        &_ => runtime.send_command(command).await
    }
}