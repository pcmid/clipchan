use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "playlist_item")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub playlist_id: i64,
    pub clip_uuid: Uuid,
    pub position: i64,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::playlist::Entity",
        from = "Column::PlaylistId",
        to = "super::playlist::Column::Id"
    )]
    Playlist,

    #[sea_orm(
        belongs_to = "super::clip::Entity",
        from = "Column::ClipUuid",
        to = "super::clip::Column::Uuid"
    )]
    Clip,
}

impl Related<super::playlist::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Playlist.def()
    }
}

impl Related<super::clip::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Clip.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
