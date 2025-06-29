use std::path::PathBuf;

use config::{Config as ConfigLoader, Environment, File};
use serde::Deserialize;

use crate::core::storage::StorageConfig;
use crate::core::streamer::RtmpStreamerConfig;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub max_file_size: bytesize::ByteSize,
    pub database_url: String,
    pub tmp_dir: String,
    pub storage: StorageConfig,
    pub stream: RtmpStreamerConfig,
}

impl Config {
    pub fn load(config_file: impl Into<PathBuf>) -> anyhow::Result<Self> {
        let config = ConfigLoader::builder()
            .add_source(File::from(config_file.into()).required(false))
            .add_source(Environment::with_prefix("CLIPCHAN").separator("__"))
            .build()?
            .try_deserialize()?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_config() {
        let config = Config::load("clipchan.toml").unwrap();
        assert_eq!(config.port, 3000);
        assert!(!config.database_url.is_empty());
    }
}
