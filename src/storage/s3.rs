// use crate::storage::StorageBackend;
// use anyhow::Result;
// use std::path::PathBuf;
//
// pub struct S3Storage {}
//
// impl S3Storage {
//     pub fn new() -> Result<Self> {
//         anyhow::bail!("Unimplemented S3 storage backend");
//     }
// }
//
// #[async_trait::async_trait]
// impl StorageBackend for S3Storage {
//     async fn init(&mut self) -> Result<()> {
//         tracing::debug!("S3 storage is available");
//         Ok(())
//     }
//
//     async fn copy(&mut self, from: &PathBuf, dest: String) -> Result<()> {
//         todo!()
//     }
// }
