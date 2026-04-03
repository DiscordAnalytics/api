use anyhow::Result;
use s3::{
    Auth, Client, Credentials,
    providers::{R2Endpoint, R2Jurisdiction, cloudflare_r2},
};

use crate::app_env;

#[derive(Clone)]
pub struct R2Repository {
    pub client: Client,
    bucket_name: String,
}

impl R2Repository {
    pub fn new() -> Result<Self> {
        let credentials =
            Credentials::new(&app_env!().cloudflare_id, &app_env!().cloudflare_token)?;

        let client = cloudflare_r2(
            &app_env!().r2_account_id,
            R2Endpoint::Jurisdiction(R2Jurisdiction::Eu),
        )?
        .async_client_builder()?
        .auth(Auth::Static(credentials))
        .build()?;

        Ok(Self {
            client,
            bucket_name: app_env!().r2_bucket_name.to_string(),
        })
    }

    pub async fn ping(&self) -> Result<()> {
        self.client
            .objects()
            .list_v2(&self.bucket_name)
            .max_keys(1)
            .send()
            .await?;
        Ok(())
    }

    pub async fn put_object(&self, key: &str, body: &[u8], content_type: &str) -> Result<()> {
        self.client
            .objects()
            .put(&self.bucket_name, key)
            .content_type(content_type)
            .body_bytes(body.to_vec())
            .send()
            .await?;
        Ok(())
    }

    pub async fn put_png(&self, key: &str, body: &[u8]) -> Result<()> {
        self.put_object(key, body, "image/png").await
    }
}
