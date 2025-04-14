use crate::error::Error;
use base64::{engine::general_purpose, Engine as _};
use ed25519_dalek;

pub const SERVER_URL: &str = "http://192.168.10.3:8080";

//private key: cmVRqjUjp18o7yT5xwJVZQnA6wc4kMkfj4N6UGXchEI=
//public key: VHaq+MHUCpzStns+lwGeeyYamyOD7E3Z0562RCtp/68=
pub const SIGNING_PRIVATE_KEY: &str = "cmVRqjUjp18o7yT5xwJVZQnA6wc4kMkfj4N6UGXchEI=";

#[derive(Debug)]
pub struct Config {
    pub signing_public_key: ed25519_dalek::VerifyingKey,
    pub signing_private_key: ed25519_dalek::SecretKey,
}

impl Config {
    pub fn load() -> Result<Config, Error> {
        let signing_private_key_decode = general_purpose::STANDARD.decode(SIGNING_PRIVATE_KEY)?;
        let signing_private_key: ed25519_dalek::SecretKey =
            ed25519_dalek::SecretKey::try_from(signing_private_key_decode).unwrap();
        let signingkey: ed25519_dalek::SigningKey = (&signing_private_key).into();
        let signing_public_key: ed25519_dalek::VerifyingKey = signingkey.verifying_key();

        Ok(Config {
            signing_private_key,
            signing_public_key,
        })
    }
}
