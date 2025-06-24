use std::sync::Arc;
use sea_orm::{DatabaseConnection, TransactionTrait, DbErr};
use async_trait::async_trait;

use crate::utils::AppError;

/// 基础仓储 trait，定义通用的数据访问接口
#[async_trait]
pub trait Repository: Send + Sync {
    /// 获取数据库连接
    fn get_connection(&self) -> Arc<DatabaseConnection>;
    
    /// 执行事务
    async fn transaction<F, R>(&self, func: F) -> Result<R, AppError>
    where
        F: for<'c> FnOnce(&'c sea_orm::DatabaseTransaction) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<R, DbErr>> + Send + 'c>> + Send,
        R: Send;
}

/// 基础仓储实现
pub struct BaseRepository {
    connection: Arc<DatabaseConnection>,
}

impl BaseRepository {
    /// 创建新的基础仓储实例
    pub fn new(connection: Arc<DatabaseConnection>) -> Self {
        Self { connection }
    }
}

#[async_trait]
impl Repository for BaseRepository {
    fn get_connection(&self) -> Arc<DatabaseConnection> {
        self.connection.clone()
    }

    async fn transaction<F, R>(&self, func: F) -> Result<R, AppError>
    where
        F: for<'c> FnOnce(&'c sea_orm::DatabaseTransaction) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<R, DbErr>> + Send + 'c>> + Send,
        R: Send,
    {
        self.connection
            .transaction(func)
            .await
            .map_err(|e| AppError::Internal(format!("事务执行失败: {}", e)))
    }
}

/// 分页查询结果
#[derive(Debug, Clone)]
pub struct PageResult<T> {
    /// 数据项
    pub items: Vec<T>,
    /// 总数量
    pub total: u64,
} 