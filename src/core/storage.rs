use std::path::PathBuf;

use anyhow::Context;
use serde::Deserialize;
use tokio::io::AsyncRead;

use crate::storage::local::LocalStorageConfig;
use crate::storage::s3::S3StorageConfig;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct StorageConfig {
    #[serde(rename = "type")]
    pub _type: String, // "local" or "s3"
    pub local: Option<LocalStorageConfig>,
    pub s3: Option<S3StorageConfig>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StorageBackendConfig {
    Local(LocalStorageConfig),
    S3(S3StorageConfig),
}

#[derive(Clone)]
pub enum Storage {
    Local(crate::storage::local::LocalStorage),
    S3(crate::storage::s3::S3Storage),
}

impl Storage {
    pub async fn new(cfg: &StorageConfig) -> anyhow::Result<Self> {
        match cfg._type.as_str() {
            "local" => {
                if let Some(ref local_cfg) = cfg.local {
                    let backend = crate::storage::local::LocalStorage::new(local_cfg)?;
                    Ok(Self::Local(backend))
                } else {
                    Err(anyhow::anyhow!("Invalid configuration for local storage"))
                }
            }
            "s3" => {
                if let Some(ref s3_cfg) = cfg.s3 {
                    let backend = crate::storage::s3::S3Storage::new(s3_cfg)?;
                    Ok(Self::S3(backend))
                } else {
                    Err(anyhow::anyhow!("Invalid configuration for S3 storage"))
                }
            }
            _ => Err(anyhow::anyhow!("Unsupported storage type: {}", cfg._type)),
        }
    }

    pub async fn store_file(&self, name: String, file: &PathBuf) -> anyhow::Result<()> {
        match self {
            Storage::Local(backend) => backend
                .copy(file, name)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to copy file to local storage: {}", e)),
            Storage::S3(backend) => backend
                .put_object(file, name)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to upload file to S3 storage: {}", e)),
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
            Storage::S3(backend) => backend
                .get_object(name)
                .await
                .context("Failed to retrieve file from S3 storage"),
        }
    }

    pub async fn get_file_size(&self, name: &str) -> anyhow::Result<u64> {
        match self {
            Storage::Local(backend) => backend
                .get_file_size(name)
                .await
                .context("Failed to get file size from local storage"),
            Storage::S3(backend) => backend
                .get_file_size(name)
                .await
                .context("Failed to get file size from S3 storage"),
        }
    }

    pub async fn get_file_range(
        &self,
        name: &str,
        start: u64,
        end: u64,
    ) -> anyhow::Result<Box<dyn AsyncRead + Unpin + Send + 'static>> {
        match self {
            Storage::Local(backend) => backend
                .get_range(name, start, end)
                .await
                .context("Failed to get file range from local storage"),
            Storage::S3(backend) => backend
                .get_object_range(name, start, end)
                .await
                .context("Failed to get file range from S3 storage"),
        }
    }

    pub async fn delete_file(&self, name: &str) -> anyhow::Result<()> {
        match self {
            Storage::Local(backend) => backend
                .delete(&name)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to delete file from local storage: {}", e)),
            Storage::S3(backend) => backend
                .delete(&name)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to delete file from S3 storage: {}", e)),
        }
    }
}
