use sea_orm_migration::prelude::*;

mod m20240101_000001_create_images_table;
mod m20241201_000001_create_cache_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240101_000001_create_images_table::Migration),
            Box::new(m20241201_000001_create_cache_table::Migration),
        ]
    }
} 