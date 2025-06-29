use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use serde::Deserialize;
use tokio::io::{AsyncRead, AsyncReadExt};

#[derive(Debug, Clone, Deserialize)]
pub struct LocalStorageConfig {
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct LocalStorage {
    pub path: PathBuf,
}

impl LocalStorage {
    pub fn new(cfg: &LocalStorageConfig) -> Result<Self> {
        let path = std::path::PathBuf::from(&cfg.path);
        tracing::debug!(path = ?path, "Initialized local storage");
        std::fs::create_dir_all(&path)
            .with_context(|| format!("Failed to create storage directory: {}", path.display()))?;
        Ok(Self { path })
    }

    pub(crate) async fn copy(&self, from: &PathBuf, dest: String) -> Result<()> {
        let dest_path = self.path.join(dest);
        tracing::trace!(
            "Copying file from {} to {}",
            from.display(),
            dest_path.display()
        );
        tokio::fs::copy(&from, &dest_path).await.with_context(|| {
            format!(
                "Failed to copy file to local storage: {}",
                dest_path.display()
            )
        })?;
        tracing::trace!("File copied successfully to {}", dest_path.display());
        Ok(())
    }

    pub(crate) async fn get(
        &self,
        path: &str,
    ) -> Result<Box<dyn AsyncRead + Unpin + Send + 'static>> {
        let file_path = self.path.join(path);
        tracing::trace!(
            "Retrieving file from local storage: {}",
            file_path.display()
        );
        if !file_path.exists() {
            return Err(anyhow::anyhow!("File not found: {}", file_path.display()));
        }
        let file = tokio::fs::File::open(&file_path)
            .await
            .with_context(|| format!("Failed to open file: {}", file_path.display()))?;
        tracing::trace!("File opened successfully: {}", file_path.display());
        Ok(Box::new(file))
    }

    pub(crate) async fn delete(&self, path: &str) -> Result<()> {
        let file_path = self.path.join(path);
        tracing::trace!("Deleting file from local storage: {}", file_path.display());
        if !file_path.exists() {
            return Ok(());
        }
        tokio::fs::remove_file(&file_path)
            .await
            .with_context(|| format!("Failed to delete file: {}", file_path.display()))?;
        tracing::trace!("File deleted successfully: {}", file_path.display());
        Ok(())
    }

    pub(crate) async fn get_file_size(&self, path: &str) -> Result<u64> {
        let file_path = self.path.join(path);
        tracing::trace!("Getting file size from local storage: {}", file_path.display());
        if !file_path.exists() {
            return Err(anyhow::anyhow!("File not found: {}", file_path.display()));
        }
        let metadata = tokio::fs::metadata(&file_path)
            .await
            .with_context(|| format!("Failed to get file metadata: {}", file_path.display()))?;
        Ok(metadata.len())
    }

    pub(crate) async fn get_range(
        &self,
        path: &str,
        start: u64,
        end: u64,
    ) -> Result<Box<dyn AsyncRead + Unpin + Send + 'static>> {
        let file_path = self.path.join(path);
        tracing::trace!(
            "Retrieving file range [{}-{}] from local storage: {}",
            start,
            end,
            file_path.display()
        );
        if !file_path.exists() {
            return Err(anyhow::anyhow!("File not found: {}", file_path.display()));
        }

        let mut file = tokio::fs::File::open(&file_path)
            .await
            .with_context(|| format!("Failed to open file: {}", file_path.display()))?;

        // 移动到起始位置
        use tokio::io::{AsyncSeekExt, SeekFrom};
        file.seek(SeekFrom::Start(start))
            .await
            .with_context(|| format!("Failed to seek to position {}", start))?;

        // 创建一个限制读取长度的读取器
        let length = end - start + 1;
        let limited_reader = file.take(length);

        tracing::trace!("File range opened successfully: {}", file_path.display());
        Ok(Box::new(limited_reader))
    }
}
