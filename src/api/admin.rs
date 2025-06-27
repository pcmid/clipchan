use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};

use crate::core::entity::user;
use crate::server::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: i64,
    pub mid: i64,
    pub uname: String,
    pub is_admin: bool,
    pub can_stream: bool,
    pub is_disabled: bool,
    pub created_at: String,
}

impl From<user::Model> for UserResponse {
    fn from(user: user::Model) -> Self {
        UserResponse {
            id: user.id,
            mid: user.mid,
            uname: user.uname,
            is_admin: user.is_admin,
            can_stream: user.can_stream,
            is_disabled: user.is_disabled,
            created_at: user.created_at.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserPermissionsRequest {
    pub is_admin: Option<bool>,
    pub can_stream: Option<bool>,
    pub is_disabled: Option<bool>,
}

// 管理员获取所有用户列表
pub async fn list_all_users(
    State(state): State<Arc<AppState>>,
    Extension(admin_user): Extension<user::Model>,
) -> impl IntoResponse {
    // 检查管理员权限
    if let Err(e) = state.user_svc.check_admin_permissions(&admin_user).await {
        return Err((StatusCode::FORBIDDEN, e.to_string()));
    }

    match state.user_svc.list_all_users().await {
        Ok(users) => {
            let response = users
                .into_iter()
                .map(UserResponse::from)
                .collect::<Vec<_>>();
            Ok(Json(response))
        }
        Err(e) => {
            tracing::error!("Failed to list all users: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}

// 管理员更新用户权限
pub async fn update_user_permissions(
    State(state): State<Arc<AppState>>,
    Extension(admin_user): Extension<user::Model>,
    Path(user_id): Path<i64>,
    Json(request): Json<UpdateUserPermissionsRequest>,
) -> impl IntoResponse {
    // 检查管理员权限
    if let Err(e) = state.user_svc.check_admin_permissions(&admin_user).await {
        return Err((StatusCode::FORBIDDEN, e.to_string()));
    }

    // 防止管理员修改自己的管理员权限
    if admin_user.id == user_id && request.is_admin == Some(false) {
        return Err((
            StatusCode::BAD_REQUEST,
            "Cannot remove admin privileges from yourself".into(),
        ));
    }

    match state
        .user_svc
        .update_user_permissions(
            user_id,
            request.is_admin,
            request.can_stream,
            request.is_disabled,
        )
        .await
    {
        Ok(updated_user) => Ok(Json(UserResponse::from(updated_user))),
        Err(e) => {
            tracing::error!("Failed to update user permissions: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}

pub async fn get_current_user(Extension(user): Extension<user::Model>) -> impl IntoResponse {
    Json(UserResponse::from(user))
}
