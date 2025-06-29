use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Result, anyhow};
use aws_sdk_s3::Client;
use aws_sdk_s3::config::{Credentials, Region};
use aws_sdk_s3::error::DisplayErrorContext;
use aws_sdk_s3::primitives::ByteStream;
use serde::Deserialize;
use tokio::io::{AsyncRead, AsyncReadExt};

#[derive(Debug, Clone, Deserialize)]
pub struct S3StorageConfig {
    pub endpoint: String,
    pub bucket: String,
    pub region: String,
    pub access_key: String,
    pub secret_key: String,
}

#[derive(Debug, Clone)]
pub struct S3Storage {
    client: Arc<Client>,
    bucket: String,
}

impl S3Storage {
    pub fn new(cfg: &S3StorageConfig) -> Result<Self> {
        let credentials = Credentials::new(
            &cfg.access_key,
            &cfg.secret_key,
            None,
            None,
            "S3Credentials",
        );
        let config = aws_sdk_s3::config::Builder::new()
            .region(Region::new(cfg.region.clone()))
            .credentials_provider(credentials)
            .endpoint_url(cfg.endpoint.clone())
            .behavior_version_latest()
            .build();

        let client = Client::from_conf(config);
        tracing::debug!(bucket = %cfg.bucket, "Initialized S3 storage");

        Ok(Self {
            client: Arc::new(client),
            bucket: cfg.bucket.clone(),
        })
    }

    pub(crate) async fn put_object(&self, from: &PathBuf, dest: String) -> Result<()> {
        let mut file = tokio::fs::File::open(from).await?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).await?;
        let body = ByteStream::from(buf);

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(dest)
            .body(body)
            .send()
            .await
            .map_err(|e| anyhow!("S3 put_object error: {}", DisplayErrorContext(&e)))?;
        Ok(())
    }

    pub(crate) async fn get_object(
        &self,
        path: &str,
    ) -> Result<Box<dyn AsyncRead + Unpin + Send + 'static>> {
        let resp = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(path)
            .send()
            .await
            .map_err(|e| anyhow!("S3 get_object error: {}", DisplayErrorContext(&e)))?;

        let stream = resp.body.into_async_read();
        Ok(Box::new(stream))
    }

    pub(crate) async fn delete(&self, path: &str) -> Result<()> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(path)
            .send()
            .await
            .map_err(|e| anyhow!("S3 delete_object error: {}", DisplayErrorContext(&e)))?;
        Ok(())
    }
}
