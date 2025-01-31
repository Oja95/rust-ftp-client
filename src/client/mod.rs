mod data;
mod commands;

use crate::config::Config;
use crate::tls;
use rustls::pki_types::ServerName;
use rustls::ClientConfig;
use std::error::Error;
use std::io;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream;
use tokio_rustls::TlsConnector;
use log::{error, info};

pub struct Runtime {
    pub tls_config: Option<ClientConfig>,
    data_port: Option<usize>,
    control_channel_tls_stream: Option<TlsStream<TcpStream>>,
    control_channel_tcp_stream: Option<TcpStream>,
}

impl Runtime {
    fn new() -> Self {
        Runtime {
            tls_config: None,
            data_port: None,
            control_channel_tls_stream: None,
            control_channel_tcp_stream: None,
        }
    }

    async fn connect(&mut self, config: &Config) -> Result<(), Box<dyn Error>> {
        info!("Connecting to FTP server: {}", config.server);

        let mut tcp_stream = TcpStream::connect(format!("{}:{}", config.server, config.port)).await?;
        let mut reader = BufReader::new(&mut tcp_stream);
        let mut response = String::new();
        reader.read_line(&mut response).await?;

        self.control_channel_tcp_stream = Some(tcp_stream);
        info!("Connect response: {}", response);

        if config.use_tls {
            self.send_command("AUTH TLS").await?;

            tls::configure_tls(self).await;
            let connector = TlsConnector::from(Arc::new(self.tls_config.clone().unwrap()));

            let tcp_stream = self.control_channel_tcp_stream.take().unwrap();
            let tls_stream = connector.connect(ServerName::try_from(config.server.clone()).unwrap(), tcp_stream).await?;
            self.control_channel_tls_stream = Some(tls_stream);

            self.send_command("PROT P").await?;
        }

        Ok(())
    }

    pub async fn send_command(&mut self, command: &str) -> Result<String, Box<dyn Error>> {
        info!("Sending command: {}", command);
        if self.control_channel_tls_stream.is_some() {
            commands::send_command_internal(self.control_channel_tls_stream.as_mut().unwrap(), command).await
        } else {
            commands::send_command_internal(self.control_channel_tcp_stream.as_mut().unwrap(), command).await
        }
    }

    pub async fn read_control_channel_line(&mut self) -> Result<String, Box<dyn Error>> {
        let mut response = String::new();
        if let Some(tls_stream) = self.control_channel_tls_stream.as_mut() {
            let mut reader = BufReader::new(tls_stream);
            reader.read_line(&mut response).await?;
        } else {
            let mut reader = BufReader::new(self.control_channel_tcp_stream.as_mut().unwrap());
            reader.read_line(&mut response).await?;
        }

        Ok(response)
    }
}

pub struct FtpClient {
    config: Config,
    runtime: Runtime
}

impl FtpClient {
    pub(crate) fn new(config: Config) -> Self {
        FtpClient {
            config,
            runtime: Runtime::new()
        }
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        self.runtime.connect(&self.config).await?;
        self.attempt_login().await?;

        loop {
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            let input = input.trim();

            if input.eq_ignore_ascii_case("exit") {
                info!("Exiting!");
                return Ok(());
            }

            match commands::eval_command(&mut self.runtime, input).await {
                Ok(result) => info!("Result: {}", result),
                Err(err) => error!("Error: {}", err),
            }
        }
    }

    async fn attempt_login(&mut self) -> Result<(), Box<dyn Error>> {
        if !self.config.username.is_empty() {
            self.runtime.send_command(format!("USER {}", self.config.username).as_str()).await?;
            self.runtime.send_command(format!("PASS {}", self.config.password).as_str()).await?;
        }

        Ok(())
    }

}
