use sqlx::{query, query_scalar, Postgres, PgPool};
use anyhow::Result;
use twilight_model::id::{
        Id,
        marker::{
            ChannelMarker,
            MessageMarker
        }
    };

use crate::Bot;

trait Encode<'a, T: sqlx::Encode<'a, Postgres>> {
    fn encode(&self) -> T;
}

impl<T> Encode<'_, i64> for Id<T> {
    #[allow(clippy::cast_possible_wrap)]
    fn encode(&self) -> i64 {
        self.get() as i64
    }
}

impl Bot {
    /*
    pub async fn insert_button(&self, custom_id: String, message: Id<MessageMarker>, channel: Id<ChannelMarker>) -> Result<()> {
        query!(
            "INSERT INTO buttons (custom_id, message_id, channel_id) VALUES ($1, $2, $3) ON CONFLICT (custom_id) DO \
             UPDATE SET message_id = $2, channel_id = $3)",
            message.encode(),
            channel.encode()
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }
    */

    pub async fn setup_database(&self) -> Result<()> {
        query!(
            "CREATE TABLE IF NOT EXISTS buttons (custom_id VARCHAR UNIQUE NOT NULL, message_id SERIAL NOT NULL, channel_id SERIAL NOT NULL);"
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }
}

