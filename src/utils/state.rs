use std::{collections::HashMap, sync::Arc};

use bb8_redis::{bb8::Pool, RedisConnectionManager};
use sqlx::SqlitePool;
use tokio::sync::Mutex;
use twilight_http::{client::InteractionClient, Client as HttpClient};
use twilight_model::id::{marker::ApplicationMarker, Id};

pub struct AppState {
    pub pool: Arc<SqlitePool>,
    pub http: Arc<HttpClient>,
    pub redis: Arc<Pool<RedisConnectionManager>>,
    pub application_id: Id<ApplicationMarker>,
}

impl AppState {
    pub async fn setup(
        database_uri: String,
        redis_uri: String,
        discord_token: String,
    ) -> anyhow::Result<Self> {
        let pool = SqlitePool::connect(&database_uri).await?;
        sqlx::migrate!().run(&pool).await?;
        tracing::info!("Connect to database");

        let http = HttpClient::new(discord_token);
        let application = http.current_user_application().await?.model().await?;

        let manager = RedisConnectionManager::new(redis_uri)?;
        let redis = Pool::builder().build(manager).await?;

        Ok(Self {
            pool: Arc::new(pool),
            http: Arc::new(http),
            redis: Arc::new(redis),
            application_id: application.id,
        })
    }

    pub fn interaction(&self) -> InteractionClient {
        self.http.interaction(self.application_id)
    }
}
