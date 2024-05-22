use std::{mem, sync::Arc};

use anyhow::{bail, Context};
use twilight_model::{
    channel::{Channel, message::component::{ActionRow, Component, SelectMenu, SelectMenuOption, TextInput, TextInputStyle}, message::{MessageFlags, embed::Embed}},
    id::{
        Id,
        marker::{
            UserMarker,
        }
    },
    user::User,
    application::interaction::{message_component::MessageComponentInteractionData, Interaction, modal::ModalInteractionData},
    http::interaction::{InteractionResponse, InteractionResponseType},
};

use twilight_util::builder::{
    InteractionResponseDataBuilder,
    embed::EmbedBuilder,
};
use rand::distributions::{Alphanumeric, DistString};

use crate::Bot;

fn get_select_row(users: Vec<User>) -> Component {
    let id = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);

    let mut options: Vec<SelectMenuOption> = Vec::new();

    for user in users {
        options.push(
            SelectMenuOption {
                default: false,
                description: None,
                emoji: None,
                label: user.name,
                value: user.id.to_string(),
            }
        );
    }

    Component::ActionRow(ActionRow {
        components: vec![Component::SelectMenu(SelectMenu {
            custom_id: format!("Report - {}", id).to_owned(),
            disabled: false,
            max_values: Some(2),
            min_values: Some(1),
            options,
            placeholder: Some("Choose a user".to_owned()),
        })],
    })
}

fn get_modal() -> Component {
    let id = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);

    Component::ActionRow(ActionRow {
        components: vec![Component::TextInput(TextInput {
            custom_id: format!("TextInput - {}", id).to_owned(),
            label: "Report reason for user(s)".to_owned(),
            required: Some(true),
            style: TextInputStyle::Paragraph,
            value: None,
            max_length: None,
            min_length: None,
            placeholder: None,
        })],
    })
}

impl Bot {
    pub async fn handle_rep(&self, users: Vec<Id<UserMarker>>, channel: Channel) -> anyhow::Result<()> {
        let name = channel.name.unwrap();
        let index = name.find('-').unwrap();
        let queue_type = match name.chars().nth(index + 2){
            Some('A') => false,
            Some('B') => true,
            _ => anyhow::bail!("invalid queue type"),
        };

        if queue_type {
            let _ = self.update_user(*users.get(2).unwrap(), 2).await;
        }
        else {
            for user in &users {
                let _ = self.update_user(*user, 1).await;
            }
        }


        match users.as_slice() {
            [id1, id2, id3] => {
                let user1 = self.client.user(*id1).await?.model().await?;
                let user2 = self.client.user(*id2).await?.model().await?;
                let user3 = self.client.user(*id3).await?.model().await?;

                let _ = self.dm_poll(user1.clone(), user2.clone(), user3.clone()).await;
                let _ = self.dm_poll(user2.clone(), user1.clone(), user3.clone()).await;
                let _ = self.dm_poll(user3.clone(), user1.clone(), user2.clone()).await;
            },
            _ => bail!("Not 3 users in users"),
        };

        let _ = self.remove_thread(channel.id).await;

        Ok(())
    }


    async fn dm_poll(&self, user1: User, user2: User, user3: User) -> anyhow::Result<()> {
        let channel = self.client.create_private_channel(user1.id).await?.model().await?;

        let embed = EmbedBuilder::new()
            .color(0x50C878)
            .title("Thank you for your participation")
            .description("Would you like to report any members? If so, please select them below. If not, you may ignore this message.")
            .build();

        let _ = self.client.create_message(channel.id)
            .embeds(&[embed])?
            .components(&[get_select_row(vec![user2, user3])])?
            .await?;

        Ok(())
    }

}

pub async fn handle_report(
        interaction: Interaction,
        bot: &Bot,
        data: &[String]
    ) -> anyhow::Result<()> {

    let id = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);

    let client = bot.client.interaction(interaction.application_id);

    let mut string = data[0].clone();
    if data.len() > 1 {
        string.push_str(&(" - ".to_owned() + data[1].as_str()));
    }

    let data = InteractionResponseDataBuilder::new()
                   .components([get_modal()])
                   .title("Report")
                   .custom_id(format!("Modal - {} - {}", string, id))
                   .build();

    let acknolewedge = InteractionResponse {
        kind: InteractionResponseType::Modal,
        data: Some(data),
    };

    client.create_response(interaction.id, &interaction.token, &acknolewedge).await?;

    Ok(())
}

pub async fn handle_text (
    interaction: Interaction,
    data: ModalInteractionData,
    bot: &Bot,
) -> anyhow::Result<()> {
    let report = data.components.get(0).unwrap().components.get(0).unwrap().value.as_ref().unwrap();

    let reporter = interaction.user.unwrap().id;

    let users =  data.custom_id.split(" - ").collect::<Vec<&str>>();

    let client = bot.client.interaction(interaction.application_id);

    let embed = EmbedBuilder::new()
        .color(0x50C878)
        .title("Report filed")
        .description("Thank you for filing a report.")
        .build();

    let data = InteractionResponseDataBuilder::new()
                   .embeds([embed])
                   .flags(MessageFlags::EPHEMERAL)
                   .build();

    let acknolewedge = InteractionResponse {
        kind: InteractionResponseType::ChannelMessageWithSource,
        data: Some(data),
    };

    client.create_response(interaction.id, &interaction.token, &acknolewedge).await?;

    let message = client.response(&interaction.token).await?.model().await?.reference.unwrap();

    let message = bot.client.message(message.channel_id.unwrap(), message.message_id.unwrap()).await?.model().await?;

    let mut options: Vec<SelectMenuOption> = if let Component::ActionRow (ActionRow { components: arr }) = message.components.get(0).unwrap() {
        if let Component::SelectMenu ( SelectMenu { options: options, .. } ) = arr.get(0).unwrap() {
            options.to_vec()
        } else {
            bail!("Should be a select menu, is not.")
        }
    } else {
        bail!("First component should be Action row.")
    };

    options.retain(|i| !users.contains(&i.value.as_str()));


    if options.len() > 0 {
        let id = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
        let _ = bot.client.update_message(message.channel_id, message.id)
                    .components(Some(&[Component::ActionRow(ActionRow {
            components: vec![Component::SelectMenu(SelectMenu {
                custom_id: format!("Report - {}", id).to_owned(),
                disabled: false,
                max_values: None,
                min_values: None,
                options,
                placeholder: Some("Choose a user".to_owned()),
            })],})]))?.await?;
    }
    else {
        let message = bot.client.update_message(message.channel_id, message.id)
                        .components(Some(&[]))?.await?.model().await?;
    }

    let mut embeds: Vec<Embed> = Vec::new();
    for user in &users.as_slice()[1..users.len()-1] {
        let id = user.parse::<u64>().unwrap();
        let _  = bot.update_user(Id::new(id), -1).await;
        embeds.push(EmbedBuilder::new()
            .color(0x50C878)
            .title("Report")
            .description(format!("Report issued by <@{}> against <@{}>: \n`{}`", reporter.get(), user, report))
            .build());
    }

    let channel_id = Id::new(std::env::var("LOG")
                                        .context("Log channel not set")?
                                        .parse::<u64>()
                                        .unwrap());

    bot.client.create_message(channel_id)
        .embeds(embeds.as_slice())?
        .await?;
    Ok(())
}

