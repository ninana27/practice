use base64::{engine::general_purpose, Engine as _};
use comfy_table::{Cell, Color, Table};

use crate::{api::Api, error::Error};

pub async fn list_agents(api: &Api) -> Result<(), Error> {
    let agents = api.get_list_agents().await?;

    let mut table = Table::new();

    table.set_header(vec![
        "Agent ID",
        "Created At",
        "Last Seen At",
        "Signing Public Key",
        "Public Prekey",
    ]);

    for agent in agents {
        let signing_public_key_base64 = general_purpose::STANDARD.encode(agent.signing_public_key);
        let public_prekey_base64 = general_purpose::STANDARD.encode(agent.public_prekey);
        table.add_row(vec![
            Cell::new(agent.id.to_string().as_str()).fg(Color::Cyan),
            Cell::new(agent.created_at.to_string().as_str()),
            Cell::new(agent.last_seen_at.to_string().as_str()),
            Cell::new(signing_public_key_base64.as_str()),
            Cell::new(public_prekey_base64.as_str()),
        ]);
    }

    print!("{table}");

    Ok(())
}
