use std::sync::Arc;

use axum::response::IntoResponse;
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
};
use axum_extra::TypedHeader;
use axum_extra::headers::Authorization;
use axum_extra::headers::authorization::Bearer;

use crate::server::AppState;

pub async fn auth(
    State(state): State<Arc<AppState>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    mut req: Request,
    next: Next,
) -> impl IntoResponse {
    let token = auth.token();
    match state.user_svc.get_user_by_token(token).await {
        Ok(Some(user)) => {
            req.extensions_mut().insert(user);
            Ok(next.run(req).await)
        }
        Ok(None) => Err((StatusCode::UNAUTHORIZED, "Session expired".to_string())),
        Err(e) => Err((
            StatusCode::UNAUTHORIZED,
            format!("Authentication failed: {}", e),
        )),
    }
}
