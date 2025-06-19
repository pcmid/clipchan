use anyhow::anyhow;
use sea_orm::prelude::*;
use sea_orm::{ActiveValue, IntoActiveModel, Order, QueryOrder, Set, TransactionTrait};
use uuid::Uuid;

use crate::core::entity::{clip, playlist, playlist_item};
use crate::service::errors::Error;

pub struct PlaylistService {
    db: DatabaseConnection,
}

impl PlaylistService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn create_playlist(&self, req: playlist::Model) -> anyhow::Result<playlist::Model> {
        let mut playlist = playlist::ActiveModel::new();
        playlist.name = Set(req.name);
        playlist.description = Set(req.description);
        playlist.user_id = Set(req.user_id);
        playlist.is_active = Set(req.is_active);
        let playlist = playlist.insert(&self.db).await?;
        Ok(playlist)
    }

    pub async fn get_user_playlists(&self, user_id: i64) -> anyhow::Result<Vec<playlist::Model>> {
        let playlists = playlist::Entity::find()
            .filter(playlist::Column::UserId.eq(user_id))
            .order_by(playlist::Column::CreatedAt, Order::Desc)
            .all(&self.db)
            .await?;
        Ok(playlists)
    }

    pub async fn get_playlist(&self, user_id: i64, id: i64) -> anyhow::Result<playlist::Model> {
        let playlist = playlist::Entity::find()
            .filter(playlist::Column::Id.eq(id))
            .filter(playlist::Column::UserId.eq(user_id))
            .one(&self.db)
            .await?
            .ok_or(Error::NotFound("Playlist not found".to_string()))?;
        Ok(playlist)
    }

    pub async fn update_playlist(
        &self,
        user_id: i64,
        playlist: playlist::Model,
    ) -> anyhow::Result<playlist::Model> {
        let mut playlist_model = self
            .get_playlist(user_id, playlist.id)
            .await?
            .into_active_model();
        let now: DateTimeWithTimeZone = chrono::Utc::now().into();
        playlist_model.name = Set(playlist.name);
        playlist_model.description = Set(playlist.description);
        playlist_model.updated_at = Set(now);
        let playlist = playlist_model.update(&self.db).await?;
        Ok(playlist)
    }

    pub async fn delete_playlist(&self, user_id: i64, id: i64) -> anyhow::Result<()> {
        let playlist_model = self.get_playlist(user_id, id as i64).await?;
        playlist_model.delete(&self.db).await?;
        Ok(())
    }

    pub async fn set_active_playlist(&self, user_id: i64, id: i64) -> anyhow::Result<()> {
        let playlist = self.get_playlist(user_id, id).await?;
        if playlist.is_active {
            return Ok(());
        }
        let mut model = playlist.into_active_model();
        model.is_active = Set(true);
        model.update(&self.db).await?;
        Ok(())
    }

    pub async fn unset_active_playlist(&self, user_id: i64, id: i64) -> anyhow::Result<()> {
        let playlist = self.get_playlist(user_id, id).await?;
        if !playlist.is_active {
            return Ok(());
        }
        let mut model = playlist.into_active_model();
        model.is_active = Set(false);
        model.update(&self.db).await?;
        Ok(())
    }

    pub async fn get_user_active_playlist(
        &self,
        user_id: i64,
    ) -> anyhow::Result<Vec<playlist::Model>> {
        let playlists = playlist::Entity::find()
            .filter(playlist::Column::UserId.eq(user_id))
            .filter(playlist::Column::IsActive.eq(true))
            .all(&self.db)
            .await?;
        Ok(playlists)
    }

    pub async fn get_playlist_item_by_clip_uuid(
        &self,
        user_id: i64,
        playlist_id: i64,
        clip_uuid: Uuid,
    ) -> anyhow::Result<playlist_item::Model> {
        self.get_playlist(user_id, playlist_id).await?;
        let item = playlist_item::Entity::find()
            .filter(playlist_item::Column::PlaylistId.eq(playlist_id))
            .filter(playlist_item::Column::ClipUuid.eq(clip_uuid))
            .one(&self.db)
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
        let items = playlist_item::Entity::find()
            .filter(playlist_item::Column::PlaylistId.eq(playlist_id))
            .order_by(playlist_item::Column::Position, Order::Asc)
            .all(&self.db)
            .await?;
        let mut resp = Vec::<(playlist_item::Model, clip::Model)>::with_capacity(items.len());
        for item in items {
            let clip = clip::Entity::find()
                .filter(clip::Column::Uuid.eq(item.clip_uuid))
                .one(&self.db)
                .await?
                .ok_or(Error::NotFound("Clip not found".to_string()))?;
            resp.push((item, clip));
        }
        Ok(resp)
    }

    pub async fn get_playlist_item_count(&self, playlist_id: i64) -> anyhow::Result<i64> {
        let count = playlist_item::Entity::find()
            .filter(playlist_item::Column::PlaylistId.eq(playlist_id))
            .count(&self.db)
            .await?;
        Ok(count as i64)
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
        let item = playlist_item::Entity::find()
            .filter(playlist_item::Column::PlaylistId.eq(playlist_id))
            .filter(playlist_item::Column::Position.eq(position))
            .find_also_related(clip::Entity)
            .one(&self.db)
            .await?;
        Ok(item.map(|(_, clip)| clip).unwrap_or(None))
    }

    pub async fn add_to_playlist(
        &self,
        user_id: i64,
        playlist_id: i64,
        clip_uuid: Uuid,
    ) -> anyhow::Result<()> {
        self.get_playlist(user_id, playlist_id).await?;
        let existing = self
            .get_playlist_item_by_clip_uuid(user_id, playlist_id, clip_uuid)
            .await;
        if let Err(e) = existing {
            if !matches!(e.downcast_ref::<Error>(), Some(Error::NotFound(_))) {
                return Err(anyhow!("Failed to check existing playlist item: {}", e));
            }
        } else {
            return Ok(());
        }

        let max_position = playlist_item::Entity::find()
            .filter(playlist_item::Column::PlaylistId.eq(playlist_id))
            .order_by(playlist_item::Column::Position, Order::Desc)
            .one(&self.db)
            .await?
            .map(|item| item.position)
            .unwrap_or(-1);

        let now: DateTimeWithTimeZone = chrono::Utc::now().into();
        let item = playlist_item::ActiveModel {
            id: ActiveValue::NotSet,
            playlist_id: Set(playlist_id),
            clip_uuid: Set(clip_uuid),
            position: Set(max_position + 1),
            created_at: Set(now),
        };
        item.insert(&self.db).await?;
        Ok(())
    }

    pub async fn remove_from_playlist(
        &self,
        user_id: i64,
        playlist_id: i64,
        clip_uuid: Uuid,
    ) -> anyhow::Result<()> {
        self.get_playlist(user_id, playlist_id).await?;
        let item = self
            .get_playlist_item_by_clip_uuid(user_id, playlist_id, clip_uuid)
            .await?;

        let tx = self.db.begin().await?;
        let item_model = item.into_active_model();
        item_model
            .delete(&tx)
            .await
            .map_err(|e| anyhow!("Failed to delete playlist item: {}", e))?;

        let mut items = playlist_item::Entity::find()
            .filter(playlist_item::Column::PlaylistId.eq(playlist_id))
            .order_by(playlist_item::Column::Position, Order::Asc)
            .all(&tx)
            .await?;

        for (index, item) in items.iter_mut().enumerate() {
            if item.position != index as i64 {
                let mut model = item.clone().into_active_model();
                model.position = Set(index as i64);
                model
                    .update(&tx)
                    .await
                    .map_err(|e| anyhow!("Failed to update playlist item position: {}", e))?;
            }
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn reorder_playlist_item(
        &self,
        user_id: i64,
        playlist_id: i64,
        item_id: i64,
        new_position: i64,
    ) -> anyhow::Result<()> {
        self.get_playlist(user_id, playlist_id).await?;
        let item = playlist_item::Entity::find_by_id(item_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow!("Playlist item not found"))?;

        if item.playlist_id != playlist_id {
            return Err(anyhow!("Playlist item does not belong to this playlist"));
        }

        let tx = self.db.begin().await?;
        let mut items = playlist_item::Entity::find()
            .filter(playlist_item::Column::PlaylistId.eq(playlist_id))
            .order_by(playlist_item::Column::Position, Order::Asc)
            .all(&tx)
            .await?;
        if new_position < 0 || new_position >= items.len() as i64 {
            return Err(anyhow!("Invalid position"));
        }
        let current_position = item.position;

        if current_position == new_position {
            tx.commit().await?;
            return Ok(());
        }

        for item in items.iter_mut() {
            let mut model = item.clone().into_active_model();
            if item.id == item_id {
                model.position = Set(new_position);
                model.update(&tx).await?;
            } else if current_position < new_position
                && item.position > current_position
                && item.position <= new_position
            {
                model.position = Set(item.position - 1);
                model.update(&tx).await?;
            } else if current_position > new_position
                && item.position >= new_position
                && item.position < current_position
            {
                model.position = Set(item.position + 1);
                model.update(&tx).await?;
            }
        }
        tx.commit().await?;
        Ok(())
    }
}
