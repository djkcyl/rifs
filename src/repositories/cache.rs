use std::sync::Arc;
use sea_orm::{
    DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect,
    ColumnTrait, ActiveModelTrait, Condition
};
use chrono::{Utc, DateTime};
use async_trait::async_trait;
use tracing::{info, debug, error};

use crate::entities::{cache, Cache};
use crate::models::{CacheInfo, CacheStats};
use crate::repositories::{Repository, BaseRepository};
use crate::utils::AppError;

/// 缓存仓储接口
#[async_trait]
pub trait CacheRepositoryTrait: Repository {
    /// 插入新的缓存记录
    async fn insert(&self, cache_info: &CacheInfo) -> Result<(), AppError>;
    
    /// 根据缓存键获取缓存信息
    async fn find_by_key(&self, cache_key: &str) -> Result<Option<CacheInfo>, AppError>;
    
    /// 更新缓存访问信息
    async fn update_access(&self, cache_key: &str) -> Result<bool, AppError>;
    
    /// 删除缓存记录
    async fn delete_by_key(&self, cache_key: &str) -> Result<bool, AppError>;
    
    /// 根据原始hash删除相关缓存
    async fn delete_by_original_hash(&self, original_hash: &str) -> Result<u64, AppError>;
    
    /// 获取缓存统计信息
    async fn get_stats(&self) -> Result<CacheStats, AppError>;
    
    /// 获取待清理的缓存列表
    async fn find_cleanup_candidates(&self, max_age_seconds: Option<i64>, limit: Option<u64>) -> Result<Vec<CacheInfo>, AppError>;
    
    /// 清理所有缓存
    async fn clear_all(&self) -> Result<u64, AppError>;
    
    /// 批量衰减所有缓存的热度评分
    async fn decay_all_heat_scores(&self) -> Result<u64, AppError>;
    
    /// 获取低热度评分的缓存列表（用于清理）
    async fn cleanup_low_heat_caches(&self) -> Result<Vec<CacheInfo>, AppError>;
}

/// 缓存仓储实现
pub struct CacheRepository {
    base: BaseRepository,
}

impl CacheRepository {
    /// 创建新的缓存仓储实例
    pub fn new(connection: Arc<DatabaseConnection>) -> Self {
        Self {
            base: BaseRepository::new(connection),
        }
    }

    /// 构建老化缓存的查询条件
    fn build_aged_condition(&self, max_age_seconds: i64) -> Condition {
        let cutoff_time = Utc::now() - chrono::Duration::seconds(max_age_seconds);
        Condition::all().add(cache::Column::CreatedAt.lte(cutoff_time))
    }

    /// 计算衰减后的热度评分
    /// 基于访问频率、时间衰减和配置的衰减因子
    fn calculate_heat_score(
        &self,
        access_count: i64,
        created_at: DateTime<Utc>,
        last_accessed: DateTime<Utc>,
    ) -> f64 {
        let config = crate::config::AppConfig::get();
        let now = Utc::now();
        
        // 计算缓存年龄（小时）
        let age_hours = (now - created_at).num_hours() as f64;
        
        // 计算最后访问时间距现在的小时数
        let hours_since_last_access = (now - last_accessed).num_hours() as f64;
        
        // 基础热度评分：访问次数 / 年龄（小时）
        let base_score = if age_hours > 0.0 {
            access_count as f64 / age_hours.max(1.0)
        } else {
            access_count as f64
        };
        
        // 应用时间衰减因子
        // 每小时应用一次衰减因子
        let decay_factor = config.cache.heat_decay_factor;
        let decayed_score = base_score * decay_factor.powf(hours_since_last_access.max(0.0));
        
        // 确保评分不为负数，并设置最小值
        decayed_score.max(0.0)
    }


}

#[async_trait]
impl Repository for CacheRepository {
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
impl CacheRepositoryTrait for CacheRepository {
    async fn insert(&self, cache_info: &CacheInfo) -> Result<(), AppError> {
        debug!("插入缓存记录: {}", cache_info.cache_key);
        
        let active_model = cache::ActiveModel::from(cache_info);
        let connection = self.get_connection();
        
        active_model.insert(&*connection)
            .await
            .map_err(|e| AppError::Internal(format!("插入缓存记录失败: {}", e)))?;

        info!("缓存记录插入成功: {}", cache_info.cache_key);
        Ok(())
    }

