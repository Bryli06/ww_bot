use sqlx::{query, query_scalar, Postgres, PgPool};
use anyhow::Result;
use twilight_model::id::{
        Id,
        marker::{
            UserMarker,
            ChannelMarker,
            MessageMarker,
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
    pub async fn update_user(&self, user_id: Id<UserMarker>, change: i32) -> Result<()> {
        query!(
            "INSERT INTO users (user_id, rep) VALUES ($1, 0) ON CONFLICT (user_id) DO NOTHING",
            user_id.encode(),
        )
        .execute(&self.db)
        .await?;

        query!(
            "UPDATE users SET rep = rep + $2 WHERE user_id = $1",
            user_id.encode(),
            change,
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn get_rep(&self, user_id: Id<UserMarker>) -> Result<Option<i32>> {
        Ok(query_scalar!(
            "SELECT rep FROM users WHERE user_id = $1",
            user_id.encode()
        )
        .fetch_optional(&self.db)
        .await?)
    }

    pub async fn insert_thread(&self, channel_id: Id<ChannelMarker>) -> Result<()> {
        query!(
            "INSERT INTO threads (channel_id) VALUES ($1) ON CONFLICT (channel_id) DO NOTHING",
            channel_id.encode(),
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn get_thread(&self, channel_id: Id<ChannelMarker>) -> Result<Option<bool>> {
        Ok(query_scalar!(
            "SELECT count(1) > 0 FROM threads WHERE channel_id = $1;",
            channel_id.encode()
        )
        .fetch_optional(&self.db)
        .await?.unwrap())
    }

    pub async fn remove_thread(&self, channel_id: Id<ChannelMarker>) -> Result<()> {
        query!(
            "DELETE FROM threads WHERE channel_id = $1;",
            channel_id.encode()
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(())
    }

    pub async fn setup_database(&self) -> Result<()> {
        query!(
            "CREATE TABLE IF NOT EXISTS threads (channel_id BIGINT UNIQUE NOT NULL);"
        )
        .execute(&self.db)
        .await?;

        query!(
             "CREATE TABLE IF NOT EXISTS users (user_id BIGINT UNIQUE NOT NULL, rep SERIAL NOT NULL);"
        )
        .execute(&self.db)
        .await?;
        Ok(())
    }
}

