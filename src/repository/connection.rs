use mongodb::{Client, Database, error::Result};

use crate::{app_env, utils::constants::DB_NAME};

#[derive(Clone)]
pub struct DbConnection {
    db: Database,
}

impl DbConnection {
    pub async fn init() -> Result<Self> {
        let client = Client::with_uri_str(&app_env!().database_url).await?;
        let db = client.database(DB_NAME);
        Ok(Self { db })
    }

    pub fn database(&self) -> &Database {
        &self.db
    }
}
