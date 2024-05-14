use anyhow::Context;
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{
    application::interaction::{application_command::CommandData, Interaction},
    http::interaction::{InteractionResponse, InteractionResponseType},
    channel::{ChannelType::GuildText,
                message::MessageFlags,
    },
    guild::Permissions,
};
use twilight_util::builder::{
    InteractionResponseDataBuilder,
    embed::EmbedBuilder,
};

use crate::Bot;
use crate::interactions::queue;

pub const NAME: &str = "setup";

#[derive(CommandModel, CreateCommand)]
#[command(name = "setup", desc = "Sends Setup Message", default_permissions = "admin_perms")]
pub struct Setup;

fn admin_perms() -> Permissions {
    Permissions::ADMINISTRATOR
}


impl Setup {
    pub async fn handle(
        interaction: Interaction,
        data: CommandData,
        bot: &Bot,
    ) -> anyhow::Result<()> {
        let client = bot.client.interaction(interaction.application_id);

        let acknolewedge = InteractionResponse {
            kind: InteractionResponseType::DeferredChannelMessageWithSource,
            data: Some(InteractionResponseDataBuilder::new()
                       .flags(MessageFlags::EPHEMERAL)
                       .build()),
        };

        client.create_response(interaction.id, &interaction.token, &acknolewedge).await?;

        let channel = interaction.channel.context("Could not get message channel. Is this in a channel?")?;

        if channel.kind != GuildText {
            let embed = EmbedBuilder::new()
                .color(0xEE4B2B)
                .title("Error")
                .description("Command attempted to run in incorrect channel type. Is this a guild channel?")
                .build();

            client.create_followup(&interaction.token).embeds(&[embed])?.await?;

            return Ok(())
        }

        let queue_embed = EmbedBuilder::new()
            .color(0x63c5da)
            .title("Queue")
            .description("description of each queue")
            .build();

        let message = bot.client
            .create_message(channel.id)
            .embeds(&[queue_embed])?
            .await?
            .model()
            .await?;

        bot.client.update_message(channel.id, message.id)
            .components(Some(&[queue::Queue::get_action_row(channel.id, message.id)]))?
            .await?;


        let embed = EmbedBuilder::new()
            .color(0x50C878)
            .title("Success")
            .description("Successfully setup queue")
            .build();

        client.create_followup(&interaction.token).embeds(&[embed])?.await?;

        Ok(())
    }
}

