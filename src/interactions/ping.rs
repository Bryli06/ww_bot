use anyhow::Context;
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_http::Client;
use twilight_model::{
    application::interaction::{application_command::CommandData, Interaction},
    http::interaction::{InteractionResponse, InteractionResponseType},
};
use twilight_gateway::Latency;
use twilight_util::builder::InteractionResponseDataBuilder;

#[derive(CommandModel, CreateCommand)]
#[command(name = "ping", desc = "ping pong")]
pub struct Ping;


impl Ping {
    pub async fn handle(
        interaction: Interaction,
        data: CommandData,
        client: &Client,
    ) -> anyhow::Result<()> {
        let client = client.interaction(interaction.application_id);

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

