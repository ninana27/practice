use chrono::{DateTime, Utc};
use reqwest::{redirect, Client};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

mod agents;
mod jobs;

pub struct Api {
    client: Client,
    server_url: String,
}

impl Api {
    pub fn new(server_url: String) -> Api {
        let timeout = Duration::from_secs(5);
        let client_builder = reqwest::Client::builder();
        let client = client_builder
            .connect_timeout(timeout)
            .redirect(redirect::Policy::limited(4))
            .timeout(timeout)
            .build()
            .expect("Building HTTP client");

        Api { client, server_url }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response<T: Serialize> {
    pub data: Option<T>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Agent {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub signing_public_key: Vec<u8>,
    pub public_prekey: Vec<u8>,
    pub public_prekey_signature: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: Uuid,
    pub agent_id: Uuid,
    pub encrypted_job: Vec<u8>,
    pub ephemeral_public_key: Vec<u8>,
    pub nonce: Vec<u8>,
    pub signature: Vec<u8>,
    pub encrypted_result: Option<Vec<u8>>,
    pub result_ephemeral_public_key: Option<Vec<u8>>,
    pub result_nonce: Option<Vec<u8>>,
    pub result_signature: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateJob {
    pub id: Uuid,
    pub agent_id: Uuid,
    pub encrypted_job: Vec<u8>,
    pub ephemeral_public_key: [u8; 32], // X25519_PUBLIC_KEY_SIZE 32
    pub nonce: [u8; 24],                // XCHACHA20_POLY1305_NONCE_SIZE 24
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JobResult {
    pub output: String,
}
