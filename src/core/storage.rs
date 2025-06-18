use std::path::PathBuf;

use anyhow::Context;
use serde::Deserialize;
use tokio::io::AsyncRead;

use crate::storage::local::LocalStorageConfig;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StorageConfig {
    Local(LocalStorageConfig),
    #[allow(dead_code)]
    S3(S3StorageConfig),
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct S3StorageConfig {
    pub endpoint: String,
    pub bucket: String,
    pub region: String,
    pub access_key: String,
    pub secret_key: String,
}

#[derive(Clone)]
pub enum Storage {
    Local(crate::storage::local::LocalStorage),
}

impl Storage {
    pub async fn store_file(&self, name: String, file: &PathBuf) -> anyhow::Result<()> {
        match self {
            Storage::Local(backend) => backend
                .copy(file, name)
                .await
                .context("Failed to copy file to local storage"),
        }
    }

    pub async fn get_file(
        &self,
        name: &str,
    ) -> anyhow::Result<Box<dyn AsyncRead + Unpin + Send + 'static>> {
        match self {
            Storage::Local(backend) => backend
                .get(name)
                .await
                .context("Failed to retrieve file from local storage"),
        }
    }
}

impl Storage {
    pub async fn new(cfg: &StorageConfig) -> anyhow::Result<Self> {
        match cfg {
            StorageConfig::Local(cfg) => {
                let backend = crate::storage::local::LocalStorage::new(cfg)?;
                Ok(Self::Local(backend))
            }
            StorageConfig::S3(_) => {
                anyhow::bail!("Unimplemented S3 storage backend");
            }
        }
    }
}
