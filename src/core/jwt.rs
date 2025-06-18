use axum::http::StatusCode;
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

// JWT密钥，实际生产环境应该从环境变量或配置文件中获取
// 这里仅作为示例使用了硬编码的密钥
static SECRET_KEY: &[u8] = b"YOUR_SECRET_KEY_HERE";

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub mid: i64,
    pub uname: String,
    pub exp: i64,
    pub iat: i64,
}

#[derive(Debug)]
pub enum JwtError {
    InvalidToken,
    TokenCreation,
    TokenExpired,
}

impl std::fmt::Display for JwtError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JwtError::InvalidToken => write!(f, "Invalid JWT token"),
            JwtError::TokenCreation => write!(f, "Failed to create JWT token"),
            JwtError::TokenExpired => write!(f, "JWT token has expired"),
        }
    }
}

impl std::error::Error for JwtError {}

impl From<JwtError> for StatusCode {
    fn from(error: JwtError) -> Self {
        match error {
            JwtError::InvalidToken => StatusCode::UNAUTHORIZED,
            JwtError::TokenCreation => StatusCode::INTERNAL_SERVER_ERROR,
            JwtError::TokenExpired => StatusCode::UNAUTHORIZED,
        }
    }
}

impl From<jsonwebtoken::errors::Error> for JwtError {
    fn from(error: jsonwebtoken::errors::Error) -> Self {
        match error.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => JwtError::TokenExpired,
            _ => JwtError::InvalidToken,
        }
    }
}

pub fn create_token(mid: i64, uname: String, expire_days: i64) -> Result<String, JwtError> {
    let now = Utc::now();
    let expire_time = now + Duration::days(expire_days);

    let claims = Claims {
        mid,
        uname,
        exp: expire_time.timestamp(),
        iat: now.timestamp(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(SECRET_KEY),
    )
    .map_err(|_| JwtError::TokenCreation)
}

pub fn verify_token(token: &str) -> Result<Claims, JwtError> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(SECRET_KEY),
        &Validation::default(),
    )?;

    Ok(token_data.claims)
}
