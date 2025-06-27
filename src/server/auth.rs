use std::sync::Arc;

use axum::{
    Extension,
    extract::{Request, State},
    http::{StatusCode, header::AUTHORIZATION},
    middleware::Next,
    response::Response,
};

use crate::core::entity::user;
use crate::server::AppState;

pub async fn auth(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => &header[7..],
        _ => return Err(StatusCode::UNAUTHORIZED),
    };

    match state.user_svc.get_user_by_token(token).await {
        Ok(Some(user)) => {
            // 检查用户是否被禁用
            if let Err(_) = state.user_svc.check_user_permissions(&user).await {
                return Err(StatusCode::FORBIDDEN);
            }

            // 将用户信息添加到请求扩展中
            request.extensions_mut().insert(user);
            Ok(next.run(request).await)
        }
        Ok(None) => Err(StatusCode::UNAUTHORIZED),
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}

pub async fn auth_admin(
    Extension(user): Extension<user::Model>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    match user.is_admin {
        true => Ok(next.run(request).await),
        false => Err(StatusCode::FORBIDDEN),
    }
}

pub async fn can_stream(
    Extension(user): Extension<user::Model>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    match user.can_stream {
        true => Ok(next.run(request).await),
        false => Err(StatusCode::FORBIDDEN),
    }
}
