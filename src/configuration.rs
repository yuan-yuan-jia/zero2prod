use std::{collections::HashMap, env::current_dir};

use config::Environment;
use secrecy::{ExposeSecret, Secret};

#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

#[derive(serde::Deserialize)]
pub struct ApplicationSettings {
    pub port: u16,
    pub host: String,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.database_name
        ))
    }

    pub fn connection_string_without_db(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port
        ))
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
   let base_path = std::env::current_dir()
   .expect("Failed to determine the current directory");
let configuration_directory = base_path.join("configuration");
    // Detect the running enviroment
    // Default to `local` if unspecified
    let environment = std::env::var("APP_ENVIRONMEN").unwrap_or_else(|_| "local".to_string());

    let environment_filename = format!("{}.yaml", environment);
    let settings = config::Config::builder()
        .add_source(config::File::from(
            configuration_directory.join("base.yaml")
        ))
        .add_source(config::File::from(
            configuration_directory.join(environment_filename)
        ))
        .build()?;

    settings.try_deserialize::<Settings>()
}
