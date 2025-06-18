use std::collections::BTreeMap;
use std::error::Error;
use std::sync::Arc;
use std::time::SystemTime;

use md5::{Digest, Md5};
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use serde_json::Value;

use crate::client::Client;

const WBI_CACHE_DURATION: u64 = 12 * 60 * 60;

const MIXIN_KEY_ENC_TAB: [u8; 64] = [
    46, 47, 18, 2, 53, 8, 23, 32, 15, 50, 10, 31, 58, 3, 45, 35, 27, 43, 5, 49, 33, 9, 42, 19, 29,
    28, 14, 39, 12, 38, 41, 13, 37, 48, 7, 16, 24, 55, 40, 61, 26, 17, 0, 1, 60, 51, 30, 4, 22, 25,
    54, 21, 56, 59, 6, 63, 57, 62, 11, 36, 20, 34, 44, 52,
];

#[derive(Clone)]
pub struct WBI {
    client: Arc<Client>,
    last_modified: SystemTime,
    mixin_key: String,
}

impl WBI {
    pub async fn new() -> Result<Self, Box<dyn Error + Sync + Send>> {
        let client = Arc::new(Client::new(None)?);

        match Self::fetch_wbi_keys(client.clone()).await {
            Ok((img_key, sub_key)) => {
                let raw_wbi_key = format!("{}{}", img_key, sub_key);
                let mixin_key = gen_mixin_key(raw_wbi_key.as_bytes());
                tracing::debug!("WBI keys fetched: img_key={}, sub_key={}", img_key, sub_key);
                Ok(Self {
                    client,
                    last_modified: SystemTime::now(),
                    mixin_key,
                })
            }
            Err(e) => Err(e),
        }
    }

    pub async fn refresh(&mut self) -> Result<(), Box<dyn Error + Sync + Send>> {
        match Self::fetch_wbi_keys(self.client.clone()).await {
            Ok((img_key, sub_key)) => {
                let raw_wbi_key = format!("{}{}", img_key, sub_key);
                let mixin_key = gen_mixin_key(raw_wbi_key.as_bytes());
                tracing::debug!(
                    "WBI keys refreshed: img_key={}, sub_key={}",
                    img_key,
                    sub_key
                );
                self.last_modified = SystemTime::now();
                self.mixin_key = mixin_key;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    pub fn is_expired(&self) -> bool {
        SystemTime::now()
            .duration_since(self.last_modified)
            .map_or(false, |t| t.as_secs() > WBI_CACHE_DURATION)
    }

    async fn fetch_wbi_keys(
        client: Arc<Client>,
    ) -> Result<(String, String), Box<dyn Error + Sync + Send>> {
        let nav_data: Value = client
            .get("https://api.bilibili.com/x/web-interface/nav")
            .send()
            .await?
            .json()
            .await?;

        let wbi_img = nav_data
            .get("data")
            .and_then(|d| d.get("wbi_img"))
            .ok_or_else(|| "Missing wbi_img in nav response")?;

        let img_url = wbi_img
            .get("img_url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing img_url in wbi_img")?;

        let sub_url = wbi_img
            .get("sub_url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing sub_url in wbi_img")?;

        let img_key = img_url
            .split('/')
            .last()
            .unwrap_or("")
            .split('.')
            .next()
            .unwrap_or("");
        let sub_key = sub_url
            .split('/')
            .last()
            .unwrap_or("")
            .split('.')
            .next()
            .unwrap_or("");

        Ok((img_key.to_string(), sub_key.to_string()))
    }

    pub fn sign(
        &self,
        params: &BTreeMap<&str, String>,
    ) -> Result<String, Box<dyn Error + Sync + Send>> {
        if self.is_expired() {
            return Err("WBI keys are expired, please refresh".into());
        }
        Ok(calculate_w_rid(params, &self.mixin_key))
    }

    pub async fn sign_with_latest(
        &mut self,
        params: &BTreeMap<&str, String>,
    ) -> Result<String, Box<dyn Error + Sync + Send>> {
        match self.sign(params) {
            Ok(res) => Ok(res),
            Err(e) => {
                tracing::debug!("Failed to sign with WBI: {}, try refreshing WBI keys", e);
                if let Err(e) = self.refresh().await {
                    tracing::error!("Failed to refresh WBI keys: {}", e);
                    return Err(e);
                }
                self.sign(params)
            }
        }
    }
}

fn url_encode(s: &str) -> String {
    utf8_percent_encode(s, NON_ALPHANUMERIC)
        .to_string()
        .replace('+', "%20")
}

fn gen_mixin_key(raw_wbi_key: impl AsRef<[u8]>) -> String {
    let raw_wbi_key = raw_wbi_key.as_ref();
    let mut mixin_key = {
        let binding = MIXIN_KEY_ENC_TAB
            .iter()
            .map(|n| raw_wbi_key[*n as usize])
            .collect::<Vec<u8>>();
        unsafe { String::from_utf8_unchecked(binding) }
    };
    let _ = mixin_key.split_off(32); // 截取前 32 位字符
    mixin_key
}

fn calculate_w_rid(params: &BTreeMap<&str, String>, mixin_key: &str) -> String {
    // Sort parameters by key and encode values
    let encoded_params: Vec<String> = params
        .iter()
        .map(|(k, v)| format!("{}={}", k, url_encode(v)))
        .collect();

    // Join parameters with &
    let param_string = encoded_params.join("&");

    // Append mixin_key
    let string_to_hash = format!("{}{}", param_string, mixin_key);

    // Calculate MD5
    let mut hasher = Md5::new();
    hasher.update(string_to_hash.as_bytes());
    format!("{:x}", hasher.finalize())
}
