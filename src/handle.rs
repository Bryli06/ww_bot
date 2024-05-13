use std::{mem, sync::Arc};

use anyhow::{bail, Context};
use twilight_gateway::Event;
use twilight_http::Client;
use twilight_model::{
    application::interaction::{
        application_command::CommandData,
        message_component::MessageComponentInteractionData,
        Interaction,
        InteractionData,
        InteractionType,
    },
    channel::{Channel, message::component::{ComponentType, ActionRow, Component}},
};

use crate::interactions::{ping, setup, queue, end};
use crate::Bot;

impl Bot {
    pub async fn process(
        &self,
        event: Event,
    ) {
        match event {
            Event::InteractionCreate(interaction) => self.interaction_create(interaction.0).await,
            Event::ThreadUpdate(channel) => self.thread_update(channel.0).await,
            _ => return,
        };
    }

    async fn interaction_create(&self, mut interaction: Interaction) {
        match interaction.kind {
            InteractionType::ApplicationCommand => {
                let data = if let Some(InteractionData::ApplicationCommand(data)) = mem::take(&mut interaction.data) {
                    *data
                } else {
                    tracing::error!("Data could not be unpacked as CommandData");
                    return;
                };

                if let Err(error) = self.handle_command(interaction, data).await {
                    tracing::error!(?error, "error while handling command");
                }

            }
            InteractionType::MessageComponent => {
                let data = if let Some(InteractionData::MessageComponent(data)) = mem::take(&mut interaction.data) {
                    data
                } else {
                    tracing::error!("Data could not be unpacked as MessageComponentInteractionData");
                    return;
                };

                if let Err(error) = self.handle_interaction(interaction, data).await {
                    tracing::error!(?error, "error while handling interaction");
                }
            }
            _ => {
                tracing::warn!("ignoring modal interaction");
                return;
            }
        };

    }

    async fn thread_update(&self, channel: Channel) {
        tracing::info!("{:?}", channel);
        if channel.thread_metadata.unwrap().archived {
            let _ = self.remove_thread(channel.id).await;
        }
    }

    async fn handle_command(
        &self,
        interaction: Interaction,
        data: CommandData,
    ) -> anyhow::Result<()> {
        match &*data.name {
            ping::NAME => ping::Ping::handle(interaction, data, self).await,
            setup::NAME => setup::Setup::handle(interaction, data, self).await,
            end::NAME => end::End::handle(interaction, data, self).await,
            name => bail!("unknown command: {}", name),
        }
    }

    async fn handle_interaction(
        &self,
        interaction: Interaction,
        data: MessageComponentInteractionData,
    ) -> anyhow::Result<()> {
        let id = Some(data.custom_id);
        match data.component_type{
            ComponentType::Button => {
                let component_number = if let Component::ActionRow (ActionRow { components: arr }) = &interaction.message.as_ref().unwrap()
                                    .components[0] {
                    arr.iter().position(|i| {
                        match i {
                            Component::Button(button) => button.custom_id == id,
                            _ => false
                        }
                    })
                } else { None };

                match component_number {
                    Some(0) => queue::Queue::handle_queue_a(interaction, self).await,
                    Some(1) => queue::Queue::handle_queue_b(interaction, self).await,
                    Some(2) => queue::Queue::handle_queue_c(interaction, self).await,
                    Some(_) => bail!("Shouldn't have more than 3 buttons"),
                    None => bail!("Shouldn't be possible"),
                }
            },
            _ => bail!("ignoring interaction"),
        }
    }
}
