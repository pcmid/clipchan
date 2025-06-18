use sea_orm::prelude::*;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize, DeriveEntityModel)]
#[sea_orm(table_name = "clip")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    pub uuid: Uuid,
    pub title: String,
    pub vup: String,
    pub song: String,
    pub upload_time: chrono::DateTime<chrono::Utc>,
    pub status: Status,
    pub user_id: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(32))")]
pub enum Status {
    #[sea_orm(string_value = "pending")]
    Pending,
    #[sea_orm(string_value = "processing")]
    Processing,
    #[sea_orm(string_value = "failed")]
    Failed,
    #[sea_orm(string_value = "reviewing")]
    Reviewing,
    #[sea_orm(string_value = "reviewed")]
    Reviewed,
}

impl Default for Status {
    fn default() -> Self {
        Status::Pending
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,

    #[sea_orm(has_many = "super::playlist_item::Entity")]
    PlaylistItem,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::playlist_item::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PlaylistItem.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
