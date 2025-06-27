use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "user")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub mid: i64,
    pub uname: String,
    #[sea_orm(column_type = "Text")]
    pub session: String,
    #[serde(default)]
    pub is_admin: bool,
    #[serde(default = "default_can_stream")]
    pub can_stream: bool,
    #[serde(default)]
    pub is_disabled: bool,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

fn default_can_stream() -> bool {
    true
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::clip::Entity")]
    Clip,
    #[sea_orm(has_many = "super::playlist::Entity")]
    Playlist,
}

impl Related<super::clip::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Clip.def()
    }
}

impl Related<super::playlist::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Playlist.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
