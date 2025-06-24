use std::sync::Arc;
use std::path::Path;
use sea_orm::DatabaseConnection;
use chrono::Utc;
use sha2::{Sha256, Digest};
use tokio::fs;
use tracing::{info, warn, error};

use crate::repositories::{CacheRepository, CacheRepositoryTrait};
use crate::models::{CacheInfo, CacheStats, CacheCleanupPolicy, CacheCleanupResult};
use crate::utils::AppError;
use crate::config::AppConfig;

/// 缓存服务
pub struct CacheService {
    cache_repo: CacheRepository,
    cache_dir: String,
}

impl CacheService {
    /// 创建新的缓存服务实例
    pub fn new(connection: Arc<DatabaseConnection>) -> Result<Self, AppError> {
        let config = AppConfig::get();
        let cache_dir = config.cache.cache_dir.clone();
        
        Ok(Self {
            cache_repo: CacheRepository::new(connection),
            cache_dir,
        })
    }

    /// 异步初始化缓存目录
    pub async fn ensure_cache_dir(&self) -> Result<(), AppError> {
        // 确保缓存目录存在
        tokio::fs::create_dir_all(&self.cache_dir).await
            .map_err(|e| AppError::Internal(format!("创建缓存目录失败: {}", e)))?;
        Ok(())
    }

