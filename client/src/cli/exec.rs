use std::{thread::sleep, time};

use uuid::Uuid;
use comfy_table::{Table, Cell, Color};
use crate::{api::{Api, CreateJob}, error::Error};

pub async fn agent_exec(api: &Api, agent_id: &String, command: &String) -> Result<(), Error> {
    let agent_id = Uuid::parse_str(agent_id.as_str())?;
    let createjob = CreateJob {
        agent_id,
        command: command.to_string(),
    };
    
    let job_info = api.post_create_job(createjob).await?;
    let mut table = Table::new();

    table.set_header(vec![
        "Job ID", 
        "Created At", 
        "Executed At", 
        "Command", 
        "Args",
        "Output",
        "Agent ID",
    ]);

    table.add_row(vec![
        Cell::new(job_info.id.to_string().as_str()).fg(Color::Green),
        Cell::new(job_info.created_at.to_string().as_str()),
        Cell::new(
            job_info.executed_at
                .map(|t| t.to_string())
                .unwrap_or("".to_string())
                .as_str()
        ),
        Cell::new(job_info.command.as_str()).fg(Color::Yellow),
        Cell::new(job_info.args.join(" ").as_str()),
        Cell::new(job_info.output.unwrap_or("".to_string()).as_str()),
        Cell::new(job_info.agent_id.to_string().as_str()).fg(Color::Cyan),
    ]);

    println!("{table}");
    sleep(time::Duration::from_secs(3));
    match api.get_job_result(job_info.id).await {
        Ok(job_result) => {
            println!("executed at: {}", job_result.executed_time);
            println!("result:\n{}", job_result.output);
        },
        Err(err) => { println!("{}", err.to_string()) },
    };
    
    Ok(())
}