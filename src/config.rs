use std::env;
use dotenv::dotenv;
use dotenv_codegen::dotenv;

pub struct Config {
    pub server: String,
    pub port: usize,
    pub username: String,
    pub password: String,
    pub use_tls: bool,
}

pub fn load_config() -> Result<Config, std::io::Error> {
    dotenv().ok();
    let port = env::var("SERVER_CONTROL_CHANNEL_PORT")
        .map_or_else(
            |_| {
                eprintln!("Environment variable not set, defaulting to 21");
                "21".to_string()
            },
            |s| s,
        )
        .parse::<usize>()
        .unwrap_or_else(|_| {
            eprintln!("Invalid port in .env, defaulting to 21");
            21
        });
    Ok(Config {
        server: dotenv!("SERVER_URL").to_string(),
        port,
        use_tls: parse_bool(dotenv!("USE_TLS")),
        username: get_env_variable_or_default("FTP_USERNAME", ""),
        password: get_env_variable_or_default("PASSWORD", "")
    })
}

fn get_env_variable_or_default(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| {
        eprintln!("{} is not set. Defaulting to {}", key, default);
        default.to_string()
    })
}

fn parse_bool(s: &str) -> bool {
    matches!(s.to_lowercase().as_str(), "true" | "1" | "yes" | "on")
}

