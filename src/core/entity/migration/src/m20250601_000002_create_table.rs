use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Clip::Table)
                    .if_not_exists()
                    .col(pk_auto(Clip::Id))
                    .col(uuid(Clip::Uuid).not_null())
                    .col(string(Clip::Title).not_null())
                    .col(string(Clip::Vup))
                    .col(string(Clip::Song))
                    .col(
                        timestamp(Clip::UploadTime)
                            .not_null()
                            .default(Expr::cust("CURRENT_TIMESTAMP")),
                    )
                    .col(
                        string(Clip::Status)
                            .not_null()
                            .default("pending".to_owned()),
                    )
                    .col(big_integer(Clip::UserId).not_null().default(0))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Clip::Table)
                    .name("idx_uuid")
                    .unique()
                    .col(Clip::Uuid)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .table(Clip::Table)
                    .name("idx_upload_time")
                    .col(Clip::UploadTime)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .table(Clip::Table)
                    .name("idx_uuid_status")
                    .col(Clip::Uuid)
                    .col(Clip::Status)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .table(Clip::Table)
                    .name("idx_user_id")
                    .col(Clip::UserId)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Clip::Table).if_exists().to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Clip {
    Table,
    Id,
    Uuid,
    Title,
    Vup,
    Song,
    UploadTime,
    Status,
    UserId,
}
