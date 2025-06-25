use sea_orm::DatabaseConnection;
use sea_orm_migration::MigratorTrait;
use tracing::{error, info};

use crate::migrations::Migrator;
use crate::utils::AppError;

/// 数据库迁移管理器
pub struct MigrationManager;

impl MigrationManager {
    /// 运行所有待执行的迁移
    pub async fn migrate_up(connection: &DatabaseConnection) -> Result<(), AppError> {
        info!("开始执行数据库迁移");

        Migrator::up(connection, None).await.map_err(|e| {
            error!("数据库迁移失败: {}", e);
            AppError::Internal(format!("数据库迁移失败: {}", e))
        })?;

        info!("数据库迁移完成");
        Ok(())
    }
}
