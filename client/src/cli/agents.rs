use comfy_table::{Table, Cell, Color};
use crate::{error::Error, api::Api};

pub async fn list_agents(api: &Api) -> Result<(), Error> {
    let agents = api.get_list_agents().await?;

    let mut table = Table::new();

    table.set_header(vec!["Agent ID", "Created At", "Last Seen At"]);

    for agent in agents {
        table.add_row(vec![
            Cell::new(agent.id.to_string().as_str()).fg(Color::Cyan),
            Cell::new(agent.created_at.to_string().as_str()),
            Cell::new(agent.last_seen_at.to_string().as_str()),
        ]);
    }

    print!("{table}");

    Ok(())
}