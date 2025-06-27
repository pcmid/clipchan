use sea_orm::prelude::*;
use sea_orm::{ActiveValue, IntoActiveModel, Set};
use uuid::Uuid;

use crate::core::entity::{clip, playlist, playlist_item};
use crate::data::PlaylistData;
use crate::service::errors::Error;

pub struct PlaylistService {
    playlist_data: PlaylistData,
}

impl PlaylistService {
    pub fn new(playlist_data: PlaylistData) -> Self {
        Self { playlist_data }
    }

    pub async fn create_playlist(&self, req: playlist::Model) -> anyhow::Result<playlist::Model> {
        let mut playlist = playlist::ActiveModel::new();
        playlist.name = Set(req.name);
        playlist.description = Set(req.description);
        playlist.user_id = Set(req.user_id);
        playlist.is_active = Set(req.is_active);
        self.playlist_data.create_playlist(playlist).await
    }

    pub async fn get_user_playlists(&self, user_id: i64) -> anyhow::Result<Vec<playlist::Model>> {
        self.playlist_data.get_user_playlists(user_id).await
    }

    pub async fn get_playlist(&self, user_id: i64, id: i64) -> anyhow::Result<playlist::Model> {
        let playlist = self
            .playlist_data
            .get_playlist(user_id, id)
            .await?
            .ok_or(Error::NotFound("Playlist not found".to_string()))?;
        Ok(playlist)
    }

    pub async fn update_playlist(
        &self,
        user_id: i64,
        playlist: playlist::Model,
    ) -> anyhow::Result<playlist::Model> {
        let existing_playlist = self.get_playlist(user_id, playlist.id).await?;
        let mut playlist_model = existing_playlist.into_active_model();
        let now: DateTimeWithTimeZone = chrono::Utc::now().into();
        playlist_model.name = Set(playlist.name);
        playlist_model.description = Set(playlist.description);
        playlist_model.updated_at = Set(now);
        self.playlist_data.update_playlist(playlist_model).await
    }

    pub async fn delete_playlist(&self, user_id: i64, id: i64) -> anyhow::Result<()> {
        let playlist_model = self.get_playlist(user_id, id as i64).await?;
        self.playlist_data.delete_playlist(playlist_model).await
    }

    pub async fn set_active_playlist(&self, user_id: i64, id: i64) -> anyhow::Result<()> {
        let playlist = self.get_playlist(user_id, id).await?;
        if playlist.is_active {
            return Ok(());
        }
        let mut model = playlist.into_active_model();
        model.is_active = Set(true);
        self.playlist_data.update_playlist(model).await?;
        Ok(())
    }

    pub async fn unset_active_playlist(&self, user_id: i64, id: i64) -> anyhow::Result<()> {
        let playlist = self.get_playlist(user_id, id).await?;
        if !playlist.is_active {
            return Ok(());
        }
        let mut model = playlist.into_active_model();
        model.is_active = Set(false);
        self.playlist_data.update_playlist(model).await?;
        Ok(())
    }

    pub async fn get_user_active_playlist(
        &self,
        user_id: i64,
    ) -> anyhow::Result<Vec<playlist::Model>> {
        self.playlist_data.get_user_active_playlist(user_id).await
    }

    pub async fn _get_playlist_item_by_clip_uuid(
        &self,
        user_id: i64,
        playlist_id: i64,
        clip_uuid: Uuid,
    ) -> anyhow::Result<playlist_item::Model> {
        self.get_playlist(user_id, playlist_id).await?;
        let item = self
            .playlist_data
            .get_playlist_item_by_clip_uuid(playlist_id, clip_uuid)
            .await?
            .ok_or(Error::NotFound("Playlist Item not found".to_string()))?;
        Ok(item)
    }

    pub async fn get_playlist_item_by_playlist_id(
        &self,
        user_id: i64,
        playlist_id: i64,
    ) -> anyhow::Result<Vec<(playlist_item::Model, clip::Model)>> {
        self.get_playlist(user_id, playlist_id).await?;
        let items = self
            .playlist_data
            .get_playlist_items_with_clips(playlist_id)
            .await?;

        let mut resp = Vec::<(playlist_item::Model, clip::Model)>::with_capacity(items.len());
        for (item, clip_opt) in items {
            if let Some(clip) = clip_opt {
                resp.push((item, clip));
            }
        }
        Ok(resp)
    }

    pub async fn get_playlist_item_count(&self, playlist_id: i64) -> anyhow::Result<i64> {
        self.playlist_data
            .get_playlist_item_count(playlist_id)
            .await
    }

    pub async fn get_active_clip_by_position(
        &self,
        user_id: i64,
        playlist_id: i64,
        position: i64,
    ) -> anyhow::Result<Option<clip::Model>> {
        let playlist = self.get_playlist(user_id, playlist_id).await?;
        if !playlist.is_active {
            return Ok(None);
        }
        self.playlist_data
            .get_clip_by_position(playlist_id, position)
            .await
    }

    pub async fn add_to_playlist(
        &self,
        user_id: i64,
        playlist_id: i64,
        clip_uuid: Uuid,
    ) -> anyhow::Result<()> {
        self.get_playlist(user_id, playlist_id).await?;

        let existing = self
            .playlist_data
            .get_playlist_item_by_clip_uuid(playlist_id, clip_uuid)
            .await?;
        if existing.is_some() {
            return Ok(());
        }

        let max_position = self.playlist_data.get_max_position(playlist_id).await?;

        let now: DateTimeWithTimeZone = chrono::Utc::now().into();
        let item = playlist_item::ActiveModel {
            id: ActiveValue::NotSet,
            playlist_id: Set(playlist_id),
            clip_uuid: Set(clip_uuid),
            position: Set(max_position + 1),
            created_at: Set(now),
        };
        self.playlist_data.add_playlist_item(item).await?;
        Ok(())
    }

    pub async fn remove_from_playlist(
        &self,
        user_id: i64,
        playlist_id: i64,
        clip_uuid: Uuid,
    ) -> anyhow::Result<()> {
        self.get_playlist(user_id, playlist_id).await?;
        self.playlist_data
            .remove_playlist_item_and_reorder(playlist_id, clip_uuid)
            .await
    }

    pub async fn reorder_playlist_item(
        &self,
        user_id: i64,
        playlist_id: i64,
        item_id: i64,
        new_position: i64,
    ) -> anyhow::Result<()> {
        self.get_playlist(user_id, playlist_id).await?;
        self.playlist_data
            .reorder_playlist_item(playlist_id, item_id, new_position)
            .await
    }
}
