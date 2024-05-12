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
            MessageMarker
        }
    },
};

pub struct queue;

impl Buttons {
    pub fn get_action_row(channel: Id<ChannelMarker>, message: Id<MessageMarker>) -> Component {

        Component::ActionRow ( ActionRow {
            components: Vec::from([Component::Button(Button {
                custom_id: Some(format!("{}-{}-1", channel, message).to_owned()),
                disabled: false,
                emoji: None,
                label: Some("Queue A".to_owned()),
                style: ButtonStyle::Primary,
                url: None,
            }),
            Component::Button(Button {
                custom_id: Some(format!("{}-{}-2", channel, message).to_owned()),
                disabled: false,
                emoji: None,
                label: Some("Queue B".to_owned()),
                style: ButtonStyle::Primary,
                url: None,
            }),
            Component::Button(Button {
                custom_id: Some(format!("{}-{}-3", channel, message).to_owned()),
                disabled: false,
                emoji: None,
                label: Some("Queue C".to_owned()),
                style: ButtonStyle::Primary,
                url: None,
            }),
            ]),
        })
    }


}
