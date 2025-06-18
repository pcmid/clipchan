use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};

use crate::core::entity::user;
use crate::server::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartLiveRequest {
    pub area_id: i32,
}

pub async fn get_live_areas(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<user::Model>,
) -> impl IntoResponse {
    match state.live_svc.get_live_areas(&user).await {
        Ok(areas) => Ok(Json(areas)),
        Err(e) => {
            tracing::error!("Failed to get live areas: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}

pub async fn start_live(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<user::Model>,
    Json(req): Json<StartLiveRequest>,
) -> impl IntoResponse {
    match state.live_svc.start_live(&user, req.area_id).await {
        Ok(_) => Ok(Json(())),
        Err(e) => {
            tracing::error!("Failed to start live: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}

pub async fn stop_live(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<user::Model>,
) -> impl IntoResponse {
    match state.live_svc.stop_live(&user).await {
        Ok(_) => Ok(Json(())),
        Err(e) => {
            tracing::error!("Failed to stop live: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}

pub async fn get_live_status(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<user::Model>,
) -> impl IntoResponse {
    match state.live_svc.get_room_info(&user).await {
        Ok(status) => Ok(Json(status)),
        Err(e) => {
            tracing::error!("Failed to get live status: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}
