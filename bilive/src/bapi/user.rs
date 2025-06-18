use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub mid: u64,
    pub uname: String,
    pub userid: String,
    pub sign: String,
    pub birthday: String,
    pub sex: String,
    pub nick_free: bool,
    pub rank: String,
}
