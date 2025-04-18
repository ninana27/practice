use crate::{api::Api, error::Error};
use comfy_table::{Cell, Color, Table};

pub async fn list_jobs(api: &Api) -> Result<(), Error> {
    let jobs = api.get_list_jobs().await?;

    let mut table = Table::new();

    table.set_header(vec!["Job ID", "Agent ID"]);

    for agent in jobs {
        table.add_row(vec![
            Cell::new(agent.id.to_string().as_str()).fg(Color::Green),
            Cell::new(agent.agent_id.to_string().as_str()).fg(Color::Cyan),
        ]);
    }

    print!("{table}");

    Ok(())
}
