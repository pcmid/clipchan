use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::entity::playlist;
use crate::core::entity::user::Model as UserModel;
use crate::server::AppState;

#[derive(Deserialize)]
pub struct PlaylistItemReq {
    pub playlist_id: i64,
    pub clip_uuid: Uuid,
}

#[derive(Deserialize)]
pub struct ReorderItemReq {
    pub playlist_id: i64,
    pub item_id: i64,
    pub new_position: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistRequest {
    pub name: String,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistResponse {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub is_active: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub item_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistItemResponse {
    pub id: i64,
    pub playlist_id: i64,
    pub clip_uuid: String,
    pub position: i64,
    pub clip_title: String,
    pub clip_vup: String,
}

pub async fn list_playlists(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<UserModel>,
) -> impl IntoResponse {
    match state.playlist_svc.get_user_playlists(user.id).await {
        Ok(playlists) => Ok(Json(playlists)),
        Err(e) => {
            tracing::error!("Failed to list playlists: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}

pub async fn create_playlist(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<UserModel>,
    Json(req): Json<PlaylistRequest>,
) -> impl IntoResponse {
    let req = playlist::Model {
        user_id: user.id,
        name: req.name,
        description: req.description.unwrap_or_default(),
        is_active: req.is_active.unwrap_or_default(),
        ..Default::default()
    };

    match state.playlist_svc.create_playlist(req).await {
        Ok(playlist) => Ok(Json(playlist)),
        Err(e) => {
            tracing::error!("Failed to create playlist: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}

pub async fn get_playlist_by_id(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<UserModel>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    match state.playlist_svc.get_playlist(user.id, id).await {
        Ok(playlist) => Ok(Json(playlist)),
        Err(e) => {
            tracing::error!("Failed to get playlist: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}

pub async fn update_playlist(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<UserModel>,
    Path(id): Path<i64>,
    Json(req): Json<PlaylistRequest>,
) -> impl IntoResponse {
    let req = playlist::Model {
        id,
        user_id: user.id,
        name: req.name,
        description: req.description.unwrap_or_default(),
        is_active: req.is_active.unwrap_or_default(),
        ..Default::default()
    };

    match state.playlist_svc.update_playlist(user.mid, req).await {
        Ok(playlist) => Ok(Json(playlist)),
        Err(e) => {
            tracing::error!("Failed to update playlist: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}

pub async fn delete_playlist(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<UserModel>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    match state.playlist_svc.delete_playlist(user.id, id).await {
        Ok(_) => Ok(Json(())),
        Err(e) => {
            tracing::error!("Failed to delete playlist: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}

pub async fn set_active_playlist(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<UserModel>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    match state.playlist_svc.set_active_playlist(user.id, id).await {
        Ok(_) => Ok(Json(())),
        Err(e) => {
            tracing::error!("Failed to set active playlist: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}

pub async fn unset_active_playlist(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<UserModel>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    match state.playlist_svc.unset_active_playlist(user.id, id).await {
        Ok(_) => Ok(Json(())),
        Err(e) => {
            tracing::error!("Failed to set active playlist: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}

pub async fn get_active_playlist(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<UserModel>,
) -> impl IntoResponse {
    match state.playlist_svc.get_user_active_playlist(user.mid).await {
        Ok(playlist) => Ok(Json(playlist)),
        Err(e) => {
            tracing::error!("Failed to get active playlist: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}

pub async fn add_clip_to_playlist(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<UserModel>,
    Json(req): Json<PlaylistItemReq>,
) -> impl IntoResponse {
    match state
        .playlist_svc
        .add_to_playlist(user.id, req.playlist_id, req.clip_uuid)
        .await
    {
        Ok(_) => Ok(Json(())),
        Err(e) => {
            tracing::error!("Failed to add clip to playlist: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}

pub async fn remove_clip_from_playlist(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<UserModel>,
    Json(req): Json<PlaylistItemReq>,
) -> impl IntoResponse {
    match state
        .playlist_svc
        .remove_from_playlist(user.id, req.playlist_id, req.clip_uuid)
        .await
    {
        Ok(_) => Ok(Json(())),
        Err(e) => {
            tracing::error!("Failed to remove clip from playlist: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}

pub async fn get_playlist_items(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<UserModel>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    match state
        .playlist_svc
        .get_playlist_item_by_playlist_id(user.id, id)
        .await
    {
        Ok(items) => {
            let mut resp = Vec::with_capacity(items.len());
            for (item, clip) in &items {
                resp.push(PlaylistItemResponse {
                    id: item.id,
                    playlist_id: item.playlist_id,
                    clip_uuid: item.clip_uuid.to_string(),
                    position: item.position,
                    clip_title: clip.title.clone(),
                    clip_vup: clip.vup.clone(),
                });
            }
            Ok(Json(resp))
        }
        Err(e) => {
            tracing::error!("Failed to get playlist items: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}

pub async fn reorder_playlist_item(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<UserModel>,
    Json(req): Json<ReorderItemReq>,
) -> impl IntoResponse {
    match state
        .playlist_svc
        .reorder_playlist_item(user.id, req.playlist_id, req.item_id, req.new_position)
        .await
    {
        Ok(_) => Ok(Json(())),
        Err(e) => {
            tracing::error!("Failed to reorder playlist item: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}
