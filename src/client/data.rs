use crate::client::Runtime;
use regex::Regex;
use rustls::pki_types::ServerName;
use std::sync::Arc;
use log::info;
use tokio::io::{AsyncReadExt, BufReader};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;

pub async fn handle_list(runtime: &mut Runtime, command: &str) ->  Result<String, Box<dyn std::error::Error>> {
    let result = runtime.send_command(command).await?;
    info!("LIST command server response: {}", result);

    let data_stream = TcpStream::connect(format!("localhost:{}", runtime.data_port.unwrap())).await?;
    let connector = TlsConnector::from(Arc::new(runtime.tls_config.clone().unwrap()));
    let mut tls_stream = connector.connect(ServerName::try_from("localhost").unwrap(), data_stream).await?;

    let mut reader = BufReader::new(&mut tls_stream);
    let mut list_data = Vec::new();
    reader.read_to_end(&mut list_data).await?;
    let file_listing_string = String::from_utf8(list_data).unwrap();
    info!("{:?}", file_listing_string);

    // Read control channel response about success of listing transfer
    let response = runtime.read_control_channel_line().await?;
    Ok(response)
}

pub async fn handle_retr(runtime: &mut Runtime, command: &str) ->  Result<String, Box<dyn std::error::Error>> {
    let result = runtime.send_command(command).await?;
    info!("RETR commands server response: {}", result);
    let filename = command.split_whitespace().nth(1).unwrap();

    let stream = TcpStream::connect(format!("localhost:{}", runtime.data_port.unwrap())).await?;
    let connector = TlsConnector::from(Arc::new(runtime.tls_config.clone().unwrap()));
    let mut tls_stream = connector.connect(ServerName::try_from("localhost").unwrap(), stream).await?;

    let mut reader = BufReader::new(&mut tls_stream);
    let mut file_data = Vec::new();
    reader.read_to_end(&mut file_data).await?;

    tokio::fs::write(filename, &file_data).await?;

    let response = runtime.read_control_channel_line().await?;
    Ok(response)
}

// Configures passive mode for data channel. Making client responsible for creating the data channel connection.
pub async fn handle_epsv(runtime: &mut Runtime) -> Result<String, Box<dyn std::error::Error>> {
    let result = runtime.send_command("EPSV").await?;

    // Expected response: 229 Entering Extended Passive Mode (|||53897|)
    let regex = Regex::new(r".*\|(\d+)\|.*").unwrap();
    let port = regex.captures(&result).unwrap()[1].parse::<usize>().unwrap();
    info!("New data channel port set: {}", port);
    runtime.data_port = Some(port);

    Ok(result)
}
