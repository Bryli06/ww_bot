use anyhow::Context;
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{
    application::interaction::{application_command::CommandData, Interaction},
    http::interaction::{InteractionResponse, InteractionResponseType},
};
use twilight_util::builder::InteractionResponseDataBuilder;

use crate::Bot;

pub const NAME: &str = "ping";

#[derive(CommandModel, CreateCommand)]
#[command(name = "ping", desc = "ping pong")]
pub struct Ping;


impl Ping {
    pub async fn handle(
        interaction: Interaction,
        _data: CommandData,
        bot: &Bot,
    ) -> anyhow::Result<()> {
        let client = bot.client.interaction(interaction.application_id);

        let data = InteractionResponseDataBuilder::new()
            .content("pong")
            .build();

        let response = InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(data),
        };

        client.create_response(interaction.id, &interaction.token, &response).await?;

        Ok(())
    }
}

