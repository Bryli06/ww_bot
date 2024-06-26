mod interactions;
mod handle;
mod database;

use std::{env, sync::{Arc}};
use std::collections::HashMap;

use anyhow::Context;
use tracing::Level;
use sqlx::PgPool;
use twilight_gateway::{
    stream::{self, ShardEventStream},
    Config, Intents,
    EventTypeFlags,
};
use futures_util::StreamExt;
use twilight_http::Client;
use twilight_model::{
    gateway::{
        payload::outgoing::update_presence::UpdatePresencePayload,
        presence::{ActivityType, MinimalActivity, Status},
    },
    id::{
        Id,
        marker::{
            UserMarker,
            MessageMarker,
        }
    },
};

use twilight_interactions::command::CreateCommand;
use tokio::sync::Mutex;

use crate::{interactions::{ping::Ping, setup::Setup, end::End}};

pub struct Bot {
    db: PgPool,
    client: Client,
    queues: Arc<Mutex<HashMap<Id<MessageMarker>, CombinedQueues>>>,
}

#[derive(Debug)]
pub struct CombinedQueues {
    queue_a: Vec<Id<UserMarker>>,
    queue_b: Vec<Id<UserMarker>>,
    queue_c: Vec<Id<UserMarker>>,
}

impl CombinedQueues {
    pub fn contains(&self, id: &Id<UserMarker>) -> bool {
        self.queue_a.contains(id) || self.queue_b.contains(id) || self.queue_c.contains(id)
    }

    pub fn pop(&mut self, id: &Id<UserMarker>) {
        self.queue_a.retain(|i| *i != *id);
        self.queue_b.retain(|i| *i != *id);
        self.queue_c.retain(|i| *i != *id);
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let token = env::var("TOKEN").context("Bot token is not set")?;
    let database_url = env::var("DATABASE_URL").context("Database url is not set")?;

    let db = PgPool::connect(&database_url).await?;

    tracing_subscriber::fmt()
        .compact()
        .with_max_level(Level::INFO)
        .init();

    let bot = Arc::new( Bot {
        client: Client::new(token.clone()),
        db,
        queues: Arc::new(Mutex::new(HashMap::new())),
    });

    let _ = bot.setup_database().await;

    let config = Config::builder(token.clone(),
                                 Intents::GUILDS)
        .event_types(EventTypeFlags::THREAD_UPDATE |
                     EventTypeFlags::INTERACTION_CREATE |
                     EventTypeFlags::GATEWAY_HELLO |
                     EventTypeFlags::GATEWAY_HEARTBEAT |
                     EventTypeFlags::GATEWAY_RECONNECT |
                     EventTypeFlags::GATEWAY_HEARTBEAT_ACK |
                     EventTypeFlags::GATEWAY_INVALIDATE_SESSION )
        .presence(presence())
        .build();

    let commands = [
        Ping::create_command().into(),
        Setup::create_command().into(),
        End::create_command().into(),
    ];

    let application = bot.client.current_user_application().await?.model().await?;
    let interaction_client = bot.client.interaction(application.id);

    tracing::info!("logged as {} with ID {}", application.name, application.id);

    if let Err(error) = interaction_client.set_global_commands(&commands).await {
        tracing::error!(?error, "failed to register commands");
    }

    let mut shards = stream::create_recommended(&bot.client, config, |_id, builder| builder.build())
        .await?
        .collect::<Vec<_>>();
    let mut stream = ShardEventStream::new(shards.iter_mut());

    while let Some((shard, event)) = stream.next().await {
        let bot_ref = Arc::clone(&bot);
        let event = match event {
            Ok(event) => event,
            Err(error) => {
                if error.is_fatal() {
                    tracing::error!(?error, "fatal error while receiving event");
                    break;
                }

                tracing::warn!(?error, "error while receiving event");
                continue;
            }
        };

        tracing::info!(kind = ?event.kind(), shard = ?shard.id().number(), "received event");
        tokio::spawn(async move {
            bot_ref.process(event).await;
        });
    }

    Ok(())
}

fn presence() -> UpdatePresencePayload {
    let activity = MinimalActivity {
        kind: ActivityType::Watching,
        name: String::from("you sleep"),
        url: None,
    };

    UpdatePresencePayload {
        activities: vec![activity.into()],
        afk: false,
        since: None,
        status: Status::Online,
    }
}
