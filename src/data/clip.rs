use sea_orm::prelude::*;
use sea_orm::{IntoActiveModel, Order, QueryOrder, Set, TransactionTrait};
use uuid::Uuid;

use crate::core::entity::{clip, playlist_item};

#[derive(Clone)]
pub struct ClipData {
    db: DatabaseConnection,
}

impl ClipData {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn create_clip(&self, clip: clip::ActiveModel) -> anyhow::Result<clip::Model> {
        let clip = clip.insert(&self.db).await?;
        Ok(clip)
    }

    pub async fn update_clip(&self, clip: clip::ActiveModel) -> anyhow::Result<clip::Model> {
        let clip = clip.update(&self.db).await?;
        Ok(clip)
    }

    pub async fn list_clips_by_user(&self, user_id: i64) -> anyhow::Result<Vec<clip::Model>> {
        let clips = clip::Entity::find()
            .filter(clip::Column::UserId.eq(user_id))
            .all(&self.db)
            .await?;
        Ok(clips)
    }

    pub(crate) async fn list_all_clips(&self) -> anyhow::Result<Vec<clip::Model>> {
        let clips = clip::Entity::find().all(&self.db).await?;
        Ok(clips)
    }

    pub async fn get_clip_by_uuid(
        &self,
        user_id: i64,
        uuid: Uuid,
    ) -> anyhow::Result<Option<clip::Model>> {
        let clip = clip::Entity::find()
            .filter(clip::Column::Uuid.eq(uuid))
            .filter(clip::Column::UserId.eq(user_id))
            .one(&self.db)
            .await?;
        Ok(clip)
    }

    pub async fn _get_clip_by_id(&self, id: i64) -> anyhow::Result<Option<clip::Model>> {
        let clip = clip::Entity::find()
            .filter(clip::Column::Id.eq(id))
            .one(&self.db)
            .await?;
        Ok(clip)
    }

    pub async fn delete_clip_with_playlist_items(
        &self,
        user_id: i64,
        uuid: Uuid,
    ) -> anyhow::Result<()> {
        let tx = self.db.begin().await?;

        let clip = clip::Entity::find()
            .filter(clip::Column::Uuid.eq(uuid))
            .filter(clip::Column::UserId.eq(user_id))
            .one(&tx)
            .await?;

        let clip = match clip {
            Some(c) => c,
            None => return Ok(()),
        };

        // 获取所有相关的播放列表项
        let playlist_items = playlist_item::Entity::find()
            .filter(playlist_item::Column::ClipUuid.eq(clip.uuid))
            .all(&tx)
            .await?;

        // 删除播放列表项并重新排序
        for item in playlist_items {
            let playlist_id = item.playlist_id;
            let item_model = item.into_active_model();
            item_model.delete(&tx).await?;

            // 重新排序剩余的播放列表项
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

        // 删除clip
        clip.into_active_model().delete(&tx).await?;
        tx.commit().await?;
        Ok(())
    }
}
