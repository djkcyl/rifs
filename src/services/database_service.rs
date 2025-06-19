use std::sync::Arc;
use sea_orm::{
    Database, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
    ColumnTrait, ActiveModelTrait, PaginatorTrait, Condition,
    ConnectionTrait, DbBackend, Statement
};
use sea_orm_migration::MigratorTrait;
use chrono::Utc;
use sha2::{Sha256, Digest};

use crate::entities::{image, Image};
use crate::models::{ImageInfo, ImageQuery, ImageStats, TypeStat, TimeStat};
use crate::utils::AppError;
use crate::config::AppConfig;
use crate::migrations::Migrator;

/// 数据库服务结构体
pub struct DatabaseService {
    connection: Arc<DatabaseConnection>,
    db_backend: DbBackend,
}

impl DatabaseService {
    /// 初始化数据库连接
    pub async fn new() -> Result<Self, AppError> {
        let config = AppConfig::get();
        
        // 根据配置创建数据库连接
        let (connection, db_backend) = match config.database.database_type.as_str() {
            "sqlite" => {
                // 确保数据目录存在
                let db_path = config.database.connection_string.strip_prefix("sqlite:")
                    .unwrap_or(&config.database.connection_string);
                
                if let Some(parent) = std::path::Path::new(db_path).parent() {
                    tokio::fs::create_dir_all(parent).await
                        .map_err(|e| AppError::Internal(format!("创建数据目录失败: {}", e)))?;
                }

                // 如果数据库文件不存在，先创建一个空文件
                if !std::path::Path::new(db_path).exists() {
                    tokio::fs::File::create(db_path).await
                        .map_err(|e| AppError::Internal(format!("创建数据库文件失败: {}", e)))?;
                }

                let conn = Database::connect(&config.database.connection_string)
                    .await
                    .map_err(|e| AppError::Internal(format!("SQLite数据库连接失败: {}", e)))?;
                (conn, DbBackend::Sqlite)
            },
            "postgres" | "postgresql" => {
                let conn = Database::connect(&config.database.connection_string)
                    .await
                    .map_err(|e| AppError::Internal(format!("PostgreSQL数据库连接失败: {}", e)))?;
                (conn, DbBackend::Postgres)
            },
            "mysql" => {
                let conn = Database::connect(&config.database.connection_string)
                    .await
                    .map_err(|e| AppError::Internal(format!("MySQL数据库连接失败: {}", e)))?;
                (conn, DbBackend::MySql)
            },
            _ => {
                return Err(AppError::Internal(format!("不支持的数据库类型: {}", config.database.database_type)));
            }
        };

        // 运行数据库迁移
        Migrator::up(&connection, None)
            .await
            .map_err(|e| AppError::Internal(format!("数据库迁移失败: {}", e)))?;

        Ok(Self {
            connection: Arc::new(connection),
            db_backend,
        })
    }

    /// 插入新的图片记录
    pub async fn insert_image(&self, image_info: &ImageInfo) -> Result<(), AppError> {
        let active_model = image::ActiveModel::from(image_info);
        
        active_model.insert(&*self.connection)
            .await
            .map_err(|e| AppError::Internal(format!("插入图片记录失败: {}", e)))?;

        Ok(())
    }

    /// 根据hash获取图片信息
    pub async fn get_image(&self, identifier: &str) -> Result<Option<ImageInfo>, AppError> {
        let result = Image::find()
            .filter(image::Column::Hash.eq(identifier))
            .one(&*self.connection)
            .await
            .map_err(|e| AppError::Internal(format!("查询图片失败: {}", e)))?;

        Ok(result.map(|model| model.into()))
    }

    /// 查询图片列表
    pub async fn query_images(&self, query: &ImageQuery) -> Result<(Vec<ImageInfo>, u64), AppError> {
        let mut select = Image::find();
        let mut condition = Condition::all();

        // 构建查询条件
        if let Some(mime_type) = &query.mime_type {
            condition = condition.add(image::Column::MimeType.eq(mime_type));
        }

        if let Some(min_size) = query.min_size {
            condition = condition.add(image::Column::Size.gte(min_size as i64));
        }

        if let Some(max_size) = query.max_size {
            condition = condition.add(image::Column::Size.lte(max_size as i64));
        }

        if let Some(start_time) = query.start_time {
            condition = condition.add(image::Column::CreatedAt.gte(start_time));
        }

        if let Some(end_time) = query.end_time {
            condition = condition.add(image::Column::CreatedAt.lte(end_time));
        }

        if let Some(search) = &query.search {
            condition = condition.add(image::Column::Hash.contains(search));
        }

        select = select.filter(condition.clone());

        // 排序
        let order_by = query.order_by.as_deref().unwrap_or("created_at");
        let order_dir = query.order_dir.as_deref().unwrap_or("DESC");
        
        select = match order_by {
            "hash" => {
                if order_dir.to_uppercase() == "ASC" {
                    select.order_by_asc(image::Column::Hash)
                } else {
                    select.order_by_desc(image::Column::Hash)
                }
            },

            "size" => {
                if order_dir.to_uppercase() == "ASC" {
                    select.order_by_asc(image::Column::Size)
                } else {
                    select.order_by_desc(image::Column::Size)
                }
            },
            "created_at" | "upload_time" => { // upload_time 作为 created_at 的别名
                if order_dir.to_uppercase() == "ASC" {
                    select.order_by_asc(image::Column::CreatedAt)
                } else {
                    select.order_by_desc(image::Column::CreatedAt)
                }
            },
            "access_count" => {
                if order_dir.to_uppercase() == "ASC" {
                    select.order_by_asc(image::Column::AccessCount)
                } else {
                    select.order_by_desc(image::Column::AccessCount)
                }
            },
            _ => { // 默认按创建时间排序
                if order_dir.to_uppercase() == "ASC" {
                    select.order_by_asc(image::Column::CreatedAt)
                } else {
                    select.order_by_desc(image::Column::CreatedAt)
                }
            }
        };

        // 分页
        let limit = query.limit.unwrap_or(20);
        let offset = query.offset.unwrap_or(0);

        let paginator = select.paginate(&*self.connection, limit);
        let total = paginator.num_items()
            .await
            .map_err(|e| AppError::Internal(format!("查询图片总数失败: {}", e)))?;

        let models = paginator.fetch_page(offset / limit)
            .await
            .map_err(|e| AppError::Internal(format!("查询图片列表失败: {}", e)))?;

        let images: Vec<ImageInfo> = models.into_iter().map(|model| model.into()).collect();

        Ok((images, total))
    }

