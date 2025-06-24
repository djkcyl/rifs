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
    /// 关闭超时时间（秒）
    pub shutdown_timeout: u64,
    /// 请求处理超时时间（秒）
    pub request_timeout: u64,
}

/// 存储配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StorageConfig {
    /// 上传目录
    pub upload_dir: String,
    /// 最大文件大小（字节）
    pub max_file_size: u64,
    /// 支持的文件类型
    pub allowed_types: Vec<String>,
    /// 是否保留原始文件名
    pub preserve_filename: bool,
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
    /// 最小连接数
    pub min_connections: u32,
    /// 连接超时时间（秒）
    pub connect_timeout: u64,
    /// 空闲连接超时时间（秒）
    pub idle_timeout: u64,
    /// 连接最大生存时间（秒）
    pub max_lifetime: u64,
}

/// 日志配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggingConfig {
    /// 日志级别
    pub level: String,
    /// 是否启用颜色
    pub enable_color: bool,
    /// 日志文件路径
    pub log_file: String,
    /// 日志文件大小限制（字节）
    pub max_log_size: u64,
}

/// 缓存配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CacheConfig {
    /// 静态文件缓存时间（秒）
    pub max_age: u64,
    /// 转换结果缓存目录
    pub cache_dir: String,
    /// 启用转换结果缓存
    pub enable_transform_cache: bool,
    /// 缓存最大项目数量
    pub max_cache_entries: u64,
    /// 缓存最大总大小（字节）
    pub max_cache_size: u64,
    /// 缓存项最大生存时间（秒）
    pub max_cache_age: u64,
    /// 自动清理间隔（秒）
    pub auto_cleanup_interval: u64,
    /// 热度评分衰减因子
    pub heat_decay_factor: f64,
    /// 最小热度评分阈值
    pub min_heat_score: f64,
    /// 空间使用阈值百分比（0.0-1.0），超过此阈值才触发基于热度的清理
    pub space_threshold_percent: f64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 3000,
                enable_cors: true,
                shutdown_timeout: 30,
                request_timeout: 60,
            },
            storage: StorageConfig {
                upload_dir: "uploads".to_string(),
                max_file_size: 10 * 1024 * 1024, // 10MB
                allowed_types: Vec::new(),
                preserve_filename: false,
            },
            database: DatabaseConfig {
                database_type: "sqlite".to_string(),
                connection_string: "sqlite:./data/images.db".to_string(),
                max_connections: 20,
                min_connections: 4,
                connect_timeout: 30,
                idle_timeout: 300,
                max_lifetime: 1800,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                enable_color: true,
                log_file: "".to_string(), // 默认只输出到控制台
                max_log_size: 0, // 默认无限制
            },
            cache: CacheConfig {
                max_age: 31536000, // 1年
                cache_dir: "cache".to_string(),
                enable_transform_cache: true,
                max_cache_entries: 1000,
                max_cache_size: 100 * 1024 * 1024, // 100MB
                max_cache_age: 3600, // 1 hour
                auto_cleanup_interval: 3600, // 1 hour
                heat_decay_factor: 0.9,
                min_heat_score: 0.1,
                space_threshold_percent: 0.8, // 80%使用率时才触发热度清理
            },
        }
    }
}

impl AppConfig {
    /// 从配置文件加载配置
    pub fn load_from_file(config_path: &str) -> Result<Self, AppError> {
        let settings = config::Config::builder()
            .add_source(config::File::with_name(config_path))
            .add_source(config::Environment::with_prefix("RIFS"))
            .build()
            .map_err(|e| AppError::Internal(format!("配置加载失败: {}", e)))?;

        settings
            .try_deserialize()
            .map_err(|e| AppError::Internal(format!("配置解析失败: {}", e)))
    }

