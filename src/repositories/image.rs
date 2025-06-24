use std::sync::Arc;
use sea_orm::{
    DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
    ColumnTrait, ActiveModelTrait, PaginatorTrait, Condition,
    ConnectionTrait, Statement
};
use chrono::Utc;
use async_trait::async_trait;
use tracing::{info, debug};

use crate::entities::{image, Image};
use crate::models::{ImageInfo, ImageQuery, ImageStats, TypeStat, TimeStat};
use crate::repositories::{Repository, BaseRepository, PageResult};
use crate::utils::AppError;

/// 图片仓储接口
#[async_trait]
pub trait ImageRepositoryTrait: Repository {
    /// 插入新的图片记录
    async fn insert(&self, image_info: &ImageInfo) -> Result<(), AppError>;
    
    /// 根据hash获取图片信息
    async fn find_by_hash(&self, hash: &str) -> Result<Option<ImageInfo>, AppError>;
    
    /// 分页查询图片列表
    async fn find_by_query(&self, query: &ImageQuery) -> Result<PageResult<ImageInfo>, AppError>;
    
    /// 更新图片访问信息
    async fn update_access(&self, hash: &str) -> Result<bool, AppError>;
    
    /// 删除图片记录
    async fn delete_by_hash(&self, hash: &str) -> Result<bool, AppError>;
    
    /// 获取统计信息
    async fn get_stats(&self) -> Result<ImageStats, AppError>;
}

/// 图片仓储实现
pub struct ImageRepository {
    base: BaseRepository,
}

impl ImageRepository {
    /// 创建新的图片仓储实例
    pub fn new(connection: Arc<DatabaseConnection>) -> Self {
        Self {
            base: BaseRepository::new(connection),
        }
    }

    /// 构建查询条件
    fn build_query_condition(&self, query: &ImageQuery) -> Condition {
        let mut condition = Condition::all();

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

        condition
    }

    /// 应用排序
    fn apply_ordering(&self, select: sea_orm::Select<Image>, query: &ImageQuery) -> sea_orm::Select<Image> {
        let order_by = query.order_by.as_deref().unwrap_or("created_at");
        let order_dir = query.order_dir.as_deref().unwrap_or("DESC");
        
        match order_by {
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
            "created_at" | "upload_time" => {
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
            _ => {
                if order_dir.to_uppercase() == "ASC" {
                    select.order_by_asc(image::Column::CreatedAt)
                } else {
                    select.order_by_desc(image::Column::CreatedAt)
                }
            }
        }
    }
}

#[async_trait]
impl Repository for ImageRepository {
    fn get_connection(&self) -> Arc<DatabaseConnection> {
        self.base.get_connection()
    }

    async fn transaction<F, R>(&self, func: F) -> Result<R, AppError>
    where
        F: for<'c> FnOnce(&'c sea_orm::DatabaseTransaction) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<R, sea_orm::DbErr>> + Send + 'c>> + Send,
        R: Send,
    {
        self.base.transaction(func).await
    }
}

#[async_trait]
impl ImageRepositoryTrait for ImageRepository {
    async fn insert(&self, image_info: &ImageInfo) -> Result<(), AppError> {
        debug!("插入图片记录: {}", image_info.hash);
        
        let active_model = image::ActiveModel::from(image_info);
        let connection = self.get_connection();
        
        active_model.insert(&*connection)
            .await
            .map_err(|e| AppError::Internal(format!("插入图片记录失败: {}", e)))?;

        info!("图片记录插入成功: {}", image_info.hash);
        Ok(())
    }

    async fn find_by_hash(&self, hash: &str) -> Result<Option<ImageInfo>, AppError> {
        debug!("根据hash查询图片: {}", hash);
        
        let connection = self.get_connection();
        let result = Image::find()
            .filter(image::Column::Hash.eq(hash))
            .one(&*connection)
            .await
            .map_err(|e| AppError::Internal(format!("查询图片失败: {}", e)))?;

        Ok(result.map(|model| model.into()))
    }

    async fn find_by_query(&self, query: &ImageQuery) -> Result<PageResult<ImageInfo>, AppError> {
        debug!("分页查询图片列表: {:?}", query);
        
        let connection = self.get_connection();
        let mut select = Image::find();
        let condition = self.build_query_condition(query);
        
        select = select.filter(condition.clone());
        select = self.apply_ordering(select, query);

        // 分页
        let limit = query.limit.unwrap_or(20);
        let offset = query.offset.unwrap_or(0);

        let paginator = select.paginate(&*connection, limit);
        let total = paginator.num_items()
            .await
            .map_err(|e| AppError::Internal(format!("查询图片总数失败: {}", e)))?;

        let models = paginator.fetch_page(offset / limit)
            .await
            .map_err(|e| AppError::Internal(format!("查询图片列表失败: {}", e)))?;

        let images: Vec<ImageInfo> = models.into_iter().map(|model| model.into()).collect();
        let _page = offset / limit;

        Ok(PageResult {
            items: images,
            total,
        })
    }

    async fn update_access(&self, hash: &str) -> Result<bool, AppError> {
        debug!("更新图片访问信息: {}", hash);
        
        let connection = self.get_connection();
        let image_model = Image::find()
            .filter(image::Column::Hash.eq(hash))
            .one(&*connection)
            .await
            .map_err(|e| AppError::Internal(format!("查询图片失败: {}", e)))?;

        if let Some(model) = image_model {
            let mut active_model: image::ActiveModel = model.into();
            active_model.access_count = sea_orm::Set(active_model.access_count.unwrap() + 1);
            active_model.last_accessed = sea_orm::Set(Some(Utc::now()));
            
            active_model.update(&*connection)
                .await
                .map_err(|e| AppError::Internal(format!("更新访问信息失败: {}", e)))?;
            
            debug!("图片访问信息更新成功: {}", hash);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn delete_by_hash(&self, hash: &str) -> Result<bool, AppError> {
        debug!("删除图片记录: {}", hash);
        
        let connection = self.get_connection();
        let result = Image::delete_many()
            .filter(image::Column::Hash.eq(hash))
            .exec(&*connection)
            .await
            .map_err(|e| AppError::Internal(format!("删除图片记录失败: {}", e)))?;

        let deleted = result.rows_affected > 0;
        if deleted {
            info!("图片记录删除成功: {}", hash);
        }
        
        Ok(deleted)
    }

    async fn get_stats(&self) -> Result<ImageStats, AppError> {
        debug!("获取图片统计信息");
        
        let connection = self.get_connection();
        let db_backend = connection.get_database_backend();

        // 获取基本统计信息
        let basic_stats = connection
            .query_one(Statement::from_string(
                db_backend,
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
        let type_stats = connection
            .query_all(Statement::from_string(
                db_backend,
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
        let time_stats = connection
            .query_all(Statement::from_string(
                db_backend,
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
} 