use std::{mem, sync::Arc};

use anyhow::bail;
use twilight_gateway::Event;
use twilight_http::Client;
use twilight_model::application::interaction::{
    application_command::CommandData, Interaction, InteractionData,
};

use crate::interactions::ping::Ping;

pub async fn process(event: Event, client: Arc<Client>) {
    let mut interaction = match event {
        Event::InteractionCreate(interaction) => interaction.0,
        _ => return,
    };

    let data = match mem::take(&mut interaction.data) {
        Some(InteractionData::ApplicationCommand(data)) => *data,
        _ => {
            tracing::warn!("ignoring non-command interaction");
            return;
        }
    };

    if let Err(error) = handle_command(interaction, data, &client).await {
        tracing::error!(?error, "error while handling command");
    }
}

async fn handle_command(
    interaction: Interaction,
    data: CommandData,
    client: &Client,
) -> anyhow::Result<()> {
    match &*data.name {
        "ping" => Ping::handle(interaction, data, client).await,
        "setup" => bail!("hi"),
        name => bail!("unknown command: {}", name),
    }
}
