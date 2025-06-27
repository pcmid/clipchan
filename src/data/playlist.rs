use anyhow::anyhow;
use sea_orm::prelude::*;
use sea_orm::{IntoActiveModel, Order, QueryOrder, Set, TransactionTrait};
use uuid::Uuid;

use crate::core::entity::{clip, playlist, playlist_item};

#[derive(Clone)]
pub struct PlaylistData {
    db: DatabaseConnection,
}

impl PlaylistData {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn create_playlist(
        &self,
        playlist: playlist::ActiveModel,
    ) -> anyhow::Result<playlist::Model> {
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

    pub async fn get_playlist(
        &self,
        user_id: i64,
        id: i64,
    ) -> anyhow::Result<Option<playlist::Model>> {
        let playlist = playlist::Entity::find()
            .filter(playlist::Column::Id.eq(id))
            .filter(playlist::Column::UserId.eq(user_id))
            .one(&self.db)
            .await?;
        Ok(playlist)
    }

    pub async fn update_playlist(
        &self,
        playlist: playlist::ActiveModel,
    ) -> anyhow::Result<playlist::Model> {
        let playlist = playlist.update(&self.db).await?;
        Ok(playlist)
    }

    pub async fn delete_playlist(&self, playlist: playlist::Model) -> anyhow::Result<()> {
        playlist.delete(&self.db).await?;
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
        playlist_id: i64,
        clip_uuid: Uuid,
    ) -> anyhow::Result<Option<playlist_item::Model>> {
        let item = playlist_item::Entity::find()
            .filter(playlist_item::Column::PlaylistId.eq(playlist_id))
            .filter(playlist_item::Column::ClipUuid.eq(clip_uuid))
            .one(&self.db)
            .await?;
        Ok(item)
    }

    pub async fn get_playlist_items_with_clips(
        &self,
        playlist_id: i64,
    ) -> anyhow::Result<Vec<(playlist_item::Model, Option<clip::Model>)>> {
        let items = playlist_item::Entity::find()
            .filter(playlist_item::Column::PlaylistId.eq(playlist_id))
            .order_by(playlist_item::Column::Position, Order::Asc)
            .find_also_related(clip::Entity)
            .all(&self.db)
            .await?;
        Ok(items)
    }

    pub async fn get_playlist_item_count(&self, playlist_id: i64) -> anyhow::Result<i64> {
        let count = playlist_item::Entity::find()
            .filter(playlist_item::Column::PlaylistId.eq(playlist_id))
            .count(&self.db)
            .await?;
        Ok(count as i64)
    }

    pub async fn get_clip_by_position(
        &self,
        playlist_id: i64,
        position: i64,
    ) -> anyhow::Result<Option<clip::Model>> {
        let item = playlist_item::Entity::find()
            .filter(playlist_item::Column::PlaylistId.eq(playlist_id))
            .filter(playlist_item::Column::Position.eq(position))
            .find_also_related(clip::Entity)
            .one(&self.db)
            .await?;
        Ok(item.and_then(|(_, clip)| clip))
    }

    pub async fn get_max_position(&self, playlist_id: i64) -> anyhow::Result<i64> {
        let max_position = playlist_item::Entity::find()
            .filter(playlist_item::Column::PlaylistId.eq(playlist_id))
            .order_by(playlist_item::Column::Position, Order::Desc)
            .one(&self.db)
            .await?
            .map(|item| item.position)
            .unwrap_or(-1);
        Ok(max_position)
    }

    pub async fn add_playlist_item(
        &self,
        item: playlist_item::ActiveModel,
    ) -> anyhow::Result<playlist_item::Model> {
        let item = item.insert(&self.db).await?;
        Ok(item)
    }

    pub async fn remove_playlist_item_and_reorder(
        &self,
        playlist_id: i64,
        clip_uuid: Uuid,
    ) -> anyhow::Result<()> {
        let tx = self.db.begin().await?;

        let item = playlist_item::Entity::find()
            .filter(playlist_item::Column::PlaylistId.eq(playlist_id))
            .filter(playlist_item::Column::ClipUuid.eq(clip_uuid))
            .one(&tx)
            .await?;

        if let Some(item) = item {
            let item_model = item.into_active_model();
            item_model.delete(&tx).await?;

            // 重新排序
            let mut items = playlist_item::Entity::find()
                .filter(playlist_item::Column::PlaylistId.eq(playlist_id))
                .order_by(playlist_item::Column::Position, Order::Asc)
                .all(&tx)
                .await?;

            for (index, item) in items.iter_mut().enumerate() {
                if item.position != index as i64 {
                    let mut model = item.clone().into_active_model();
                    model.position = Set(index as i64);
                    model.update(&tx).await?;
                }
            }
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn reorder_playlist_item(
        &self,
        playlist_id: i64,
        item_id: i64,
        new_position: i64,
    ) -> anyhow::Result<()> {
        let tx = self.db.begin().await?;

        let item = playlist_item::Entity::find_by_id(item_id)
            .one(&tx)
            .await?
            .ok_or_else(|| anyhow!("Playlist item not found"))?;

        if item.playlist_id != playlist_id {
            return Err(anyhow!("Playlist item does not belong to this playlist"));
        }

        let mut items = playlist_item::Entity::find()
            .filter(playlist_item::Column::PlaylistId.eq(playlist_id))
            .order_by(playlist_item::Column::Position, Order::Asc)
            .all(&tx)
            .await?;

        if new_position < 0 || new_position >= items.len() as i64 {
            return Err(anyhow!("Invalid position"));
        }

        let current_position = item.position;

        if current_position != new_position {
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
        }

        tx.commit().await?;
        Ok(())
    }
}
