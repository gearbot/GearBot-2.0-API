use crate::error::StartupError;
use serde::Deserialize;
use std::fs;

#[derive(Deserialize, Debug)]
pub struct ApiConfig {
    pub redis: String,
    pub port: u16,
    pub application_id: u64,
    pub client_secret: String,
    pub redirect_uri: String,
    pub domain: String,
    pub secure: bool
}

impl ApiConfig {
    pub fn new(filename: &str) -> Result<Self, StartupError> {
        let config_file = fs::read_to_string(filename).map_err(|_| StartupError::NoConfig)?;
        toml::from_str::<ApiConfig>(&config_file).map_err(|_| StartupError::InvalidConfig)
    }
}
