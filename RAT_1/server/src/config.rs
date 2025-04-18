use crate::error::Error;
use base64::{engine::general_purpose, Engine as _};

pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub client_signing_public_key: ed25519_dalek::VerifyingKey,
}

const ENV_DATABASE_URL: &str = "DATABASE_URL";
const ENV_PORT: &str = "PORT";
const ENV_CLIENT_SIGNING_PUBLIC_KEY: &str = "CLIENT_SIGNING_PUBLIC_KEY";

const DEFAULT_PORT: u16 = 8080;

impl Config {
    pub fn load() -> Result<Config, Error> {
        dotenv::dotenv().ok();

        let port = std::env::var(ENV_PORT)
            .ok()
            .map_or(Ok(DEFAULT_PORT), |env_var| env_var.parse::<u16>())?;

        let database_url =
            std::env::var(ENV_DATABASE_URL).map_err(|_| env_not_found(ENV_DATABASE_URL))?;

        let client_signing_public_key_str = std::env::var(ENV_CLIENT_SIGNING_PUBLIC_KEY)
            .ok()
            .unwrap_or(String::new());

        let client_signing_public_key_bytes = general_purpose::STANDARD
            .decode(&client_signing_public_key_str)
            .map_err(|err| Error::Internal(err.to_string()))?;

        let client_signing_public_key_bytes_arry: [u8; 32] = client_signing_public_key_bytes
            .try_into()
            .expect("not valid");

        let client_signing_public_key: ed25519_dalek::VerifyingKey =
            ed25519_dalek::VerifyingKey::from_bytes(&client_signing_public_key_bytes_arry)?;

        Ok(Config {
            port,
            database_url,
            client_signing_public_key,
        })
    }
}

fn env_not_found(var: &str) -> Error {
    Error::NotFound(format!("Config: {} env var not found", var))
}
