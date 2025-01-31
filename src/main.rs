extern crate dotenv_codegen;

use dotenv::dotenv;
use env_logger::{Builder, Target};

mod client;
mod config;
mod tls;

#[tokio::main]
async fn main() {
    dotenv().ok();

    Builder::from_default_env()
        .target(Target::Stdout)
        .init();

    let config = config::load_config().expect("Failed to load configuration");
    client::FtpClient::new(config)
        .run().await.expect("FTP client crashed!");
}