    async fn find_by_key(&self, cache_key: &str) -> Result<Option<CacheInfo>, AppError> {
        debug!("根据缓存键查询缓存: {}", cache_key);
        
        let connection = self.get_connection();
        let result = Cache::find()
            .filter(cache::Column::CacheKey.eq(cache_key))
            .one(&*connection)
            .await
            .map_err(|e| AppError::Internal(format!("查询缓存失败: {}", e)))?;

        Ok(result.map(|model| model.into()))
    }

    async fn update_access(&self, cache_key: &str) -> Result<bool, AppError> {
        debug!("更新缓存访问信息: {}", cache_key);
        
        let connection = self.get_connection();
        let cache_model = Cache::find()
            .filter(cache::Column::CacheKey.eq(cache_key))
            .one(&*connection)
            .await
            .map_err(|e| AppError::Internal(format!("查询缓存失败: {}", e)))?;

        if let Some(model) = cache_model {
            let mut active_model: cache::ActiveModel = model.into();
            let new_access_count = active_model.access_count.unwrap() + 1;
            
            // 更新访问计数和时间
            active_model.access_count = sea_orm::Set(new_access_count);
            active_model.last_accessed = sea_orm::Set(Utc::now());
            
            // 重新计算热度评分
            let created_at = match &active_model.created_at {
                sea_orm::ActiveValue::Set(dt) => *dt,
                sea_orm::ActiveValue::Unchanged(dt) => *dt,
                _ => Utc::now(),
            };
            let last_accessed = Utc::now();
            let heat_score = self.calculate_heat_score(
                new_access_count,
                created_at,
                last_accessed,
            );
            active_model.heat_score = sea_orm::Set(heat_score);
            
            active_model.update(&*connection)
                .await
                .map_err(|e| AppError::Internal(format!("更新缓存访问信息失败: {}", e)))?;
            
            debug!("缓存访问信息更新成功: {}", cache_key);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn delete_by_key(&self, cache_key: &str) -> Result<bool, AppError> {
        debug!("删除缓存记录: {}", cache_key);
        
        let connection = self.get_connection();
        let result = Cache::delete_many()
            .filter(cache::Column::CacheKey.eq(cache_key))
            .exec(&*connection)
            .await
            .map_err(|e| AppError::Internal(format!("删除缓存记录失败: {}", e)))?;

        let deleted = result.rows_affected > 0;
        if deleted {
            info!("缓存记录删除成功: {}", cache_key);
        }
        
        Ok(deleted)
    }

    async fn delete_by_original_hash(&self, original_hash: &str) -> Result<u64, AppError> {
        debug!("根据原始hash删除相关缓存: {}", original_hash);
        
        let connection = self.get_connection();
        let result = Cache::delete_many()
            .filter(cache::Column::OriginalHash.eq(original_hash))
            .exec(&*connection)
            .await
            .map_err(|e| AppError::Internal(format!("删除相关缓存失败: {}", e)))?;

        if result.rows_affected > 0 {
            info!("删除原始hash{}的{}个相关缓存", original_hash, result.rows_affected);
        }
        
        Ok(result.rows_affected)
    }

    async fn get_stats(&self) -> Result<CacheStats, AppError> {
        debug!("获取缓存统计信息");
        
        let connection = self.get_connection();

        // 获取基本统计信息
        let cache_models = Cache::find()
            .all(&*connection)
            .await
            .map_err(|e| AppError::Internal(format!("查询缓存记录失败: {}", e)))?;

        let total_count = cache_models.len() as u64;
        let total_size: u64 = cache_models.iter().map(|c| c.file_size as u64).sum();
        let total_access_count: u64 = cache_models.iter().map(|c| c.access_count as u64).sum();
        
        let average_size = if total_count > 0 {
            total_size as f64 / total_count as f64
        } else {
            0.0
        };

        let hit_rate = if total_access_count > 0 {
            // 这是一个简化的命中率计算，实际应该追踪请求总数
            total_access_count as f64 / (total_access_count as f64 + total_count as f64)
        } else {
            0.0
        };

        Ok(CacheStats {
            total_count: total_count as i64,
            total_size: total_size as i64,
            average_size,
            hit_rate,
            last_cleanup: None,
            top_cached: Vec::new(),
        })
    }

    async fn find_cleanup_candidates(&self, max_age_seconds: Option<i64>, limit: Option<u64>) -> Result<Vec<CacheInfo>, AppError> {
        debug!("查找待清理的缓存候选项，最大年龄: {:?}秒，限制: {:?}", max_age_seconds, limit);
        
        let connection = self.get_connection();
        let mut select = Cache::find();

        // 如果指定了最大年龄，添加年龄条件
        if let Some(max_age) = max_age_seconds {
            let condition = self.build_aged_condition(max_age);
            select = select.filter(condition);
        }

        // 按最后访问时间升序排序（LRU）
        select = select.order_by_asc(cache::Column::LastAccessed);

        // 如果指定了限制，应用限制
        if let Some(limit_val) = limit {
            select = select.limit(limit_val);
        }

        let models = select
            .all(&*connection)
            .await
            .map_err(|e| AppError::Internal(format!("查询待清理缓存失败: {}", e)))?;

        let caches: Vec<CacheInfo> = models.into_iter().map(|model| model.into()).collect();
        debug!("找到{}个待清理的缓存候选项", caches.len());
        
        Ok(caches)
    }

    async fn clear_all(&self) -> Result<u64, AppError> {
        debug!("清理所有缓存");
        
        let connection = self.get_connection();
        let result = Cache::delete_many()
            .exec(&*connection)
            .await
            .map_err(|e| AppError::Internal(format!("清理所有缓存失败: {}", e)))?;

        info!("清理所有缓存完成，删除{}条记录", result.rows_affected);
        Ok(result.rows_affected)
    }

    async fn decay_all_heat_scores(&self) -> Result<u64, AppError> {
        debug!("开始批量衰减所有缓存的热度评分");
        
        let connection = self.get_connection();
        
        // 获取所有缓存记录
        let cache_models = Cache::find()
            .all(&*connection)
            .await
            .map_err(|e| AppError::Internal(format!("查询缓存记录失败: {}", e)))?;

        let mut updated_count = 0;
        
        for model in cache_models {
            // 重新计算热度评分
            let new_heat_score = self.calculate_heat_score(
                model.access_count,
                model.created_at,
                model.last_accessed,
            );
            
            // 只有热度评分发生变化时才更新
            if (new_heat_score - model.heat_score).abs() > 0.001 {
                let mut active_model: cache::ActiveModel = model.into();
                active_model.heat_score = sea_orm::Set(new_heat_score);
                
                let cache_key_ref = active_model.cache_key.as_ref().to_string();
                if let Err(e) = active_model.update(&*connection).await {
                    error!("更新缓存热度评分失败: {} - {}", cache_key_ref, e);
                } else {
                    updated_count += 1;
                }
            }
        }
        
        info!("批量热度衰减完成，更新了 {} 个缓存项", updated_count);
        Ok(updated_count)
    }

    async fn cleanup_low_heat_caches(&self) -> Result<Vec<CacheInfo>, AppError> {
        debug!("查找低热度评分的缓存");
        
        let config = crate::config::AppConfig::get();
        let connection = self.get_connection();
        
        let models = Cache::find()
            .filter(cache::Column::HeatScore.lt(config.cache.min_heat_score))
            .order_by_asc(cache::Column::HeatScore) // 按热度评分升序排序，最冷的在前面
            .all(&*connection)
            .await
            .map_err(|e| AppError::Internal(format!("查询低热度缓存失败: {}", e)))?;

        let caches: Vec<CacheInfo> = models.into_iter().map(|model| model.into()).collect();
        debug!("找到 {} 个低热度缓存项", caches.len());
        
        Ok(caches)
    }
} 