    /// 更新图片访问信息
    pub async fn update_access(&self, identifier: &str) -> Result<(), AppError> {
        let image_model = Image::find()
            .filter(image::Column::Hash.eq(identifier))
            .one(&*self.connection)
            .await
            .map_err(|e| AppError::Internal(format!("查询图片失败: {}", e)))?;

        if let Some(model) = image_model {
            let mut active_model: image::ActiveModel = model.into();
            active_model.access_count = sea_orm::Set(active_model.access_count.unwrap() + 1);
            active_model.last_accessed = sea_orm::Set(Some(Utc::now()));
            
            active_model.update(&*self.connection)
                .await
                .map_err(|e| AppError::Internal(format!("更新访问信息失败: {}", e)))?;
        }

        Ok(())
    }

    /// 删除图片记录
    pub async fn delete_image(&self, identifier: &str) -> Result<bool, AppError> {
        let result = Image::delete_many()
            .filter(image::Column::Hash.eq(identifier))
            .exec(&*self.connection)
            .await
            .map_err(|e| AppError::Internal(format!("删除图片记录失败: {}", e)))?;

        Ok(result.rows_affected > 0)
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> Result<ImageStats, AppError> {
        // 获取基本统计信息
        let basic_stats = self.connection
            .query_one(Statement::from_string(
                self.db_backend,
                "SELECT COUNT(*) as total_count, COALESCE(SUM(size), 0) as total_size, COALESCE(AVG(size), 0) as average_size FROM images".to_string()
            ))
            .await
            .map_err(|e| AppError::Internal(format!("查询基本统计失败: {}", e)))?;

        let (total_count, total_size, average_size) = if let Some(row) = basic_stats {
            (
                row.try_get("", "total_count").unwrap_or(0i64),
                row.try_get("", "total_size").unwrap_or(0i64),
                row.try_get("", "average_size").unwrap_or(0.0f64)
            )
        } else {
            (0i64, 0i64, 0.0f64)
        };

        // 按类型统计
        let type_stats = self.connection
            .query_all(Statement::from_string(
                self.db_backend,
                "SELECT mime_type, COUNT(*) as count, COALESCE(SUM(size), 0) as total_size FROM images GROUP BY mime_type".to_string()
            ))
            .await
            .map_err(|e| AppError::Internal(format!("查询类型统计失败: {}", e)))?;

        let mut by_type = Vec::new();
        for row in type_stats {
            by_type.push(TypeStat {
                mime_type: row.try_get("", "mime_type").unwrap_or_default(),
                count: row.try_get("", "count").unwrap_or(0),
                total_size: row.try_get("", "total_size").unwrap_or(0),
            });
        }

        // 按时间统计（按天）
        let time_stats = self.connection
            .query_all(Statement::from_string(
                self.db_backend,
                "SELECT DATE(created_at) as date, COUNT(*) as count, COALESCE(SUM(size), 0) as total_size FROM images GROUP BY DATE(created_at) ORDER BY date DESC LIMIT 30".to_string()
            ))
            .await
            .map_err(|e| AppError::Internal(format!("查询时间统计失败: {}", e)))?;

        let mut by_time = Vec::new();
        for row in time_stats {
            by_time.push(TimeStat {
                date: row.try_get("", "date").unwrap_or_default(),
                count: row.try_get("", "count").unwrap_or(0),
                total_size: row.try_get("", "total_size").unwrap_or(0),
            });
        }

        Ok(ImageStats {
            total_count,
            total_size,
            average_size,
            by_type,
            by_time,
        })
    }

    /// 根据hash查找图片
    pub async fn find_by_hash(&self, file_hash: &str) -> Result<Option<ImageInfo>, AppError> {
        let result = Image::find()
            .filter(image::Column::Hash.eq(file_hash))
            .one(&*self.connection)
            .await
            .map_err(|e| AppError::Internal(format!("根据hash查询图片失败: {}", e)))?;

        Ok(result.map(|model| model.into()))
    }

    /// 计算文件哈希值
    pub fn calculate_file_hash(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }
} 