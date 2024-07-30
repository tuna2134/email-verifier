use std::{collections::HashMap, sync::Arc};

use sqlx::SqlitePool;
use tokio::sync::Mutex;
use twilight_http::{client::InteractionClient, Client as HttpClient};
use twilight_model::id::{marker::ApplicationMarker, Id};

pub struct AppState {
    pub pool: Arc<SqlitePool>,
    pub http: Arc<HttpClient>,
    pub cache: Arc<Mutex<HashMap<String, String>>>,
    pub application_id: Id<ApplicationMarker>,
}

impl AppState {
    pub async fn setup(database_uri: String, discord_token: String) -> anyhow::Result<Self> {
        let pool = SqlitePool::connect(&database_uri).await?;
        sqlx::migrate!().run(&pool).await?;
        tracing::info!("Connect to database");

        let http = HttpClient::new(discord_token);
        let application = http.current_user_application().await?.model().await?;

        Ok(Self {
            pool: Arc::new(pool),
            http: Arc::new(http),
            cache: Arc::new(Mutex::new(HashMap::new())),
            application_id: application.id,
        })
    }

    pub fn interaction(&self) -> InteractionClient {
        self.http.interaction(self.application_id)
    }
}
