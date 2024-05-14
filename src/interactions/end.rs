use anyhow::Context;
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{
    application::interaction::{application_command::CommandData, Interaction},
    http::interaction::{InteractionResponse, InteractionResponseType},
    channel::message::{
        MessageFlags,
        component::{
            ActionRow,
            Button,
            ButtonStyle,
            Component
        },
    }
};
use twilight_util::builder::{
    InteractionResponseDataBuilder,
    embed::EmbedBuilder,
};

use rand::distributions::{Alphanumeric, DistString};
use regex::Regex;

use crate::Bot;

pub const NAME: &str = "end";

#[derive(CommandModel, CreateCommand)]
#[command(name = "end", desc = "end a thread")]
pub struct End;


impl End {

    pub fn get_action_row(disabled: bool) -> Component {
        let id = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);

        Component::ActionRow ( ActionRow {
            components: Vec::from([Component::Button(Button {
                custom_id: Some(format!("End-{}", id).to_owned()),
                disabled,
                emoji: None,
                label: Some("Confirm".to_owned()),
                style: ButtonStyle::Success,
                url: None,
            }),
            ]),
        })
    }

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
        /*
        let acknolewedge = InteractionResponse {
            kind: InteractionResponseType::DeferredChannelMessageWithSource,
            data: Some(InteractionResponseDataBuilder::new()
                       .build()),
        };

        client.create_response(interaction.id, &interaction.token, &acknolewedge).await?;
        */

        let mut users = bot.get_thread(channel.id).await?.unwrap();
        let user_invoke = interaction.author_id().unwrap();

        users.retain(|i| *i != user_invoke);

        let ping = users.into_iter().map(|i| format!("<@{}> ", i)).collect::<String>();

        let embed = EmbedBuilder::new()
            .color(0xEE4B2B)
            .title("End Session")
            .description(format!("<@{}> would like to end this session. Click confirm to successfully end.", user_invoke.get()))
            .build();

        let data = InteractionResponseDataBuilder::new()
                      .content(ping.as_str())
                       .embeds([embed])
                       .components([Self::get_action_row(false)])
                       .build();

        let acknolewedge = InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(data),
        };

        client.create_response(interaction.id, &interaction.token, &acknolewedge).await?;
        /*
        client.create_followup(&interaction.token)
              .content(ping.as_str())?
              .embeds(&[embed])?
              .components(&[Self::get_action_row(false)])?
              .await?;
        */

        Ok(())
    }

    pub async fn handle_confirm (
        interaction: Interaction,
        bot: &Bot,
    ) -> anyhow::Result<()> {
        let client = bot.client.interaction(interaction.application_id);

        let message = interaction.message.clone().unwrap();

        let valid: Vec<u64> = Regex::new(r"\d+").unwrap()
            .find_iter(message.content.as_str())
            .map(|i| i.as_str().parse::<u64>().unwrap())
            .collect::<Vec<u64>>(); //what the f-

        if !valid.contains(&interaction.author_id().unwrap().get()) {
            let embed = EmbedBuilder::new()
                .color(0xEE4B2B)
                .title("Error")
                .description("You do not have permission to confirm this request.")
                .build();

            let data = InteractionResponseDataBuilder::new()
                           .flags(MessageFlags::EPHEMERAL)
                           .embeds([embed])
                           .build();

            let acknolewedge = InteractionResponse {
                kind: InteractionResponseType::ChannelMessageWithSource,
                data: Some(data),
            };

            client.create_response(interaction.id, &interaction.token, &acknolewedge).await?;

            return Ok(());
        }

        let embed = EmbedBuilder::new()
            .color(0x50C878)
            .title("Confirmed")
            .description("Closing this thread.")
            .build();

        let data = InteractionResponseDataBuilder::new()
                       .flags(MessageFlags::EPHEMERAL)
                       .embeds([embed])
                       .build();

        let acknolewedge = InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(data),
        };

        client.create_response(interaction.id, &interaction.token, &acknolewedge).await?;


        let _ = bot.client.update_message(message.channel_id, message.id)
                  .components(Some(&[Self::get_action_row(true)]));

        bot.client.update_thread(interaction.channel.unwrap().id)
                  .archived(true)
                  .locked(true)
                  .await?;
        Ok(())
    }
}

