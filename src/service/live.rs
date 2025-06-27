use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::anyhow;
use bilive::wbi::WBI;
use dashmap::DashMap;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::core::entity::user;
use crate::core::storage::Storage;
use crate::core::streamer::{RtmpStreamer, RtmpStreamerConfig};
use crate::service::{ClipService, PlaylistService, UserService};

pub struct LiveService {
    user_svc: Arc<UserService>,
    #[allow(dead_code)]
    clip_svc: Arc<ClipService>,
    playlist_svc: Arc<PlaylistService>,
    storage: Arc<Storage>,
    tasks: DashMap<String, (JoinHandle<()>, Arc<RtmpStreamer>, Arc<AtomicBool>)>,
    config: RtmpStreamerConfig,
    wbi: Arc<Mutex<WBI>>,
}

impl LiveService {
    pub fn new(
        user_svc: Arc<UserService>,
        clip_svc: Arc<ClipService>,
        playlist_svc: Arc<PlaylistService>,
        storage: Arc<Storage>,
        config: RtmpStreamerConfig,
        wbi: Arc<Mutex<WBI>>,
    ) -> Self {
        let tasks = DashMap::new();
        Self {
            user_svc,
            clip_svc,
            playlist_svc,
            storage,
            tasks,
            config,
            wbi,
        }
    }

    pub async fn get_live_areas(
        &self,
        user: &user::Model,
    ) -> anyhow::Result<Vec<bilive::bapi::area::LiveArea>> {
        let session = self.user_svc.get_session_and_refresh(user).await?;
        let live = bilive::live::Live::new(session, self.wbi.clone());
        let areas = live
            .get_all_areas()
            .await
            .map_err(|e| anyhow!("Failed to get live areas: {}", e))?;
        Ok(areas)
    }

    pub async fn start_live(&self, user: &user::Model, area_id: i32) -> anyhow::Result<()> {
        // 检查用户是否有开播权限
        self.user_svc.check_stream_permissions(user).await?;

        let mid = user.mid as u64;
        let session = self.user_svc.get_session_and_refresh(user).await?;
        let live = bilive::live::Live::new(session, self.wbi.clone());
        let room_info = live
            .room_info_by_mid(mid)
            .await
            .map_err(|e| anyhow!("Failed to get room info: {}", e))?;
        if room_info.live_status == 1 {
            anyhow::bail!("Live room is already living");
        }
        let live_info = live
            .start_live(room_info.room_id, area_id)
            .await
            .map_err(|e| anyhow!("Failed to start live: {}", e))?;
        if live_info.change != 1 {
            anyhow::bail!("Failed to start live: {}", live_info.status);
        }
        let rtmp_url = format!("{}{}", live_info.rtmp.addr, live_info.rtmp.code);
        let config = self.config.clone();

        let streamer = Arc::new(RtmpStreamer::new(config, &rtmp_url)?);
        streamer.start().await?;
        let storage = self.storage.clone();
        let playlist_svc = self.playlist_svc.clone();

        let streamer_clone = streamer.clone();
        let user_id = user.id;
        let stopped = Arc::new(AtomicBool::new(false));
        let stopped_clone = stopped.clone();
        let task = tokio::spawn(async move {
            loop {
                let playlists = playlist_svc
                    .get_user_active_playlist(user_id)
                    .await
                    .map_err(|e| {
                        tracing::error!(
                            "Failed to get user active playlists for {}: {}",
                            user_id,
                            e
                        );
                    })
                    .unwrap_or_default();
                if playlists.is_empty() {
                    tracing::warn!("No active playlists found for user {}", user_id);
                    return;
                }
                for playlist in playlists {
                    if !playlist.is_active {
                        continue;
                    }

                    let item_count = playlist_svc
                        .get_playlist_item_count(playlist.id)
                        .await
                        .map_err(|e| {
                            tracing::warn!(
                                "Failed to get playlist item count for {}: {}",
                                playlist.id,
                                e
                            );
                        })
                        .unwrap_or(0);

                    if item_count == 0 {
                        tracing::warn!("No items in playlist {} for user {}", playlist.id, user_id);
                        continue;
                    }

                    for index in 0..item_count {
                        if stopped_clone.load(Ordering::SeqCst) {
                            tracing::info!("Live stream stopped for user {}", user_id);
                            return;
                        }
                        let clip = playlist_svc
                            .get_active_clip_by_position(user_id, playlist.id, index)
                            .await
                            .map_err(async move |e| {
                                tracing::warn!(
                                    "Failed to get playlist items for {}: {}",
                                    playlist.id,
                                    e
                                );
                                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                            })
                            .unwrap_or_default();

                        if let Some(clip) = clip {
                            let clip_uuid = clip.uuid.clone();
                            let clip_title = clip.title.clone();
                            let _clip_vup = clip.vup.clone();
                            match storage.get_file(&format!("{}.mp4", clip_uuid)).await {
                                Ok(file) => {
                                    streamer_clone
                                        .update_title(&clip_title)
                                        .await
                                        .map_err(|e| {
                                            tracing::error!("Failed to update title: {}", e);
                                        })
                                        .ok();
                                    streamer_clone
                                        .push(file)
                                        .await
                                        .map_err(|e| {
                                            tracing::error!(
                                                "Failed to push clip to streamer: {}",
                                                e
                                            );
                                        })
                                        .ok();
                                }
                                Err(e) => {
                                    tracing::warn!("Failed to get file from storage: {}", e);
                                    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                                }
                            }
                        }
                    }
                }
            }
        });
        self.tasks
            .insert(user_id.to_string(), (task, streamer, stopped));
        Ok(())
    }

    pub async fn stop_live(&self, user: &user::Model) -> anyhow::Result<()> {
        if let Some((_, (_task, streamer, stopped))) = self.tasks.remove(&user.id.to_string()) {
            streamer
                .stop()
                .await
                .map_err(|e| tracing::error!("Failed to stop live: {}", e))
                .ok();
            stopped.store(true, Ordering::SeqCst);
        }
        let session = self.user_svc.get_session_and_refresh(user).await?;
        let live = bilive::live::Live::new(session, self.wbi.clone());
        let room_info = self
            .get_room_info(&user)
            .await
            .map_err(|e| anyhow!("Failed to get room info: {}", e))?;
        live.stop_live(room_info.room_id)
            .await
            .map_err(|e| anyhow!("Failed to stop live: {}", e))?;
        Ok(())
    }

    pub async fn get_room_info(
        &self,
        user: &user::Model,
    ) -> anyhow::Result<bilive::bapi::room::RoomInfo> {
        let mid = user.mid as u64;
        let session = self.user_svc.get_session_and_refresh(user).await?;
        let live = bilive::live::Live::new(session, self.wbi.clone());
        let room_info = live
            .room_info_by_mid(mid)
            .await
            .map_err(|e| anyhow!("Failed to get room info: {}", e.to_string()))?;
        Ok(room_info)
    }
}