    /// 创建默认配置文件
    pub fn create_default_config_file(config_path: &str) -> Result<(), AppError> {
        // 使用内置的默认配置模板创建配置文件
        let default_config_content = r#"# RIFS 图床服务配置文件
# 这是自动生成的默认配置文件，请根据实际需求修改
# 修改后请重新启动服务使配置生效

[server]
# 服务器监听地址
host = "0.0.0.0"
# 服务器监听端口
port = 3000
# 是否启用CORS跨域
enable_cors = true
# 关闭超时时间（秒）
shutdown_timeout = 30
# 请求处理超时时间（秒）
request_timeout = 60

[storage]
# 上传文件存储目录
upload_dir = "uploads"
# 最大文件大小（字节）10MB
max_file_size = 10485760
# 支持的文件类型（留空表示自动检测）
allowed_types = []
# 是否保留原始文件名（安全起见建议false）
preserve_filename = false

[logging]
# 日志级别: trace, debug, info, warn, error
level = "info"
# 是否启用日志颜色
enable_color = true
# 日志文件路径（留空表示只输出到控制台）
log_file = ""
# 日志文件大小限制（字节，0表示无限制）
max_log_size = 0

[cache]
# 静态文件缓存时间（秒）1年
max_age = 31536000
# 转换结果缓存目录
cache_dir = "cache"
# 启用转换结果缓存
enable_transform_cache = true
# 缓存最大项目数量
max_cache_entries = 10000
# 缓存最大总大小（字节）100MB
max_cache_size = 104857600
# 缓存项最大生存时间（秒）30天
max_cache_age = 2592000
# 自动清理间隔（秒）1小时
auto_cleanup_interval = 3600
# 热度评分衰减因子（0.0-1.0）
heat_decay_factor = 0.98
# 最小热度评分阈值
min_heat_score = 0.1
# 空间使用阈值百分比（0.0-1.0），超过此阈值才触发基于热度的清理
space_threshold_percent = 0.8

[database]
# 数据库类型: sqlite, postgres, mysql
database_type = "sqlite"
# 数据库连接字符串
connection_string = "sqlite:./data/images.db"
# 最大连接数
max_connections = 20
# 最小连接数
min_connections = 4
# 连接超时时间（秒）
connect_timeout = 30
# 空闲连接超时时间（秒）
idle_timeout = 300
# 连接最大生存时间（秒）
max_lifetime = 1800

# 数据库连接字符串示例：
# SQLite: "sqlite:./data/images.db"
# PostgreSQL: "postgres://username:password@localhost:5432/images"
# MySQL: "mysql://username:password@localhost:3306/images"
"#;
        
        std::fs::write(config_path, default_config_content)
            .map_err(|e| AppError::Internal(format!("创建默认配置文件失败: {}", e)))?;
        
        Ok(())
    }

    /// 从环境变量加载配置
    pub fn load_from_env_only() -> Result<Self, AppError> {
        let settings = config::Config::builder()
            .add_source(config::Environment::with_prefix("RIFS"))
            .build()
            .map_err(|e| AppError::Internal(format!("环境变量配置加载失败: {}", e)))?;

        // 尝试从环境变量解析，如果失败则使用默认配置
        settings
            .try_deserialize()
            .or_else(|_| {
                // 如果环境变量不完整，使用默认配置并用环境变量覆盖
                let mut default_config = Self::default();
                
                // 手动覆盖环境变量中存在的配置项
                if let Ok(config_with_env) = config::Config::builder()
                    .add_source(config::Config::try_from(&default_config).unwrap())
                    .add_source(config::Environment::with_prefix("RIFS"))
                    .build()
                    .and_then(|c| c.try_deserialize::<Self>())
                {
                    Ok::<Self, config::ConfigError>(config_with_env)
                } else {
                    Ok::<Self, config::ConfigError>(default_config)
                }
            })
            .map_err(|e| AppError::Internal(format!("配置解析失败: {}", e)))
    }

    /// 初始化全局配置
    pub fn init(config_path: Option<&str>) -> Result<(), AppError> {
        let config = if let Some(config_file) = config_path {
            // 明确指定了配置文件路径，尝试加载
            Self::load_from_file(config_file)?
        } else {
            // 未指定配置文件，尝试默认路径，失败则使用环境变量
            match Self::load_from_file("config") {
                Ok(config) => config,
                Err(_) => {
                    // 检查是否在容器环境中（通过检查常见的容器环境变量）
                    let is_container = std::env::var("RIFS_SERVER_HOST").is_ok() 
                        || std::env::var("CONTAINER").is_ok()
                        || std::env::var("KUBERNETES_SERVICE_HOST").is_ok();
                    
                    if is_container {
                        // 容器环境，直接使用环境变量配置
                        eprintln!("🐳 检测到容器环境，使用环境变量配置");
                        Self::load_from_env_only()?
                    } else {
                        // 非容器环境，创建默认配置文件
                        let config_file_path = "config.toml";
                        
                        eprintln!("⚠️  未找到配置文件: {}", config_file_path);
                        eprintln!("📝 正在创建默认配置文件...");
                        
                        Self::create_default_config_file(&config_file_path)
                            .map_err(|e| AppError::Internal(format!("创建默认配置文件失败: {}", e)))?;
                        
                        eprintln!("✅ 已创建默认配置文件: {}", config_file_path);
                        eprintln!("📋 配置文件包含所有必需设置和详细说明");
                        eprintln!("🔧 请根据实际需求修改配置文件，然后重新启动服务");
                        eprintln!("💡 主要配置项:");
                        eprintln!("   - [server] 服务器端口和地址设置");
                        eprintln!("   - [database] 数据库类型和连接配置");
                        eprintln!("   - [storage] 文件存储目录和大小限制");
                        eprintln!("   - [cache] 缓存策略和清理设置");
                        eprintln!("   - [logging] 日志级别和输出设置");
                        
                        return Err(AppError::Internal(
                            "已创建默认配置文件，请修改后重新启动".to_string()
                        ));
                    }
                }
            }
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