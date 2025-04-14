use reqwest::{self, Client};
use serde::{Deserialize, Serialize};
use std::{fs, path};
use uuid::Uuid;

use base64::{engine::general_purpose, Engine as _};
use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use rand::{rngs::OsRng, RngCore};
use x25519_dalek::{x25519, X25519_BASEPOINT_BYTES};

use crate::{
    config::{self, Config, SerializedConfig},
    error::Error,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Response<T: Serialize> {
    pub data: Option<T>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AgentRegistered {
    pub id: Uuid,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AgentRegister {
    pub singing_public_key: [u8; 32],
    pub public_prekey: [u8; 32],
    pub public_prekey_signature: Vec<u8>,
}

pub async fn get_config(client: &Client) -> Result<Config, Error> {
    let get_saved_config = read_saved_config()?;
    println!("get_saved_config: {:?}", get_saved_config);
    let agent_confog = match get_saved_config {
        Some(agent_id) => agent_id,
        None => {
            let config = register_config(&client).await?;
            save_config(&config)?;
            config
        }
    };

    Ok(agent_confog)
}

async fn register_config(client: &Client) -> Result<Config, Error> {
    let url = format!("{}/api/agents", config::SERVER_URL);

    let mut csprng = OsRng;
    // generate signing keypair
    let signing_keypair: SigningKey = SigningKey::generate(&mut csprng);

    // generate prekey
    let mut private_prekey = [0u8; 32];
    csprng.fill_bytes(&mut private_prekey);
    let public_prekey = x25519(private_prekey.clone(), X25519_BASEPOINT_BYTES);

    // signing public prekey
    let public_prekey_signature = signing_keypair.sign(&public_prekey);

    // register
    let register_agent = AgentRegister {
        singing_public_key: signing_keypair.verifying_key().to_bytes(),
        public_prekey: public_prekey.clone(),
        public_prekey_signature: public_prekey_signature.to_bytes().to_vec(),
    };

    let resp = client
        .post(&url)
        .json(&register_agent)
        .send()
        .await?
        .json::<Response<AgentRegistered>>()
        .await?;

    let agent_id = match resp.data {
        Some(agent) => Ok(agent.id),
        None => Err(Error::Internal("date is null".to_string())),
    }?;

    let client_signing_public_bytes: Vec<u8> = general_purpose::STANDARD
        .decode(config::CLIENT_IDENTITY_PUBLIC_KEY)
        .unwrap();
    let client_signing_public_bytes_arry: [u8; 32] =
        client_signing_public_bytes.try_into().unwrap();
    let client_signing_public_key: VerifyingKey =
        ed25519_dalek::VerifyingKey::from_bytes(&client_signing_public_bytes_arry)?;

    // config
    let config = Config {
        agent_id,
        signing_public_key: signing_keypair.verifying_key(),
        signing_private_key: signing_keypair.to_bytes(),
        public_prekey,
        private_prekey,
        client_signing_public_key,
    };

    Ok(config)
}

fn save_config(config: &Config) -> Result<(), Error> {
    let save_file_path = config::ID_FILE_PATH;

    let serialized_config: SerializedConfig = config.into();
    let config_json = serde_json::to_string(&serialized_config)?;
    fs::write(save_file_path, config_json.as_bytes())?;
    Ok(())
}

fn read_saved_config() -> Result<Option<Config>, Error> {
    let saved_file_path = path::Path::new(config::ID_FILE_PATH);
    if saved_file_path.exists() {
        let agent_file_content = fs::read(saved_file_path)?;
        let serialized_conf: config::SerializedConfig =
            serde_json::from_slice(&agent_file_content)?;
        let agent_config: Config = serialized_conf.try_into()?;
        Ok(Some(agent_config))
    } else {
        Ok(None)
    }
}
