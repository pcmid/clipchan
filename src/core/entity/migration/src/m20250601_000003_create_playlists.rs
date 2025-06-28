use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Playlist::Table)
                    .if_not_exists()
                    .col(pk_auto(Playlist::Id))
                    .col(string(Playlist::Name).not_null())
                    .col(string(Playlist::Description).not_null())
                    .col(big_integer(Playlist::UserId).not_null())
                    .col(boolean(Playlist::IsActive).not_null().default(false))
                    .col(
                        timestamp(Playlist::CreatedAt)
                            .not_null()
                            .default(Expr::cust("CURRENT_TIMESTAMP")),
                    )
                    .col(
                        timestamp(Playlist::UpdatedAt)
                            .not_null()
                            .default(Expr::cust("CURRENT_TIMESTAMP")),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(PlaylistItem::Table)
                    .if_not_exists()
                    .col(pk_auto(PlaylistItem::Id))
                    .col(integer(PlaylistItem::PlaylistId).not_null())
                    .col(uuid(PlaylistItem::ClipUuid).not_null())
                    .col(integer(PlaylistItem::Position).not_null())
                    .col(
                        timestamp(PlaylistItem::CreatedAt)
                            .not_null()
                            .default(Expr::cust("CURRENT_TIMESTAMP")),
                    )
                    .foreign_key(
                        &mut ForeignKey::create()
                            .name("fk_playlist_item_playlist")
                            .from(PlaylistItem::Table, PlaylistItem::PlaylistId)
                            .to(Playlist::Table, Playlist::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .to_owned(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Playlist::Table)
                    .name("idx_playlist_user_mid")
                    .col(Playlist::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Playlist::Table)
                    .name("idx_playlist_user_active")
                    .col(Playlist::UserId)
                    .col(Playlist::IsActive)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(PlaylistItem::Table)
                    .name("idx_playlist_item_playlist_id")
                    .col(PlaylistItem::PlaylistId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(PlaylistItem::Table)
                    .name("idx_playlist_item_clip")
                    .col(PlaylistItem::ClipUuid)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(PlaylistItem::Table)
                    .name("idx_playlist_item_position")
                    .col(PlaylistItem::PlaylistId)
                    .col(PlaylistItem::Position)
                    .to_owned(),
            )
            .await?;

        // // 添加外键约束
        // manager
        //     .create_foreign_key(
        //         ForeignKey::create()
        //             .name("fk_playlist_item_playlist")
        //             .from(PlaylistItem::Table, PlaylistItem::PlaylistId)
        //             .to(Playlist::Table, Playlist::Id)
        //             .on_delete(ForeignKeyAction::Cascade)
        //             .to_owned(),
        //     )
        //     .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(PlaylistItem::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(Playlist::Table).if_exists().to_owned())
            .await?;

        Ok(())
    }
}

// 播放列表表的列定义
#[derive(DeriveIden)]
enum Playlist {
    Table,
    Id,
    Name,
    Description,
    UserId,
    IsActive,
    CreatedAt,
    UpdatedAt,
}

// 播放列表项表的列定义
#[derive(DeriveIden)]
enum PlaylistItem {
    Table,
    Id,
    PlaylistId,
    ClipUuid,
    Position,
    CreatedAt,
}
