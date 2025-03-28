use reqwest;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use std::{path, fs};

use crate::{config, error::Error};


#[derive(Debug, Serialize, Deserialize)]
pub struct Response<T: Serialize> {
    pub data: Option<T>,
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
pub struct AgentRegistered {
    pub id: Uuid,
}

pub async fn get_id() -> Result<Uuid, Error> {
    let get_saved_id = read_saved_id()?;
    println!("get_saved_id: {:?}", get_saved_id);
    let agent_id = match get_saved_id {
        Some(agent_id) => agent_id,
        None => {
            let agent_id = registe_id().await?;
            save_id(agent_id)?;
            agent_id
        }
    };

    Ok(agent_id)
}

async fn registe_id() -> Result<Uuid, Error>{

    let url = format!("{}/api/agents", config::SERVER_URL);
    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .send()
        .await?
        .json::<Response<AgentRegistered>>()
        .await?;    

    let agent_id = match resp.data {
        Some(agent) => Ok(agent.id),
        None => Err(Error::Internal("date is null".to_string())),
    }?;

    Ok(agent_id)
}

fn save_id(agent_id: Uuid) -> Result<(), Error> {
    let save_file_path = config::ID_FILE_PATH;
    fs::write(save_file_path, agent_id.as_bytes())?;
    Ok(())
}

fn read_saved_id() -> Result<Option<Uuid>, Error> {
    let saved_file_path = path::Path::new(config::ID_FILE_PATH);
    if saved_file_path.exists() {
        let agent_file_content = fs::read(saved_file_path)?;
        let agent_id = Uuid::from_slice(&agent_file_content)?;
        Ok(Some(agent_id))
    }
    else {
        Ok(None)
    }
}
