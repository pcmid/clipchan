use std::sync::Arc;

use anyhow::{Context, anyhow};
use apalis::prelude::{MemoryStorage, WorkerBuilder, WorkerBuilderExt, WorkerFactoryFn};
use axum::extract::{DefaultBodyLimit, State};
use axum::routing::{delete, get, post};
use axum::{Json, Router, middleware};
use migration::*;
use sea_orm::Database;
use serde::Serialize;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;

use crate::api;
use crate::config::Config;
use crate::core::storage::Storage;
use crate::data::{ClipData, PlaylistData, UserData};
use crate::server::auth;
use crate::service::{clip::process_clip, *};

pub async fn run(config: Config) -> anyhow::Result<()> {
    let host = config.host.clone();
    let port = config.port;
    let server_url = format!("{host}:{port}");

    tracing::info!("Server starting at {server_url}");

    let db = Database::connect(&config.database_url)
        .await
        .context("Failed to connect to local database")?;
    Migrator::up(&db, None)
        .await
        .context("Failed to run database migrations")?;

    let storage = Arc::new(
        Storage::new(&config.storage)
            .await
            .context("Failed to create local storage")?,
    );
    let tmp_dir = std::path::PathBuf::from(&config.tmp_dir);
    std::fs::create_dir_all(&tmp_dir).with_context(|| {
        format!(
            "Failed to create temporary directory: {}",
            tmp_dir.display()
        )
    })?;

    let queue = MemoryStorage::new();

    // Create data layer instances
    let user_data = UserData::new(db.clone());
    let clip_data = ClipData::new(db.clone());
    let playlist_data = PlaylistData::new(db.clone());

    // Create service layer instances with data layer dependencies
    let user_svc = Arc::new(UserService::new(user_data));
    let playlist_svc = Arc::new(PlaylistService::new(playlist_data));
    let clip_svc = Arc::new(ClipService::new(
        tmp_dir,
        clip_data,
        storage.clone(),
        queue.clone(),
    ));

    let wbi = Arc::new(Mutex::new(
        bilive::wbi::WBI::new().await.map_err(|e| anyhow!(e))?,
    ));
    let live_svc = Arc::new(LiveService::new(
        user_svc.clone(),
        clip_svc.clone(),
        playlist_svc.clone(),
        storage.clone(),
        config.stream.clone(),
        wbi.clone(),
    ));

    let worker = WorkerBuilder::new("processer")
        .concurrency(2)
        .data(clip_svc.clone())
        .backend(queue)
        .build_fn(process_clip);

    tokio::spawn(async move {
        worker.run().await;
    });

    let state = Arc::new(AppState {
        clip_svc,
        user_svc,
        playlist_svc,
        live_svc,
        config: config.clone(),
    });
    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any)
        .max_age(std::time::Duration::from_secs(3600));

    #[derive(Clone, Serialize)]
    struct ConfigResp {
        max_file_size: usize,
    }

    let public_routes = Router::new()
        .route(
            "/configs",
            get(async move |State(state): State<Arc<AppState>>| {
                Json(ConfigResp {
                    max_file_size: state.config.max_file_size.as_u64() as usize,
                })
            }),
        )
        .route("/user/login/qrcode", get(api::user::get_login_qrcode))
        .route("/user/login/check", get(api::user::check_bilibili_login));

    let protected_routes = Router::new()
        .route("/clips", get(api::clip::list_clip))
        .route("/upload", post(api::clip::upload))
        .layer(DefaultBodyLimit::max(config.max_file_size.as_u64() as usize))
        .route(
            "/clip/{uuid}",
            post(api::clip::update_clip).delete(api::clip::delete_clip),
        )
        // 用户信息接口
        .route("/user/me", get(api::admin::get_current_user))
        // 管理员接口
        .route_layer(middleware::from_fn_with_state(state.clone(), auth::auth));

    let live_routes = Router::new()
        .route("/playlists", get(api::playlist::list_playlists))
        .route("/playlists", post(api::playlist::create_playlist))
        .route("/playlists/active", get(api::playlist::get_active_playlist))
        .route("/playlists/{id}", get(api::playlist::get_playlist_by_id))
        .route("/playlists/{id}", post(api::playlist::update_playlist))
        .route("/playlists/{id}", delete(api::playlist::delete_playlist))
        .route(
            "/playlists/{id}/active",
            post(api::playlist::set_active_playlist).delete(api::playlist::unset_active_playlist),
        )
        .route(
            "/playlists/{id}/items",
            get(api::playlist::get_playlist_items),
        )
        .route(
            "/playlists/items/add",
            post(api::playlist::add_clip_to_playlist),
        )
        .route(
            "/playlists/items/remove",
            delete(api::playlist::remove_clip_from_playlist),
        )
        .route(
            "/playlists/items/reorder",
            post(api::playlist::reorder_playlist_item),
        )
        .route("/live/areas", get(api::live::get_live_areas))
        .route("/live/start", post(api::live::start_live))
        .route("/live/stop", post(api::live::stop_live))
        .route("/live/status", get(api::live::get_live_status))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::can_stream,
        ))
        .layer(middleware::from_fn_with_state(state.clone(), auth::auth));

    let admin_routes = Router::new()
        .route("/clip/{uuid}/reviewed", post(api::clip::reviewed_clip))
        .route("/admin/users", get(api::admin::list_all_users))
        .route(
            "/admin/users/{user_id}/permissions",
            post(api::admin::update_user_permissions),
        )
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::auth_admin,
        ))
        .layer(middleware::from_fn_with_state(state.clone(), auth::auth));

    // 合并路由
    let app = public_routes
        .merge(protected_routes)
        .merge(live_routes)
        .merge(admin_routes)
        .layer(cors.clone())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&server_url)
        .await
        .context("Listening failed")?;

    axum::serve(listener, app).await.context("Server failed")?;
    Ok(())
}

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) clip_svc: Arc<ClipService>,
    pub(crate) user_svc: Arc<UserService>,
    pub(crate) playlist_svc: Arc<PlaylistService>,
    pub(crate) live_svc: Arc<LiveService>,
    pub(crate) config: Config,
}
