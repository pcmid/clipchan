use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenInfo {
    pub expires_in: u64,
    pub mid: u64,
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoginInfo {
    pub url: String,
    pub refresh_token: String,
    pub timestamp: u64,
    pub code: i32,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct QrCodeInfo {
    pub url: String,
    pub qrcode_key: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OAuthInfo {
    pub mid: u64,
    pub access_token: String,
    pub expires_in: u32,
    pub refresh: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RefreshInfo {
    pub refresh: bool,
    pub timestamp: i64,
}
