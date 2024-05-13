use anyhow::Context;
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{
    application::interaction::{application_command::CommandData, Interaction},
    http::interaction::{InteractionResponse, InteractionResponseType},
    channel::message::MessageFlags,
};
use twilight_util::builder::{
    InteractionResponseDataBuilder,
    embed::EmbedBuilder,
};

use crate::Bot;

pub const NAME: &str = "end";

#[derive(CommandModel, CreateCommand)]
#[command(name = "end", desc = "end a thread")]
pub struct End;


impl End {
    pub async fn handle(
        interaction: Interaction,
        _data: CommandData,
        bot: &Bot,
    ) -> anyhow::Result<()> {
        let client = bot.client.interaction(interaction.application_id);

        let channel = interaction.channel.clone().context("Could not get message channel. Is this in a channel?")?;

        if !bot.is_thread(channel.id).await.unwrap().unwrap() {
            let embed = EmbedBuilder::new()
                .color(0xEE4B2B)
                .title("Error")
                .description("This is not an active queue thread.")
                .build();

            let data = InteractionResponseDataBuilder::new()
                .embeds([embed])
                .flags(MessageFlags::EPHEMERAL)
                .build();

            let response = InteractionResponse {
                kind: InteractionResponseType::ChannelMessageWithSource,
                data: Some(data),
            };

            client.create_response(interaction.id, &interaction.token, &response).await?;

            return Ok(());
        }

        let acknolewedge = InteractionResponse {
            kind: InteractionResponseType::DeferredChannelMessageWithSource,
            data: Some(InteractionResponseDataBuilder::new()
                       .build()),
        };

        client.create_response(interaction.id, &interaction.token, &acknolewedge).await?;

        let mut users = bot.get_thread(channel.id).await?.unwrap();
        let user_invoke = interaction.author_id().unwrap();

        users.retain(|i| *i != user_invoke);

        tracing::info!("{:?}", users);

        client.create_followup(&interaction.token).content("pong")?.await?;

        Ok(())
    }
}

