use std::time::Duration;
use reqwest::{Client, redirect};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
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

        Api { 
            client, 
            server_url, 
        }
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Job {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub executed_at: Option<DateTime<Utc>>,
    pub command: String,
    pub args: Vec<String>,
    pub output: Option<String>,
    pub agent_id: Uuid,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateJob {
    pub agent_id: Uuid,
    pub command: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JobResult {
    pub executed_time: String,
    pub output: String,
}
