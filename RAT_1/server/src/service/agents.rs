use chrono::Utc;
use ed25519_dalek;
use uuid::Uuid;

use super::Service;
use crate::share::{AgentRegister, AgentRegistered};
use crate::{entities::Agent, error::Error};

impl Service {
    pub async fn register_agent(&self, register: AgentRegister) -> Result<AgentRegistered, Error> {
        let id = Uuid::new_v4();
        let created_at = Utc::now();

        // verify register
        if register.public_prekey_signature.len() != 64 {
            //ED25519_SIGNATURE_SIZE 64
            return Err(Error::InvalidArgument(
                "Agent's public prekey Signature size is not valid".to_string(),
            ));
        }

        let agent_signing_public_key =
            ed25519_dalek::VerifyingKey::from_bytes(&register.singing_public_key)?;
        let signature =
            ed25519_dalek::Signature::try_from(&register.public_prekey_signature[0..64])?;

        log::debug!("register_agent: register is valid");

        if agent_signing_public_key
            .verify_strict(&register.public_prekey, &signature)
            .is_err()
        {
            return Err(Error::InvalidArgument("Signature is not valid".to_string()));
        }

        log::debug!("register_agent: agent's public_prekey signature verified");

        let agent = Agent {
            id,
            created_at,
            last_seen_at: created_at,
            signing_public_key: register.singing_public_key.to_vec(),
            public_prekey: register.public_prekey.to_vec(),
            public_prekey_signature: register.public_prekey_signature.to_vec(),
        };
        self.repo.create_agent(&self.db, &agent).await?;

        Ok(AgentRegistered { id })
    }

    pub async fn list_agents(&self) -> Result<Vec<Agent>, Error> {
        self.repo.find_all_agents(&self.db).await
    }

    pub async fn get_agent(&self, agent_id: Uuid) -> Result<Agent, Error> {
        self.repo.find_agent_by_id(&self.db, agent_id).await
    }
}
