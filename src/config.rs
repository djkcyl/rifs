use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::OnceLock;

use crate::utils::{AppError, ByteSize, Duration};

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
    /// 请求处理超时时间
    pub request_timeout: Duration,
}

/// 存储配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StorageConfig {
    /// 上传目录
    pub upload_dir: String,
    /// 最大文件大小
    pub max_file_size: ByteSize,
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
    /// 连接超时时间
    pub connect_timeout: Duration,
    /// 空闲连接超时时间
    pub idle_timeout: Duration,
    /// 连接最大生存时间
    pub max_lifetime: Duration,
}

/// 日志配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggingConfig {
    /// 日志级别
    pub level: String,
    /// 是否启用颜色
    pub enable_color: bool,
    /// 日志目录路径
    pub log_dir: String,
    /// 日志文件大小限制
    pub max_log_size: ByteSize,
}

/// 缓存配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CacheConfig {
    /// 静态文件缓存时间
    pub max_age: Duration,
    /// 转换结果缓存目录
    pub cache_dir: String,
    /// 启用转换结果缓存
    pub enable_transform_cache: bool,
    /// 缓存最大项目数量
    pub max_cache_entries: u64,
    /// 缓存最大总大小
    pub max_cache_size: ByteSize,
    /// 缓存项最大生存时间
    pub max_cache_age: Duration,
    /// 自动清理间隔
    pub auto_cleanup_interval: Duration,
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
                request_timeout: Duration::seconds(60),
            },
            storage: StorageConfig {
                upload_dir: "uploads".to_string(),
                max_file_size: ByteSize::mb(10), // 10MB
            },
            database: DatabaseConfig {
                database_type: "sqlite".to_string(),
                connection_string: "sqlite:./data/images.db".to_string(),
                max_connections: 20,
                min_connections: 4,
                connect_timeout: Duration::seconds(30),
                idle_timeout: Duration::minutes(5),
                max_lifetime: Duration::minutes(30),
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                enable_color: true,
                log_dir: "".to_string(),        // 默认只输出到控制台
                max_log_size: ByteSize::mb(10), // 10MB
            },
            cache: CacheConfig {
                max_age: Duration::days(365), // 1年
                cache_dir: "cache".to_string(),
                enable_transform_cache: true,
                max_cache_entries: 1000,
                max_cache_size: ByteSize::mb(100),         // 100MB
                max_cache_age: Duration::days(30),         // 30天
                auto_cleanup_interval: Duration::hours(1), // 1小时
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

# 配置单位说明:
# 字节大小: 支持 B, KB, MB, GB, TB (如: "10MB", "1GB")
# 时间长度: 支持 s, m, h, d, w, y (如: "30s", "1h", "7d")
# 大小写不敏感，支持小数，可省略单位

# ========================================
# 服务器配置
# ========================================

[server]
# 服务器监听地址
host = "0.0.0.0"
# 服务器监听端口
port = 3000
# 是否启用CORS跨域
enable_cors = true
# 请求处理超时时间
request_timeout = "1m"

# ========================================
# 存储配置
# ========================================

[storage]
# 上传文件存储目录
upload_dir = "uploads"
# 最大文件大小
max_file_size = "10MB"

# ========================================
# 日志配置  
# ========================================

[logging]
# 日志级别: trace, debug, info, warn, error
level = "info"
# 是否启用日志颜色
enable_color = true
# 日志目录路径（留空表示只输出到控制台）
log_dir = ""
# 日志文件大小限制（0表示仅按天轮转）
max_log_size = "10MB"

# ========================================
# 缓存配置
# ========================================

[cache]
# 静态文件缓存时间
max_age = "1y"
# 转换结果缓存目录
cache_dir = "cache"
# 启用转换结果缓存
enable_transform_cache = true
# 缓存最大项目数量
max_cache_entries = 10000
# 缓存最大总大小
max_cache_size = "100MB"
# 缓存项最大生存时间
max_cache_age = "30d"
# 自动清理间隔
auto_cleanup_interval = "1h"
# 热度评分衰减因子（0.0-1.0）
heat_decay_factor = 0.98
# 最小热度评分阈值
min_heat_score = 0.1
# 空间使用阈值百分比（0.0-1.0），超过此阈值才触发基于热度的清理
space_threshold_percent = 0.8

# ========================================
# 数据库配置
# ========================================

[database]
# 数据库类型: sqlite, postgres, mysql
database_type = "sqlite"
# 数据库连接字符串
connection_string = "sqlite:./data/images.db"
# 最大连接数
max_connections = 20
# 最小连接数
min_connections = 4
# 连接超时时间
connect_timeout = "30s"
# 空闲连接超时时间
idle_timeout = "5m"
# 连接最大生存时间
max_lifetime = "30m"

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
                let default_config = Self::default();

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
                    // 检查是否在容器环境中（通过检查常见的容器环境变量或/.dockerenv文件）
                    let is_container = std::env::var("RIFS_SERVER_HOST").is_ok()
                        || std::env::var("CONTAINER").is_ok()
                        || std::env::var("KUBERNETES_SERVICE_HOST").is_ok()
                        || std::path::Path::new("/.dockerenv").exists()
                        || std::path::Path::new("/proc/1/cgroup").exists()
                            && std::fs::read_to_string("/proc/1/cgroup")
                                .map_or(false, |content| content.contains("docker"));

                    if is_container {
                        // 容器环境，使用环境变量配置（如果有的话），否则使用默认配置
                        eprintln!("🐳 检测到容器环境，使用环境变量配置启动");
                        Self::load_from_env_only()?
                    } else {
                        // 非容器环境，创建默认配置文件后退出
                        let config_file_path = "config.toml";

                        eprintln!("⚠️  未找到配置文件: {}", config_file_path);
                        eprintln!("📝 正在创建默认配置文件...");

                        Self::create_default_config_file(config_file_path).map_err(|e| {
                            AppError::Internal(format!("创建默认配置文件失败: {}", e))
                        })?;

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
                            "已创建默认配置文件，请修改后重新启动".to_string(),
                        ));
                    }
                }
            }
        };

        CONFIG
            .set(config)
            .map_err(|_| AppError::Internal("配置已被初始化".to_string()))?;

        Ok(())
    }

    /// 获取全局配置
    pub fn get() -> &'static AppConfig {
        CONFIG
            .get()
            .expect("配置未初始化，请先调用 AppConfig::init()")
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
        format!("public, max-age={}", self.cache.max_age.as_seconds())
    }
}
