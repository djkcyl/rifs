use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 创建缓存表
        manager
            .create_table(
                Table::create()
                    .table(Cache::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Cache::CacheKey)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Cache::OriginalHash).string().not_null())
                    .col(ColumnDef::new(Cache::TransformParams).string().not_null())
                    .col(ColumnDef::new(Cache::FilePath).string().not_null())
                    .col(ColumnDef::new(Cache::FileSize).big_integer().not_null())
                    .col(ColumnDef::new(Cache::MimeType).string().not_null())
                    .col(
                        ColumnDef::new(Cache::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Cache::LastAccessed)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Cache::AccessCount)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Cache::HeatScore)
                            .double()
                            .not_null()
                            .default(0.0),
                    )
                    .to_owned(),
            )
            .await?;

        // 创建索引以提高查询性能
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_cache_original_hash")
                    .table(Cache::Table)
                    .col(Cache::OriginalHash)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_cache_last_accessed")
                    .table(Cache::Table)
                    .col(Cache::LastAccessed)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_cache_heat_score")
                    .table(Cache::Table)
                    .col(Cache::HeatScore)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_cache_access_count")
                    .table(Cache::Table)
                    .col(Cache::AccessCount)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_cache_file_size")
                    .table(Cache::Table)
                    .col(Cache::FileSize)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Cache::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Cache {
    Table,
    CacheKey,
    OriginalHash,
    TransformParams,
    FilePath,
    FileSize,
    MimeType,
    CreatedAt,
    LastAccessed,
    AccessCount,
    HeatScore,
}
