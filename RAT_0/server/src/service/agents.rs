use chrono::Utc;
use uuid::Uuid;

use super::Service;
use crate::{entities::Agent, error::Error, share::AgentRegistered};

impl Service {
    pub async fn register_agent(&self) -> Result<AgentRegistered, Error> {
        let id = Uuid::new_v4();
        let created_at = Utc::now();
        let agent = Agent {
            id,
            created_at,
            last_seen_at: created_at,
        };
        self.repo.create_agent(&self.db, &agent).await?;

        Ok(AgentRegistered { id })
    }

    pub async fn list_agents(&self) -> Result<Vec<Agent>, Error> {
        self.repo.find_all_agents(&self.db).await
    }
}
