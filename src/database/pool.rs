use std::sync::Arc;
use std::time::Duration;
use sea_orm::{Database, DatabaseConnection};
use tracing::{info, error, warn};

use crate::config::AppConfig;
use crate::utils::AppError;

/// 数据库连接池配置
#[derive(Debug, Clone)]
pub struct PoolConfig {
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout: Duration,
    pub idle_timeout: Duration,
    pub max_lifetime: Duration,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 32,
            min_connections: 4,
            connect_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(300),
            max_lifetime: Duration::from_secs(1800),
        }
    }
}

/// 数据库连接池管理器
#[derive(Clone)]
pub struct DatabasePool {
    connection: Arc<DatabaseConnection>,
    pool_config: PoolConfig,
}

impl DatabasePool {
    /// 创建新的数据库连接池
    pub async fn new() -> Result<Self, AppError> {
        Self::with_config(PoolConfig::default()).await
    }

    /// 使用自定义配置创建数据库连接池
    pub async fn with_config(pool_config: PoolConfig) -> Result<Self, AppError> {
        let config = AppConfig::get();
        
        info!("初始化数据库连接池，类型: {}", config.database.database_type);
        
        // 构建连接选项
        let mut conn_options = sea_orm::ConnectOptions::new(&config.database.connection_string);
        conn_options
            .max_connections(pool_config.max_connections)
            .min_connections(pool_config.min_connections)
            .connect_timeout(pool_config.connect_timeout)
            .idle_timeout(pool_config.idle_timeout)
            .max_lifetime(pool_config.max_lifetime)
            .sqlx_logging(true)
            .sqlx_logging_level(tracing::log::LevelFilter::Debug);

        // 根据数据库类型进行特殊处理
        match config.database.database_type.as_str() {
            "sqlite" => {
                let db_path = config.database.connection_string
                    .strip_prefix("sqlite:")
                    .unwrap_or(&config.database.connection_string);
                
                // 确保数据目录存在
                if let Some(parent) = std::path::Path::new(db_path).parent() {
                    tokio::fs::create_dir_all(parent).await
                        .map_err(|e| AppError::Internal(format!("创建数据目录失败: {}", e)))?;
                }

                // 如果数据库文件不存在，先创建一个空文件
                if !std::path::Path::new(db_path).exists() {
                    tokio::fs::File::create(db_path).await
                        .map_err(|e| AppError::Internal(format!("创建数据库文件失败: {}", e)))?;
                }
            },
            "postgres" | "postgresql" => {
                // PostgreSQL 特定配置
                conn_options.sqlx_logging(true);
            },
            "mysql" => {
                // MySQL 特定配置
                conn_options.sqlx_logging(true);
            },
            _ => {
                return Err(AppError::Internal(format!("不支持的数据库类型: {}", config.database.database_type)));
            }
        }

        // 创建连接
        let connection = Database::connect(conn_options)
            .await
            .map_err(|e| {
                error!("数据库连接失败: {}", e);
                AppError::Internal(format!("数据库连接失败: {}", e))
            })?;

        info!("数据库连接池初始化成功");

        Ok(Self {
            connection: Arc::new(connection),
            pool_config,
        })
    }

    /// 获取数据库连接
    pub fn get_connection(&self) -> Arc<DatabaseConnection> {
        self.connection.clone()
    }

    /// 检查连接池健康状态
    pub async fn health_check(&self) -> Result<(), AppError> {
        self.connection
            .ping()
            .await
            .map_err(|e| AppError::Internal(format!("数据库连接健康检查失败: {}", e)))?;
        
        Ok(())
    }

    /// 获取连接池统计信息
    pub async fn get_pool_stats(&self) -> PoolStats {
        // 注意：Sea-ORM 目前不直接暴露连接池统计信息
        // 这里返回基本配置信息
        PoolStats {
            max_connections: self.pool_config.max_connections,
            min_connections: self.pool_config.min_connections,
            active_connections: 0, // 无法直接获取
            idle_connections: 0,   // 无法直接获取
        }
    }

    /// 关闭数据库连接池
    pub async fn close(&self) -> Result<(), AppError> {
        info!("关闭数据库连接池...");
        
        // 执行最后的健康检查以确保连接正常
        if let Err(e) = self.health_check().await {
            warn!("关闭前健康检查失败: {}", e);
        }

        // Sea-ORM/SQLx 会在Drop时自动关闭连接
        // 这里我们可以做一些清理工作
        info!("数据库连接池已准备关闭，连接将在Drop时自动清理");
        
        Ok(())
    }
}

/// 连接池统计信息
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub max_connections: u32,
    pub min_connections: u32,
    pub active_connections: u32,
    pub idle_connections: u32,
}

impl PoolStats {
    pub fn utilization_rate(&self) -> f32 {
        if self.max_connections == 0 {
            0.0
        } else {
            self.active_connections as f32 / self.max_connections as f32
        }
    }
} 