use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::bapi::login::*;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResponseData<V> {
    pub code: i64,
    pub message: String,
    pub data: Option<V>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ResponseValue {
    Login(LoginInfo),
    OAuth(OAuthInfo),
    Value(Value),
}
