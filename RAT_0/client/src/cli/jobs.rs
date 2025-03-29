use comfy_table::{Table, Cell, Color};
use crate::{error::Error, api::Api};

pub async fn list_jobs(api: &Api) -> Result<(), Error> {
    let jobs = api.get_list_jobs().await?;

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

    for agent in jobs {
        table.add_row(vec![
            Cell::new(agent.id.to_string().as_str()).fg(Color::Green),
            Cell::new(agent.created_at.to_string().as_str()),
            Cell::new(
                agent.executed_at
                    .map(|t| t.to_string())
                    .unwrap_or("".to_string())
                    .as_str()
            ),
            Cell::new(agent.command.as_str()).fg(Color::Yellow),
            Cell::new(agent.args.join(" ").as_str()),
            Cell::new(agent.output.unwrap_or("".to_string()).as_str()),
            Cell::new(agent.agent_id.to_string().as_str()).fg(Color::Cyan),
        ]);
    }

    print!("{table}");

    Ok(())
}