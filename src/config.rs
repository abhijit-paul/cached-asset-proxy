use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_origin")]
    pub allow_origins: String,
    #[serde(default = "default_assets_url")]
    pub assets_url: String,
    #[serde(default = "default_redis_host")]
    pub redis_host: String,
    #[serde(default = "default_redis_port")]
    pub redis_port: u16,
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    3000
}

fn default_origin() -> String {
    "https://example.com".to_string()
}

fn default_assets_url() -> String {
    "http://localhost:5000".to_string()
}

fn default_redis_host() -> String {
    "http://localhost".to_string()
}

fn default_redis_port() -> u16 {
    6379
}

impl Config {
    pub fn new() -> Result<Self, config::ConfigError> {
        dotenv::dotenv().ok();

        let mut s = config::Config::default();

        s.merge(config::Environment::default().separator(""))?;

        s.try_into()
    }
}
