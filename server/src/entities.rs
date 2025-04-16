use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
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

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Agent {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub signing_public_key: Vec<u8>,
    pub public_prekey: Vec<u8>,
    pub public_prekey_signature: Vec<u8>,
}
