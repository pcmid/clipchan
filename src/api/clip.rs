use std::io;
use std::pin::pin;
use std::sync::Arc;

use axum::extract::{Multipart, Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use futures_util::TryStreamExt;
use sea_orm::ActiveEnum;
use serde::{Deserialize, Serialize};
use tokio_util::io::StreamReader;

use crate::core::entity::{clip, user};
use crate::server::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipRequest {
    pub title: String,
    #[serde(default)]
    pub vup: String,
    #[serde(default)]
    pub song: String,
}

impl Into<clip::Model> for ClipRequest {
    fn into(self) -> clip::Model {
        clip::Model {
            title: self.title,
            vup: self.vup,
            song: self.song,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipResponse {
    pub uuid: String,
    pub title: String,
    pub vup: String,
    pub song: String,
    pub upload_time: u64,
    pub status: String,
}

impl From<clip::Model> for ClipResponse {
    fn from(clip: clip::Model) -> Self {
        ClipResponse {
            uuid: clip.uuid.to_string(),
            title: clip.title,
            vup: clip.vup,
            song: clip.song,
            upload_time: clip.upload_time.timestamp() as u64,
            status: clip.status.to_value(),
        }
    }
}

pub async fn upload(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<user::Model>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut uuid = None;
    let mut uploaded_file = None;
    let mut req = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
    {
        match field.name().unwrap_or("") {
            "file" => {
                let file_ext = field
                    .file_name()
                    .and_then(|name| name.split('.').last())
                    .unwrap_or("mp4")
                    .to_string();

                let body_with_io_error = field.map_err(io::Error::other);
                let mut uploaded_file_reader = pin!(StreamReader::new(body_with_io_error));

                let file_path = state
                    .clip_svc
                    .save_clip_to_tmp(&mut uploaded_file_reader, &file_ext)
                    .await
                    .map_err(|e| {
                        tracing::error!("Failed to save uploaded file: {}", e,);
                        (StatusCode::BAD_REQUEST, e.to_string())
                    })?;

                uuid = Some(file_path.0);
                uploaded_file = Some(file_path.1);
            }
            "metadata" => {
                let json = field.text().await.map_err(|e| {
                    tracing::error!("Failed to read metadata field: {}", e);
                    (StatusCode::BAD_REQUEST, e.to_string())
                })?;
                let meta: ClipRequest = serde_json::from_str(&json)
                    .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid metadata format".into()))?;
                req = Some(meta);
            }
            _ => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    "Unexpected field in multipart request".into(),
                ));
            }
        }
    }
    if uuid.is_none() || uploaded_file.is_none() || req.is_none() {
        return Err((StatusCode::BAD_REQUEST, "Missing file or metadata".into()));
    }
    let uuid = uuid.unwrap();
    let req = req.unwrap();
    let uploaded_file = uploaded_file.unwrap();

    let clip = clip::Model {
        uuid,
        title: req.title,
        vup: req.vup,
        song: req.song,
        upload_time: chrono::Utc::now(),
        ..Default::default()
    };

    let clip = state
        .clip_svc
        .create_clip(user.id, clip, uploaded_file)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create clip: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    Ok(Json(ClipResponse::from(clip)))
}

pub async fn list_clip(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<user::Model>,
) -> impl IntoResponse {
    match state.clip_svc.list_clips_by_user(user.id).await {
        Ok(clips) => {
            let response = clips
                .into_iter()
                .map(|clip| ClipResponse::from(clip))
                .collect::<Vec<_>>();
            Ok(Json(response))
        }
        Err(e) => {
            tracing::error!("Failed to list clips: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}

pub async fn update_clip(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<user::Model>,
    Path(uuid): Path<uuid::Uuid>,
    Json(request): Json<ClipRequest>,
) -> impl IntoResponse {
    if uuid.is_nil() {
        return Err((StatusCode::BAD_REQUEST, "Invalid UUID".into()));
    }

    let mut clip: clip::Model = request.into();
    clip.uuid = uuid;
    match state.clip_svc.update_clip(user, clip).await {
        Ok(Some(clip)) => Ok(Json(ClipResponse::from(clip))),
        Ok(None) => Err((StatusCode::NOT_FOUND, "Clip not found".into())),
        Err(e) => {
            tracing::error!("Failed to update clip: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}

pub async fn reviewed_clip(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<user::Model>,
    Path(uuid): Path<uuid::Uuid>,
) -> impl IntoResponse {
    if uuid.is_nil() {
        return Err((StatusCode::BAD_REQUEST, "Invalid UUID".into()));
    }
    let clip = clip::Model {
        uuid,
        ..Default::default()
    };

    match state.clip_svc.set_clip_reviewed(user, clip).await {
        Ok(Some(clip)) => Ok(Json(ClipResponse::from(clip))),
        Ok(None) => Err((StatusCode::NOT_FOUND, "Clip not found".into())),
        Err(e) => {
            tracing::error!("Failed to mark clip as reviewed: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}
