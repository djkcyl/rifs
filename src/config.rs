use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::OnceLock;

use crate::utils::AppError;

/// 全局配置实例
static CONFIG: OnceLock<AppConfig> = OnceLock::new();

/// 应用配置结构体
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub storage: StorageConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
    pub cache: CacheConfig,
}

/// 服务器配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    /// 监听地址
    pub host: String,
    /// 监听端口
    pub port: u16,
    /// 是否启用CORS
    pub enable_cors: bool,
}

/// 存储配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StorageConfig {
    /// 上传目录
    pub upload_dir: String,
    /// 最大文件大小（字节）
    pub max_file_size: u64,
}

/// 数据库配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    /// 数据库类型 (sqlite, postgres, mysql)
    pub database_type: String,
    /// 数据库连接字符串
    pub connection_string: String,
    /// 最大连接数
    pub max_connections: u32,
}

/// 日志配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggingConfig {
    /// 日志级别
    pub level: String,
    /// 是否启用颜色
    pub enable_color: bool,
}

/// 缓存配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CacheConfig {
    /// 最大缓存时间（秒）
    pub max_age: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 3000,
                enable_cors: true,
            },
            storage: StorageConfig {
                upload_dir: "uploads".to_string(),
                max_file_size: 10 * 1024 * 1024, // 10MB
            },
            database: DatabaseConfig {
                database_type: "sqlite".to_string(),
                connection_string: "sqlite:./data/images.db".to_string(),
                max_connections: 20,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                enable_color: true,
            },
            cache: CacheConfig {
                max_age: 31536000, // 1年
            },
        }
    }
}

impl AppConfig {
    /// 从配置文件加载配置
    pub fn load_from_file(config_path: &str) -> Result<Self, AppError> {
        let settings = config::Config::builder()
            .add_source(config::File::with_name(config_path))
            .add_source(config::Environment::with_prefix("IMAGE_BED"))
            .build()
            .map_err(|e| AppError::Internal(format!("配置加载失败: {}", e)))?;

        settings
            .try_deserialize()
            .map_err(|e| AppError::Internal(format!("配置解析失败: {}", e)))
    }

    /// 初始化全局配置
    pub fn init(config_path: Option<&str>) -> Result<(), AppError> {
        let config = if let Some(path) = config_path {
            Self::load_from_file(path)?
        } else {
            // 尝试加载默认配置文件，失败则使用默认配置
            Self::load_from_file("config").unwrap_or_else(|_| {
                tracing::warn!("未找到配置文件，使用默认配置");
                Self::default()
            })
        };

        CONFIG.set(config).map_err(|_| {
            AppError::Internal("配置已被初始化".to_string())
        })?;

        Ok(())
    }

    /// 获取全局配置
    pub fn get() -> &'static AppConfig {
        CONFIG.get().expect("配置未初始化，请先调用 AppConfig::init()")
    }

    /// 获取服务器监听地址
    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }

    /// 获取上传目录路径
    pub fn upload_dir_path(&self) -> PathBuf {
        PathBuf::from(&self.storage.upload_dir)
    }

    /// 获取缓存控制头
    pub fn cache_control_header(&self) -> String {
        format!("public, max-age={}", self.cache.max_age)
    }
} 