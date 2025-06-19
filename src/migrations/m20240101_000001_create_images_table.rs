use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Images::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Images::Hash)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Images::Size).big_integer().not_null())
                    .col(ColumnDef::new(Images::MimeType).string().not_null())
                    .col(
                        ColumnDef::new(Images::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(Images::LastAccessed).timestamp_with_time_zone())
                    .col(ColumnDef::new(Images::Extension).string().not_null())
                    .col(
                        ColumnDef::new(Images::AccessCount)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .to_owned(),
            )
            .await?;

        // 创建索引以提高查询性能

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_mime_type")
                    .table(Images::Table)
                    .col(Images::MimeType)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_created_at")
                    .table(Images::Table)
                    .col(Images::CreatedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Images::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Images {
    Table,
    Hash,
    Size,
    MimeType,
    CreatedAt,
    LastAccessed,
    Extension,
    AccessCount,
} 