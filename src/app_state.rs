use std::sync::Arc;
use tracing::{info, error};

use crate::database::{DatabasePool, MigrationManager};
use crate::config::AppConfig;
use crate::utils::AppError;

/// 应用程序全局状态
/// 
/// 这个结构体包含了应用程序运行时需要的所有共享资源，
/// 包括数据库连接池、配置信息等。通过 Clone trait 可以
/// 在不同的处理器之间共享这些资源。
#[derive(Clone)]
pub struct AppState {
    /// 数据库连接池
    db_pool: Arc<DatabasePool>,
    /// 应用配置
    config: Arc<AppConfig>,
}

impl AppState {
    /// 创建新的应用状态
    /// 
    /// 这个方法会初始化所有必要的应用资源，包括：
    /// - 加载应用配置
    /// - 初始化数据库连接池
    /// - 运行数据库迁移
    pub async fn new() -> Result<Self, AppError> {
        info!("初始化应用状态");

        // 获取应用配置
        let config = Arc::new(AppConfig::get().clone());
        
        // 初始化数据库连接池
        let db_pool = Arc::new(DatabasePool::new().await?);
        
        // 运行数据库迁移
        let connection = db_pool.get_connection();
        MigrationManager::migrate_up(&connection).await?;
        
        // 执行数据库健康检查
        db_pool.health_check().await?;
        
        info!("应用状态初始化完成");

        Ok(Self {
            db_pool,
            config,
        })
    }

    /// 获取数据库连接池
    pub fn db_pool(&self) -> &DatabasePool {
        &self.db_pool
    }

    /// 获取应用配置
    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    /// 执行健康检查
    /// 
    /// 检查所有关键组件的健康状态
    pub async fn health_check(&self) -> Result<HealthStatus, AppError> {
        let mut status = HealthStatus::new();

        // 检查数据库连接池
        match self.db_pool.health_check().await {
            Ok(_) => {
                status.database = ComponentStatus::Healthy;
            }
            Err(e) => {
                error!("数据库健康检查失败: {}", e);
                status.database = ComponentStatus::Unhealthy(e.to_string());
                status.overall = OverallStatus::Unhealthy;
            }
        }

        // 可以添加更多组件的健康检查
        // 例如：文件系统、外部服务等

        Ok(status)
    }

    /// 获取应用统计信息
    pub async fn get_app_stats(&self) -> Result<AppStats, AppError> {
        let pool_stats = self.db_pool.get_pool_stats().await;
        
        Ok(AppStats {
            database_pool: pool_stats,
            // 可以添加更多统计信息
        })
    }

    /// 关闭应用程序
    pub async fn shutdown(&self) -> Result<(), AppError> {
        info!("开始关闭应用程序...");
        
        // 1. 停止接受新请求（由服务器层面处理）
        info!("停止接受新请求");
        
        // 2. 等待现有请求完成（给一些时间让正在处理的请求完成）
        info!("等待现有请求完成...");
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        
        // 3. 执行数据库连接池关闭
        info!("关闭数据库连接池...");
        if let Err(e) = self.db_pool.close().await {
            error!("数据库连接池关闭失败: {}", e);
            return Err(e);
        }
        
        // 4. 保存重要状态（如果需要的话）
        info!("保存应用状态...");
        // 这里可以添加需要持久化的状态保存逻辑
        
        info!("应用程序已关闭");
        Ok(())
    }
}

/// 健康状态信息
#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub overall: OverallStatus,
    pub database: ComponentStatus,
    // 可以添加更多组件状态
}

impl HealthStatus {
    pub fn new() -> Self {
        Self {
            overall: OverallStatus::Healthy,
            database: ComponentStatus::Unknown,
        }
    }

    pub fn is_healthy(&self) -> bool {
        matches!(self.overall, OverallStatus::Healthy)
    }
}

/// 整体状态
#[derive(Debug, Clone)]
pub enum OverallStatus {
    Healthy,
    Unhealthy,
    Degraded,
}

/// 组件状态
#[derive(Debug, Clone)]
pub enum ComponentStatus {
    Healthy,
    Unhealthy(String),
    Unknown,
}

/// 应用统计信息
#[derive(Debug, Clone)]
pub struct AppStats {
    pub database_pool: crate::database::PoolStats,
    // 可以添加更多统计信息字段
}

impl AppStats {
    pub fn database_utilization(&self) -> f32 {
        self.database_pool.utilization_rate()
    }
} 