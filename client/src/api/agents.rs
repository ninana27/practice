use uuid::Uuid;

use super::{Agent, Api, Response};
use crate::error::Error;

impl Api {
    pub async fn get_list_agents(&self) -> Result<Vec<Agent>, Error> {
        let list_agents_url = format!("{}/api/agents", self.server_url);
        let resp = self
            .client
            .get(list_agents_url)
            .send()
            .await?
            .json::<Response<Vec<Agent>>>()
            .await?;

        let agents = resp.data.unwrap();

        Ok(agents)
    }

    pub async fn get_agent(&self, agent_id: Uuid) -> Result<Agent, Error> {
        let get_agent_url = format!("{}/api/agents/{}", self.server_url, agent_id);
        let resp = self
            .client
            .get(get_agent_url)
            .send()
            .await?
            .json::<Response<Agent>>()
            .await?;

        let agents = resp.data.unwrap();
        Ok(agents)
    }
}
