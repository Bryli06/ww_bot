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

use crate::{Bot, CombinedQueues};
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
            .description("This is a queue bot for multiplayer Echoes farming in Wuthering Waves.

Choose a queue based on your needs for the multiplayer session :-
1. **Co-op (1)**: If you want to share elites with others who are also willing to share their elites. (+1 reputation)
2. **Carry (2)**: If you are willing to help fight elites for others in their worlds.
3. **Assist (3)**: If you need help in fighting elites in your own world. (+2 reputation)

**Assist (3)** is recommended for people who just want to farm their own world, whereas **Co-op (1)** would suit those better who are just starting the farming session and also want to farm other people's worlds alongside their own. **Carry (2)** works best for those who have farmed their own world already but still want more echoes.

Don’t forget to poll at the end of each multiplayer session for reputation points! These points will be useful soon…")
            .build();

        let message = bot.client
            .create_message(channel.id)
            .embeds(&[queue_embed])?
            .await?
            .model()
            .await?;

        bot.client.update_message(channel.id, message.id)
            .components(Some(&[queue::Queue::get_action_row()]))?
            .await?;

        bot.queues.lock().await.insert(message.id, CombinedQueues {
            queue_a: Vec::with_capacity(3),
            queue_b: Vec::new(),
            queue_c: Vec::new(),
        });


        let embed = EmbedBuilder::new()
            .color(0x50C878)
            .title("Success")
            .description("Successfully setup queue")
            .build();

        client.create_followup(&interaction.token).embeds(&[embed])?.await?;

        Ok(())
    }
}

