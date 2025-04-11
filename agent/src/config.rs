use std::convert::{Into, TryFrom};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use ed25519_dalek::VerifyingKey;
use x25519_dalek::{x25519, X25519_BASEPOINT_BYTES};
use base64::{Engine as _, engine::general_purpose};


use crate::error::Error;

pub const SERVER_URL: &str = "http://192.168.10.3:8080";
pub const ID_FILE_PATH: &str = "./id";
pub const CLIENT_IDENTITY_PUBLIC_KEY: &str = "xQ6gstFLtTbDC06LDb5dAQap+fXVG45BnRZj0L5th+M=";

#[derive(Debug)]
pub struct Config {
    pub agent_id: Uuid,
    pub signing_public_key: ed25519_dalek::VerifyingKey,
    pub signing_private_key: ed25519_dalek::SecretKey,
    pub public_prekey: [u8; 32],
    pub private_prekey: [u8; 32],
    pub client_signing_public_key: ed25519_dalek::VerifyingKey,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SerializedConfig {
    pub agent_id: Uuid,
    pub signing_private_key: [u8; ed25519_dalek::SECRET_KEY_LENGTH],
    pub private_prekey: [u8; 32],
}

impl TryFrom<SerializedConfig> for Config {
    type Error = Error;

    fn try_from(conf: SerializedConfig) -> Result<Config, Self::Error> {
        let agent_id = conf.agent_id;

        let signing_private_key: ed25519_dalek::SecretKey = 
            ed25519_dalek::SecretKey::try_from(conf.signing_private_key).unwrap();
        let signingkey: ed25519_dalek::SigningKey = (&signing_private_key).into();
        let signing_public_key: ed25519_dalek::VerifyingKey = signingkey.verifying_key();


        let private_prekey = conf.private_prekey;
        let public_prekey = x25519(private_prekey.clone(), X25519_BASEPOINT_BYTES);

        let client_signing_public_bytes: Vec<u8> = general_purpose::STANDARD
            .decode(CLIENT_IDENTITY_PUBLIC_KEY).unwrap();
        let client_signing_public_bytes_arry: [u8; 32] = client_signing_public_bytes.try_into().unwrap();
        let client_signing_public_key: VerifyingKey = ed25519_dalek::VerifyingKey::from_bytes(&client_signing_public_bytes_arry)?;

        Ok(Config {
            agent_id,
            signing_public_key,
            signing_private_key,
            public_prekey,
            private_prekey,
            client_signing_public_key,
        })
    }
}

impl Into<SerializedConfig> for &Config {
    fn into(self) -> SerializedConfig {
        SerializedConfig {
            agent_id: self.agent_id,
            signing_private_key: self.signing_private_key,
            private_prekey: self.private_prekey,
        }
    }
}