    /// 生成缓存键
    pub fn generate_cache_key(original_hash: &str, transform_params: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(original_hash.as_bytes());
        hasher.update(b":");
        hasher.update(transform_params.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// 获取缓存文件路径
    fn get_cache_file_path(&self, cache_key: &str, mime_type: &str) -> String {
        // 基于缓存键的前几位创建子目录以避免单个目录文件过多
        let prefix = &cache_key[..2];
        let middle = &cache_key[2..4];
        
        let ext = match mime_type {
            "image/jpeg" => "jpg",
            "image/png" => "png",
            "image/webp" => "webp",
            "image/gif" => "gif",
            "image/avif" => "avif",
            _ => "cache",
        };
        
        format!("{}/{}/{}/{}.{}", self.cache_dir, prefix, middle, cache_key, ext)
    }

    /// 检查缓存是否存在
    pub async fn get_cache(&self, cache_key: &str) -> Result<Option<CacheInfo>, AppError> {
        if let Some(cache_info) = self.cache_repo.find_by_key(cache_key).await? {
            // 检查文件是否实际存在
            if tokio::fs::metadata(&cache_info.file_path).await.is_ok() {
                Ok(Some(cache_info))
            } else {
                // 文件不存在，清理数据库记录
                warn!("缓存文件丢失，清理数据库记录: {}", cache_info.file_path);
                self.cache_repo.delete_by_key(cache_key).await?;
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// 读取缓存文件内容
    pub async fn read_cache(&self, cache_info: &CacheInfo) -> Result<Vec<u8>, AppError> {
        let data = fs::read(&cache_info.file_path).await
            .map_err(|e| AppError::Internal(format!("读取缓存文件失败: {}", e)))?;
        
        // 更新访问信息
        self.cache_repo.update_access(&cache_info.cache_key).await?;
        
        Ok(data)
    }

    /// 保存缓存
    pub async fn save_cache(
        &self,
        original_hash: &str,
        transform_params: &str,
        data: &[u8],
        mime_type: &str,
    ) -> Result<CacheInfo, AppError> {
        let config = AppConfig::get();
        
        // 检查是否启用缓存
        if !config.cache.enable_transform_cache {
            return Err(AppError::Internal("缓存未启用".to_string()));
        }

        let cache_key = Self::generate_cache_key(original_hash, transform_params);
        let file_path = self.get_cache_file_path(&cache_key, mime_type);
        
        // 确保缓存子目录存在
        if let Some(parent) = Path::new(&file_path).parent() {
            fs::create_dir_all(parent).await
                .map_err(|e| AppError::Internal(format!("创建缓存目录失败: {}", e)))?;
        }

        // 写入缓存文件
        fs::write(&file_path, data).await
            .map_err(|e| AppError::Internal(format!("写入缓存文件失败: {}", e)))?;

        // 创建缓存信息
        let now = Utc::now();
        let cache_info = CacheInfo {
            cache_key: cache_key.clone(),
            original_hash: original_hash.to_string(),
            transform_params: transform_params.to_string(),
            file_path: file_path.clone(),
            file_size: data.len() as u64,
            mime_type: mime_type.to_string(),
            created_at: now,
            last_accessed: now,
            access_count: 1,
            heat_score: 1.0, // 新缓存初始热度为1.0
        };

        // 保存到数据库
        self.cache_repo.insert(&cache_info).await?;

        info!("缓存保存成功: {} -> {}", cache_key, file_path);

        // 异步触发清理检查在这里暂时跳过，避免循环依赖

        Ok(cache_info)
    }

    /// 自动清理缓存（根据配置策略）
    pub async fn auto_cleanup(&self) -> Result<CacheCleanupResult, AppError> {
        let config = AppConfig::get();
        
        let policy = CacheCleanupPolicy::from_config(&config);
        self.cleanup_with_policy(&policy).await
    }

    /// 根据策略清理缓存
    pub async fn cleanup_with_policy(&self, policy: &CacheCleanupPolicy) -> Result<CacheCleanupResult, AppError> {
        let mut total_cleaned = 0;
        let mut total_freed = 0;

        // 获取当前缓存统计
        let stats = self.cache_repo.get_stats().await?;

        // 1. 按大小清理（如果超过限制）
        if let Some(max_size) = policy.max_total_size {
            if stats.total_size as u64 > max_size {
                let target_size = (max_size as f64 * 0.8) as u64; // 清理到80%
                let to_free = (stats.total_size as u64) - target_size;
                
                // 获取LRU候选项进行清理
                let candidates = self.cache_repo.find_cleanup_candidates(None, Some(100)).await?;
                let (cleaned, freed) = self.cleanup_candidates(candidates, to_free).await?;
                total_cleaned += cleaned;
                total_freed += freed;
            }
        }

        // 2. 按数量清理（如果超过限制）
        if let Some(max_count) = policy.max_entries {
            if stats.total_count as u64 > max_count {
                let to_remove = (stats.total_count as u64) - ((max_count as f64) * 0.8) as u64; // 清理到80%
                
                let candidates = self.cache_repo.find_cleanup_candidates(None, Some(to_remove)).await?;
                let (cleaned, freed) = self.cleanup_candidates(candidates, u64::MAX).await?;
                total_cleaned += cleaned;
                total_freed += freed;
            }
        }

        // 3. 按时间清理（删除过期项）
        if let Some(max_age) = policy.max_age {
            let max_age_seconds = max_age as i64;
            let candidates = self.cache_repo.find_cleanup_candidates(Some(max_age_seconds), None).await?;
            let (cleaned, freed) = self.cleanup_candidates(candidates, u64::MAX).await?;
            total_cleaned += cleaned;
            total_freed += freed;
        }

        // 4. 按热度评分清理（删除低热度项）- 仅在空间使用率超过阈值时执行
        if policy.min_heat_score.is_some() && self.should_cleanup_by_heat().await? {
            let candidates = self.cache_repo.cleanup_low_heat_caches().await?;
            let (cleaned, freed) = self.cleanup_candidates(candidates, u64::MAX).await?;
            total_cleaned += cleaned;
            total_freed += freed;
        }

        Ok(CacheCleanupResult {
            cleaned_count: total_cleaned,
            freed_space: total_freed,
            applied_policies: vec!["智能清理".to_string()],
            duration_ms: 0,
        })
    }

    /// 清理候选缓存项
    async fn cleanup_candidates(
        &self,
        candidates: Vec<CacheInfo>,
        max_free_bytes: u64,
    ) -> Result<(u64, u64), AppError> {
        let mut cleaned_count = 0;
        let mut freed_space = 0;

        for cache_info in candidates {
            if freed_space >= max_free_bytes {
                        break;
            }

            // 删除文件
            if let Err(e) = fs::remove_file(&cache_info.file_path).await {
                warn!("删除缓存文件失败: {} - {}", cache_info.file_path, e);
            } else {
                freed_space += cache_info.file_size;
            }

            // 删除数据库记录
            if let Err(e) = self.cache_repo.delete_by_key(&cache_info.cache_key).await {
                error!("删除缓存记录失败: {} - {}", cache_info.cache_key, e);
            } else {
                cleaned_count += 1;
            }
        }

        Ok((cleaned_count, freed_space))
    }

    /// 清理所有缓存
    pub async fn clear_all(&self) -> Result<CacheCleanupResult, AppError> {
        let stats = self.cache_repo.get_stats().await?;
        
        // 删除所有数据库记录
        let deleted_count = self.cache_repo.clear_all().await?;
        
        // 删除缓存目录
        if let Err(e) = fs::remove_dir_all(&self.cache_dir).await {
            warn!("删除缓存目录失败: {} - {}", self.cache_dir, e);
        }
        
        // 重新创建缓存目录
        if let Err(e) = fs::create_dir_all(&self.cache_dir).await {
            error!("重新创建缓存目录失败: {} - {}", self.cache_dir, e);
        }

        Ok(CacheCleanupResult {
            cleaned_count: deleted_count,
            freed_space: stats.total_size as u64,
            applied_policies: vec!["清理所有缓存".to_string()],
            duration_ms: 0,
        })
    }

    /// 获取缓存统计信息
    pub async fn get_stats(&self) -> Result<CacheStats, AppError> {
        self.cache_repo.get_stats().await
    }

    /// 根据原始哈希删除相关缓存
    pub async fn remove_by_original_hash(&self, original_hash: &str) -> Result<u64, AppError> {
        self.cache_repo.delete_by_original_hash(original_hash).await
    }

    /// 执行热度衰减
    /// 这个方法应该由定时任务定期调用
    pub async fn decay_heat_scores(&self) -> Result<u64, AppError> {
        info!("开始执行缓存热度衰减");
        let updated_count = self.cache_repo.decay_all_heat_scores().await?;
        info!("热度衰减完成，更新了 {} 个缓存项", updated_count);
        Ok(updated_count)
    }

    /// 智能清理：结合热度衰减的清理策略
    pub async fn smart_cleanup(&self) -> Result<CacheCleanupResult, AppError> {
        let start_time = std::time::Instant::now();
        let mut total_cleaned = 0;
        let mut total_freed = 0;
        let mut applied_policies = Vec::new();

        // 1. 首先执行热度衰减
        let decayed_count = self.decay_heat_scores().await?;
        if decayed_count > 0 {
            applied_policies.push(format!("热度衰减: 更新 {} 项", decayed_count));
        }

        // 2. 清理完全无热度的缓存（随时清理）
        let (cleaned, freed) = self.cleanup_zero_heat_caches().await?;
        if cleaned > 0 {
            total_cleaned += cleaned;
            total_freed += freed;
            applied_policies.push(format!("零热度清理: {} 项", cleaned));
        }

        // 3. 智能空间管理清理（仅在空间不足时清理低热度缓存）
        let (cleaned, freed) = self.cleanup_low_heat_caches_by_threshold().await?;
        if cleaned > 0 {
            total_cleaned += cleaned;
            total_freed += freed;
            applied_policies.push(format!("低热度清理: {} 项", cleaned));
        }

        // 4. 执行常规的策略清理
        let config = AppConfig::get();
        let policy = CacheCleanupPolicy::from_config(&config);
        let regular_result = self.cleanup_with_policy(&policy).await?;
        total_cleaned += regular_result.cleaned_count;
        total_freed += regular_result.freed_space;
        applied_policies.extend(regular_result.applied_policies);

        let duration_ms = start_time.elapsed().as_millis() as u64;

        Ok(CacheCleanupResult {
            cleaned_count: total_cleaned,
            freed_space: total_freed,
            applied_policies,
            duration_ms,
        })
    }

    /// 检查是否需要进行基于热度的清理
    /// 只有当空间使用率超过配置的阈值时才返回true
    pub async fn should_cleanup_by_heat(&self) -> Result<bool, AppError> {
        let config = AppConfig::get();
        let stats = self.cache_repo.get_stats().await?;
        
        // 计算当前空间使用率
        let space_usage_ratio = stats.total_size as f64 / config.cache.max_cache_size as f64;
        
        info!("当前缓存空间使用率: {:.1}% ({} / {} 字节)", 
            space_usage_ratio * 100.0, 
            stats.total_size, 
            config.cache.max_cache_size
        );
        
        let should_cleanup = space_usage_ratio >= config.cache.space_threshold_percent;
        
        if should_cleanup {
            info!("空间使用率 {:.1}% 超过阈值 {:.1}%，将触发基于热度的清理", 
                space_usage_ratio * 100.0, 
                config.cache.space_threshold_percent * 100.0
            );
        } else {
            info!("空间使用率 {:.1}% 未超过阈值 {:.1}%，跳过基于热度的清理", 
                space_usage_ratio * 100.0, 
                config.cache.space_threshold_percent * 100.0
            );
        }
        
        Ok(should_cleanup)
    }

    /// 清理完全没有热度的缓存（随时可清理）
    pub async fn cleanup_zero_heat_caches(&self) -> Result<(u64, u64), AppError> {
        // 获取低热度缓存候选项
        let low_heat_caches = self.cache_repo.cleanup_low_heat_caches().await?;
        
        // 筛选出完全无热度的缓存（随时可清理）
        let zero_heat_caches: Vec<CacheInfo> = low_heat_caches
            .into_iter()
            .filter(|cache| cache.heat_score <= 0.001) // 完全没有热度的缓存
            .collect();

        if zero_heat_caches.is_empty() {
            info!("没有找到完全无热度的缓存");
            return Ok((0, 0));
        }

        info!("找到 {} 个完全无热度的缓存，准备清理", zero_heat_caches.len());

        // 完全无热度的缓存可以全部清理，不受空间限制
        self.cleanup_candidates(zero_heat_caches, u64::MAX).await
    }

    /// 智能空间管理清理：在空间使用率超过阈值时清理低热度缓存
    pub async fn cleanup_low_heat_caches_by_threshold(&self) -> Result<(u64, u64), AppError> {
        // 检查是否需要执行基于热度的清理
        if !self.should_cleanup_by_heat().await? {
            return Ok((0, 0)); // 空间充足，不需要清理
        }
        
        let config = AppConfig::get();
        
        // 获取低热度缓存候选项
        let low_heat_caches = self.cache_repo.cleanup_low_heat_caches().await?;
        
        // 筛选出热度低于阈值但不为零的缓存
        let threshold_heat_caches: Vec<CacheInfo> = low_heat_caches
            .into_iter()
            .filter(|cache| cache.heat_score > 0.001 && cache.heat_score < config.cache.min_heat_score)
            .collect();

        if threshold_heat_caches.is_empty() {
            info!("没有找到低热度缓存（热度在0.001到{}之间）", config.cache.min_heat_score);
            return Ok((0, 0));
        }

        info!("找到 {} 个低热度缓存，准备清理", threshold_heat_caches.len());

        // 计算目标清理大小
        let target_usage = config.cache.space_threshold_percent * 0.9; // 清理到阈值的90%
        let target_size = (config.cache.max_cache_size as f64 * target_usage) as u64;
        let current_size = self.cache_repo.get_stats().await?.total_size as u64;
        
        let max_cleanup_size = if current_size > target_size {
            current_size - target_size
        } else {
            0
        };

        info!("当前大小: {} 字节, 目标大小: {} 字节, 最大清理: {} 字节", 
            current_size, target_size, max_cleanup_size);

        self.cleanup_candidates(threshold_heat_caches, max_cleanup_size).await
    }
} 