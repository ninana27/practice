use crate::error::Error;
use super::{Api, Agent, Response};

impl Api {
    pub async fn get_list_agents(&self) -> Result<Vec<Agent>, Error>{
        let list_agents_url = format!("{}/api/agents", self.server_url);
        let resp = self.client
            .get(list_agents_url)
            .send()
            .await?
            .json::<Response<Vec<Agent>>>()
            .await?;

        let agents = resp.data.unwrap();
        
        Ok(agents)
    }
}
