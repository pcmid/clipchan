use std::io;
use std::pin::pin;
use std::sync::Arc;

use axum::extract::{Multipart, Path, Query, State};
use axum::http::{StatusCode, HeaderMap, header};
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use axum::body::Body;
use futures_util::TryStreamExt;
use sea_orm::ActiveEnum;
use serde::{Deserialize, Serialize};
use tokio_util::io::ReaderStream;
use tokio_util::io::StreamReader;

use crate::core::entity::{clip, user};
use crate::core::jwt;
use crate::core::jwt::DEFAULT_SECRET_KEY;
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
    match state.clip_svc.list_clips_by_user(&user).await {
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
    match state.clip_svc.update_clip(&user, clip).await {
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

    match state.clip_svc.set_clip_reviewed(&user, uuid).await {
        Ok(Some(clip)) => Ok(Json(ClipResponse::from(clip))),
        Ok(None) => Err((StatusCode::NOT_FOUND, "Clip not found".into())),
        Err(e) => {
            tracing::error!("Failed to mark clip as reviewed: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}

pub async fn delete_clip(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<user::Model>,
    Path(uuid): Path<uuid::Uuid>,
) -> impl IntoResponse {
    if uuid.is_nil() {
        return Err((StatusCode::BAD_REQUEST, "Invalid UUID".into()));
    }

    match state.clip_svc.delete_clip(&user, uuid).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            tracing::error!("Failed to delete clip: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}

#[derive(Deserialize)]
pub struct RangeQuery {
    token: Option<String>,
}

pub async fn preview_clip(
    State(state): State<Arc<AppState>>,
    Path(uuid): Path<uuid::Uuid>,
    Query(query): Query<RangeQuery>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if uuid.is_nil() {
        return Err((StatusCode::BAD_REQUEST, "Invalid UUID".to_string()));
    }

    if let Some(token) = query.token {
        // Get JWT secret from config
        let jwt_secret = state.config.jwt_secret.as_deref().unwrap_or(DEFAULT_SECRET_KEY);
        match jwt::verify_token(&token, jwt_secret) {
            Ok(claims) => {
                tracing::trace!("Token validated for user: {} ({})", claims.uname, claims.mid);
            }
            Err(_) => {
                return Err((StatusCode::FORBIDDEN, "Token expired".to_string()));
            }
        }
    } else {
        return Err((StatusCode::FORBIDDEN, "Forbidden".to_string()));
    }

    let range_header = headers.get(header::RANGE);

    match state.clip_svc.get_clip_stream_with_range(uuid, range_header).await {
        Ok((stream, file_size, range_info)) => {
            let mut response_headers = HeaderMap::new();
            response_headers.insert(
                header::CONTENT_TYPE,
                "video/mp4".parse().unwrap(),
            );
            response_headers.insert(
                header::ACCEPT_RANGES,
                "bytes".parse().unwrap(),
            );

            let (status, content_length, _content_range) = match range_info {
                Some((start, end)) => {
                    response_headers.insert(
                        header::CONTENT_RANGE,
                        format!("bytes {}-{}/{}", start, end, file_size).parse().unwrap(),
                    );
                    (StatusCode::PARTIAL_CONTENT, end - start + 1, Some((start, end)))
                },
                None => {
                    (StatusCode::OK, file_size, None)
                }
            };

            response_headers.insert(
                header::CONTENT_LENGTH,
                content_length.to_string().parse().unwrap(),
            );

            response_headers.insert(
                header::ACCESS_CONTROL_ALLOW_ORIGIN,
                "*".parse().unwrap(),
            );
            response_headers.insert(
                header::ACCESS_CONTROL_ALLOW_HEADERS,
                "Range".parse().unwrap(),
            );

            let body = Body::from_stream(ReaderStream::new(stream));
            let response = Response::builder()
                .status(status)
                .body(body)
                .unwrap();

            Ok((response_headers, response))
        }
        Err(e) => {
            tracing::error!("Failed to get clip stream for UUID {}: {}", uuid, e);
            Err((StatusCode::NOT_FOUND, "Clip not found or not accessible".to_string()))
        }
    }
}
