use std::sync::Arc;

use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use bilive::bapi::Account;
use qrcode::QrCode;
use serde::{Deserialize, Serialize};

use crate::server::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrCodeInfo {
    pub qrcode_key: String,
    pub svg: String,
}

pub async fn get_login_qrcode(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.user_svc.get_login_qrcode().await {
        Ok(qrcode_info) => Ok(QrCode::new(qrcode_info.url)
            .map_err(|e| {
                tracing::error!("Failed to generate QR code: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            })
            .and_then(|qrcode| {
                let image = qrcode
                    .render::<qrcode::render::svg::Color>()
                    .min_dimensions(200, 200)
                    .build();
                Ok(Json(QrCodeInfo {
                    qrcode_key: qrcode_info.qrcode_key.clone(),
                    svg: image,
                }))
            })),
        Err(e) => {
            tracing::error!("Failed to get login QR code: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}

#[derive(Deserialize)]
pub struct CheckLoginQuery {
    qrcode_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoginState {
    NotLoggedIn,
    LoggedIn(LoginInfo),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginInfo {
    pub account: Account,
    pub token: String,
}

pub async fn check_bilibili_login(
    State(state): State<Arc<AppState>>,
    Query(params): Query<CheckLoginQuery>,
) -> impl IntoResponse {
    match state
        .user_svc
        .check_bilibili_login(&params.qrcode_key)
        .await
    {
        Ok(user) => match user {
            None => Ok(Json(LoginState::NotLoggedIn)),
            Some(account) => {
                let token = state
                    .user_svc
                    .generate_token_for_user(account.mid as i64)
                    .await
                    .unwrap();
                Ok(Json(LoginState::LoggedIn(LoginInfo { account, token })))
            }
        },
        Err(e) => {
            tracing::error!("Failed to check login status: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}
