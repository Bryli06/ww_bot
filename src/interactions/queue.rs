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
            UserMarker,
        }
    },
    application::interaction::{message_component::MessageComponentInteractionData, Interaction},
    http::interaction::{InteractionResponse, InteractionResponseType},
    channel::{ChannelType::{GuildText, PrivateThread},
                message::{MessageFlags, embed::Embed},
    },
};
use twilight_util::builder::{
    InteractionResponseDataBuilder,
    embed::EmbedBuilder,
};

use rand::distributions::{Alphanumeric, DistString};
use std::collections::hash_map::Entry::{Occupied, Vacant};

use crate::{Bot, CombinedQueues};

pub struct Queue;

impl Queue {
    pub fn get_action_row() -> Component {
        let id = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);

        Component::ActionRow ( ActionRow {
            components: Vec::from([Component::Button(Button {
                custom_id: Some(format!("QueueA-{}", id).to_owned()),
                disabled: false,
                emoji: None,
                label: Some("Queue 1".to_owned()),
                style: ButtonStyle::Primary,
                url: None,
            }),
            Component::Button(Button {
                custom_id: Some(format!("QueueB-{}", id).to_owned()),
                disabled: false,
                emoji: None,
                label: Some("Queue 2".to_owned()),
                style: ButtonStyle::Primary,
                url: None,
            }),
            Component::Button(Button {
                custom_id: Some(format!("QueueC-{}", id).to_owned()),
                disabled: false,
                emoji: None,
                label: Some("Queue 3".to_owned()),
                style: ButtonStyle::Primary,
                url: None,
            }),
            ]),
        })
    }

    fn get_cancel_button(disabled: bool) -> Component {
        let id = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);

        Component::ActionRow ( ActionRow {
            components: Vec::from([
                Component::Button( Button {
                    custom_id: Some(format!("Cancel-{}", id).to_owned()),
                    disabled,
                    emoji: None,
                    label: Some("Leave".to_owned()),
                    style: ButtonStyle::Danger,
                    url: None,
                })
            ])
        })
    }

    pub async fn handle_queue_a(
        interaction: Interaction,
        bot: &Bot,
    ) -> anyhow::Result<()> {

        fn get_group(queue: &mut CombinedQueues, author: Id<UserMarker>) -> (Option<Vec<Id<UserMarker>>>, Embed, bool) {
            let queue = &mut queue.queue_a;
            queue.push(author);
            let len = queue.len();

            let embed = EmbedBuilder::new()
                .color(0x50C878)
                .title("Success")
                .description(format!("Successfully Joined Queue 1\nQueue size: `{}/3`", len).as_str())
                .build();

            if len >= 3 {
                (Some(queue.split_off(len-3)), embed, false)
            } else{
                (None, embed, false)
            }
        }

        Self::handle_queue_generic(interaction, bot, get_group).await
    }

    pub async fn handle_queue_b(
        interaction: Interaction,
        bot: &Bot,
    ) -> anyhow::Result<()> {

        fn get_group(queue: &mut CombinedQueues, author: Id<UserMarker>) -> (Option<Vec<Id<UserMarker>>>, Embed, bool) {
            let queue_b = &mut queue.queue_b;
            let queue_c = &mut queue.queue_c;
            queue_b.push(author);
            let len_b = queue_b.len();
            let len_c = queue_c.len();

            let embed = EmbedBuilder::new()
                .color(0x50C878)
                .title("Success")
                .description(format!("Successfully Joined Queue 2\nYour position: `{}`", len_b).as_str())
                .build();

            if len_b >= 2 && len_c >= 1 {
                let mut group = queue_b.drain(..2).collect::<Vec<Id<UserMarker>>>();
                group.push(queue_c.remove(0));
                (Some(group), embed, true)
            } else{
                (None, embed, true)
            }
        }

        Self::handle_queue_generic(interaction, bot, get_group).await
    }

    pub async fn handle_queue_c(
        interaction: Interaction,
        bot: &Bot,
    ) -> anyhow::Result<()> {
        fn get_group(queue: &mut CombinedQueues, author: Id<UserMarker>) -> (Option<Vec<Id<UserMarker>>>, Embed, bool) {
            let queue_b = &mut queue.queue_b;
            let queue_c = &mut queue.queue_c;
            queue_c.push(author);
            let len_b = queue_b.len();
            let len_c = queue_c.len();

            let embed = EmbedBuilder::new()
                .color(0x50C878)
                .title("Success")
                .description(format!("Successfully Joined Queue 3\nYour position: `{}`", len_c).as_str())
                .build();

            if len_b >= 2 && len_c >= 1 {
                let mut group = queue_b.drain(..2).collect::<Vec<Id<UserMarker>>>();
                group.push(queue_c.remove(0));
                (Some(group), embed, true)
            } else{
                (None, embed, true)
            }
        }

        Self::handle_queue_generic(interaction, bot, get_group).await
    }

    async fn handle_queue_generic(
        interaction: Interaction,
        bot: &Bot,
        f: fn(&mut CombinedQueues, Id<UserMarker>) -> (Option<Vec<Id<UserMarker>>>, Embed, bool)
    ) -> anyhow::Result<()> {
        let client = bot.client.interaction(interaction.application_id);

        let acknolewedge = InteractionResponse {
            kind: InteractionResponseType::DeferredChannelMessageWithSource,
            data: Some(InteractionResponseDataBuilder::new()
                       .flags(MessageFlags::EPHEMERAL)
                       .build()),
        };

        client.create_response(interaction.id, &interaction.token, &acknolewedge).await?;


        let (group, embed, components, queuetype) = { // scope to unlock after finish
            let mut queues = bot.queues.lock().await;
            let message_id = interaction.message.as_ref().unwrap().id;
            let queue: &mut CombinedQueues = match queues.get_mut(&message_id) {
                Some(queue) => queue,
                None => {
                    match queues.entry(message_id) {
                        Occupied(_) => anyhow::bail!("shouldn't be possible"),
                        Vacant(entry) => entry.insert(CombinedQueues {
                            queue_a: Vec::with_capacity(3),
                            queue_b: Vec::new(),
                            queue_c: Vec::new(),
                        })
                    }

                }
            }; // rust moment
            let author = interaction.author_id().unwrap();
            if queue.contains(&author) {
                (None, EmbedBuilder::new()
                                    .color(0xFFE4C4)
                                    .title("Error")
                                    .description("Already joined a queue, request ignored.")
                                    .build(),
                                    Some(Self::get_cancel_button(false)),
                                    None)
            }
            else if bot.is_thread(author).await?.unwrap() {
                (None, EmbedBuilder::new()
                                    .color(0xFFE4C4)
                                    .title("Error")
                                    .description("You are currently in a thread")
                                    .build(), None, None)
            }
            else {
                let group = f(queue, author);
                (group.0, group.1, Some(Self::get_cancel_button(false)), Some(group.2))
            }
        };

        if components.is_some() {
            client.create_followup(&interaction.token).embeds(&[embed])?.components(&[components.unwrap()])?.await?;
        }
        else {
            client.create_followup(&interaction.token).embeds(&[embed])?.await?;
        }

        if group.is_none() {
            return Ok(())
        }

        let group = group.unwrap();

        let id = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);

        let title = match queuetype {
            Some(false) => format!("Echos Farming Thread - A{}", id),
            Some(true) => format!("Echos Farming Thread - B{}", id),
            _ => anyhow::bail!("got impossiblbe type"),
        };

        let thread = bot.client
                        .create_thread(interaction.channel.unwrap().id, title.as_str(), PrivateThread)?
                        .invitable(false)
                        .await?
                        .model()
                        .await?;

        let thread_embed = EmbedBuilder::new()
            .color(0x63c5da)
            .title("Welcome")
            .description("Welcome to this echos farming thread. When you have finished, please use the command /end")
            .build();

        let _ = bot.client
            .create_message(thread.id)
            .embeds(&[thread_embed])?
            .content(format!("<@{}> <@{}> <@{}>", group[0].get(), group[1].get(), group[2].get()).as_str())?
            .await?
            .model()
            .await?;

        let _ = bot.insert_thread(thread.id, group[0], group[1], group[2]).await;

        Ok(())
    }

    pub async fn handle_cancel(
        interaction: Interaction,
        bot: &Bot,
    ) -> anyhow::Result<()> {
        let message = interaction.message.clone().unwrap();
        let reference = message.clone().reference.unwrap().message_id.unwrap();
        let embed = {
            let mut queues = bot.queues.lock().await;
            match queues.get_mut(&reference) {
                Some(queue) => {
                    let author = interaction.author_id().unwrap();
                    if queue.contains(&author) {
                        queue.pop(&author);
                        EmbedBuilder::new()
                            .color(0x50C878)
                            .title("Confirmed")
                            .description("Leaving queue.")
                            .build()
                    }
                    else {
                        EmbedBuilder::new()
                            .color(0xEE4B2B)
                            .title("Error")
                            .description("Attempted to leave queue when not in one.")
                            .build()
                    }
                },
                None => {
                    EmbedBuilder::new()
                        .color(0xEE4B2B)
                        .title("Error")
                        .description("Attempted to leave queue when not in one.")
                        .build()
                },
            }
        };

        let client = bot.client.interaction(interaction.application_id);

        let data = InteractionResponseDataBuilder::new()
                       .embeds([embed])
                       .flags(MessageFlags::EPHEMERAL)
                       .build();

        let acknolewedge = InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(data),
        };

        client.create_response(interaction.id, &interaction.token, &acknolewedge).await?;

        Ok(())
    }
}
