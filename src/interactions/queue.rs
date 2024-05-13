use twilight_model::{
    channel::message::component::{
        ActionRow,
        Button,
        ButtonStyle,
        Component
    },
    id::{
        Id,
        marker::{
            ChannelMarker,
            MessageMarker,
            UserMarker,
        }
    },
    application::interaction::{message_component::MessageComponentInteractionData, Interaction},
    http::interaction::{InteractionResponse, InteractionResponseType},
    channel::{ChannelType::{GuildText, PrivateThread},
                message::MessageFlags,
    },
};
use twilight_util::builder::{
    InteractionResponseDataBuilder,
    embed::EmbedBuilder,
};

use rand::distributions::{Alphanumeric, DistString};

use crate::{Bot, CombinedQueues};

pub struct Queue;

impl Queue {
    pub fn get_action_row(channel: Id<ChannelMarker>, message: Id<MessageMarker>) -> Component {
        let id = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);

        Component::ActionRow ( ActionRow {
            components: Vec::from([Component::Button(Button {
                custom_id: Some(format!("QueueA-{}", id).to_owned()),
                disabled: false,
                emoji: None,
                label: Some("Queue A".to_owned()),
                style: ButtonStyle::Primary,
                url: None,
            }),
            Component::Button(Button {
                custom_id: Some(format!("QueueB-{}", id).to_owned()),
                disabled: false,
                emoji: None,
                label: Some("Queue B".to_owned()),
                style: ButtonStyle::Primary,
                url: None,
            }),
            Component::Button(Button {
                custom_id: Some(format!("QueueC-{}", id).to_owned()),
                disabled: false,
                emoji: None,
                label: Some("Queue C".to_owned()),
                style: ButtonStyle::Primary,
                url: None,
            }),
            ]),
        })
    }

    pub async fn handle_queue_a(
        interaction: Interaction,
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


        let (group, embed) = { // scope to unlock after finish
            let mut queues = bot.queues.lock().unwrap();
            let author = interaction.author_id().unwrap();
            if queues.contains(&author) {
                (None, EmbedBuilder::new()
                                    .color(0xFFE4C4)
                                    .title("Error")
                                    .description("Already joined queue, request ignored.")
                                    .build())
            }
            else {
                let mut queue = &mut queues.queue_a;
                queue.push(author);
                let len = queue.len();

                let embed = EmbedBuilder::new()
                    .color(0x50C878)
                    .title("Success")
                    .description("Successfully Joined Queue A")
                    .build();

                if len >= 1 {
                    (Some(queue.split_off(len-1)), embed)
                } else{
                    (None, embed)
                }
            }
        };

        client.create_followup(&interaction.token).embeds(&[embed])?.await?;

        if group.is_none() {
            return Ok(())
        }

        let group = group.unwrap();

        let thread = bot.client
                        .create_thread(interaction.channel_id.unwrap(), "Echos Farming Thread", PrivateThread)?
                        .invitable(false)
                        .await?
                        .model()
                        .await?;

        let thread_embed = EmbedBuilder::new()
            .color(0x63c5da)
            .title("Welcome")
            .description("Welcome to this echos farming thread. When you have finished, please use the command /end")
            .build();

        let message = bot.client
            .create_message(thread.id)
            .embeds(&[thread_embed])?
            .content(format!("<@{}>", group[0].get()).as_str())?
            .await?
            .model()
            .await?;

        Ok(())
    }

    pub async fn handle_queue_b(
        interaction: Interaction,
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

        let embed = EmbedBuilder::new()
            .color(0x50C878)
            .title("Success")
            .description("This shit aint implemented yet bozo")
            .build();

        client.create_followup(&interaction.token).embeds(&[embed])?.await?;

        Ok(())
    }

    pub async fn handle_queue_c(
        interaction: Interaction,
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

        let embed = EmbedBuilder::new()
            .color(0x50C878)
            .title("Success")
            .description("This shit aint implemented yet bozo")
            .build();

        client.create_followup(&interaction.token).embeds(&[embed])?.await?;

        Ok(())
    }
}