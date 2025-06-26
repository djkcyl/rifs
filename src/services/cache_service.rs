use chrono::Utc;
use sea_orm::DatabaseConnection;
use sha2::{Digest, Sha256};
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tracing::{error, info, warn};

use crate::config::AppConfig;
use crate::models::{CacheCleanupResult, CacheInfo, CacheStats, ImageTransformParams};
use crate::repositories::{CacheRepository, CacheRepositoryTrait};
use crate::utils::AppError;

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
        tokio::fs::create_dir_all(&self.cache_dir)
            .await
            .map_err(|e| AppError::Internal(format!("创建缓存目录失败: {}", e)))?;
        Ok(())
    }

    /// 生成缓存键
    /// 使用原始hash和标准化的转换参数生成一致的缓存键
    pub fn generate_cache_key(original_hash: &str, transform_params: &ImageTransformParams) -> String {
        let normalized_params = transform_params.to_normalized_string();
        let mut hasher = Sha256::new();
        hasher.update(original_hash.as_bytes());
        hasher.update(b":");
        hasher.update(normalized_params.as_bytes());
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

        format!(
            "{}/{}/{}/{}.{}",
            self.cache_dir, prefix, middle, cache_key, ext
        )
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
        let data = fs::read(&cache_info.file_path)
            .await
            .map_err(|e| AppError::Internal(format!("读取缓存文件失败: {}", e)))?;

        // 更新访问信息
        self.cache_repo.update_access(&cache_info.cache_key).await?;

        Ok(data)
    }

    /// 保存缓存
    pub async fn save_cache(
        &self,
        original_hash: &str,
        transform_params: &ImageTransformParams,
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
        let normalized_params_str = transform_params.to_normalized_string();

        // 确保缓存子目录存在
        if let Some(parent) = Path::new(&file_path).parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| AppError::Internal(format!("创建缓存目录失败: {}", e)))?;
        }

        // 写入缓存文件
        fs::write(&file_path, data)
            .await
            .map_err(|e| AppError::Internal(format!("写入缓存文件失败: {}", e)))?;

        // 创建缓存信息
        let now = Utc::now();
        let cache_info = CacheInfo {
            cache_key: cache_key.clone(),
            original_hash: original_hash.to_string(),
            transform_params: normalized_params_str,
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

        Ok(cache_info)
    }

    /// 自动清理缓存（主要清理接口）
    /// 只在空间使用率达到阈值时执行清理
    pub async fn auto_cleanup(&self) -> Result<CacheCleanupResult, AppError> {
        let start_time = std::time::Instant::now();
        let config = AppConfig::get();
        
        let stats = self.get_stats().await?;
        let space_usage_ratio = stats.total_size as f64 / config.cache.max_cache_size.as_bytes() as f64;
        
        info!(
            "当前缓存空间使用率: {:.1}% ({} / {} 字节)",
            space_usage_ratio * 100.0,
            stats.total_size,
            config.cache.max_cache_size.as_bytes()
        );

        // 如果未达到阈值，不执行清理
        if space_usage_ratio < config.cache.space_threshold_percent {
            info!(
                "空间使用率 {:.1}% 未达到阈值 {:.1}%，跳过清理",
                space_usage_ratio * 100.0,
                config.cache.space_threshold_percent * 100.0
            );
            return Ok(CacheCleanupResult {
                cleaned_count: 0,
                freed_space: 0,
                applied_policies: vec!["无需清理".to_string()],
                duration_ms: start_time.elapsed().as_millis() as u64,
            });
        }

        info!(
            "空间使用率 {:.1}% 达到阈值 {:.1}%，开始执行清理",
            space_usage_ratio * 100.0,
            config.cache.space_threshold_percent * 100.0
        );

        let mut total_cleaned = 0;
        let mut total_freed = 0;
        let mut applied_policies = Vec::new();

        // 1. 首先执行热度衰减
        let decayed_count = self.decay_heat_scores().await?;
        if decayed_count > 0 {
            applied_policies.push(format!("热度衰减: 更新 {} 项", decayed_count));
        }

        // 2. 清理完全无热度的缓存（同时考虑最大生存时间）
        let (cleaned, freed) = self.cleanup_zero_heat_caches().await?;
        if cleaned > 0 {
            total_cleaned += cleaned;
            total_freed += freed;
            applied_policies.push(format!("零热度清理: {} 项", cleaned));
        }

        // 3. 检查是否还需要继续清理
        let updated_stats = self.get_stats().await?;
        let updated_usage_ratio = updated_stats.total_size as f64 / config.cache.max_cache_size.as_bytes() as f64;
        
        if updated_usage_ratio >= config.cache.space_threshold_percent {
            // 4. 清理低热度缓存直到达到目标使用率
            let (cleaned, freed) = self.cleanup_low_heat_caches().await?;
            if cleaned > 0 {
                total_cleaned += cleaned;
                total_freed += freed;
                applied_policies.push(format!("低热度清理: {} 项", cleaned));
            }
        }

        let duration_ms = start_time.elapsed().as_millis() as u64;

        Ok(CacheCleanupResult {
            cleaned_count: total_cleaned,
            freed_space: total_freed,
            applied_policies,
            duration_ms,
        })
    }

    /// 清理完全没有热度的缓存（同时考虑最大生存时间）
    async fn cleanup_zero_heat_caches(&self) -> Result<(u64, u64), AppError> {
        let config = AppConfig::get();
        let max_age_seconds = config.cache.max_cache_age.as_seconds() as i64;
        
        // 获取所有低热度缓存候选项
        let candidates = self.cache_repo.cleanup_low_heat_caches().await?;
        
        // 筛选出需要清理的缓存：
        // 1. 完全无热度的缓存（heat_score <= 0.001）
        // 2. 或者超过最大生存时间且无热度的缓存
        let now = Utc::now();
        let zero_heat_caches: Vec<CacheInfo> = candidates
            .into_iter()
            .filter(|cache| {
                let age_seconds = (now - cache.created_at).num_seconds();
                let is_zero_heat = cache.heat_score <= 0.001;
                let is_expired_and_no_heat = age_seconds > max_age_seconds && cache.heat_score <= 0.001;
                
                is_zero_heat || is_expired_and_no_heat
            })
            .collect();

        if zero_heat_caches.is_empty() {
            info!("没有找到需要清理的零热度缓存");
            return Ok((0, 0));
        }

        info!(
            "找到 {} 个零热度缓存，准备清理",
            zero_heat_caches.len()
        );

        self.cleanup_candidates(zero_heat_caches).await
    }

    /// 清理低热度缓存
    async fn cleanup_low_heat_caches(&self) -> Result<(u64, u64), AppError> {
        let config = AppConfig::get();
        
        // 计算目标清理大小
        let target_usage = config.cache.space_threshold_percent * 0.8; // 清理到阈值的80%
        let target_size = (config.cache.max_cache_size.as_bytes() as f64 * target_usage) as u64;
        let current_stats = self.get_stats().await?;
        let current_size = current_stats.total_size as u64;

        if current_size <= target_size {
            info!("当前大小已达到目标，无需清理低热度缓存");
            return Ok((0, 0));
        }

        let need_to_free = current_size - target_size;

        // 获取低热度缓存候选项（排除完全无热度的）
        let candidates = self.cache_repo.cleanup_low_heat_caches().await?;
        let low_heat_caches: Vec<CacheInfo> = candidates
            .into_iter()
            .filter(|cache| {
                cache.heat_score > 0.001 && cache.heat_score < config.cache.min_heat_score
            })
            .collect();

        if low_heat_caches.is_empty() {
            info!("没有找到需要清理的低热度缓存");
            return Ok((0, 0));
        }

        info!(
            "找到 {} 个低热度缓存，目标释放 {} 字节",
            low_heat_caches.len(),
            need_to_free
        );

        // 按热度排序，优先清理热度最低的
        let mut sorted_caches = low_heat_caches;
        sorted_caches.sort_by(|a, b| a.heat_score.partial_cmp(&b.heat_score).unwrap());

        // 选择要清理的缓存，直到达到目标释放空间
        let mut to_cleanup = Vec::new();
        let mut will_free = 0;
        
        for cache in sorted_caches {
            to_cleanup.push(cache.clone());
            will_free += cache.file_size;
            
            if will_free >= need_to_free {
                break;
            }
        }

        info!("将清理 {} 个低热度缓存，预计释放 {} 字节", to_cleanup.len(), will_free);

        self.cleanup_candidates(to_cleanup).await
    }

    /// 清理指定的缓存候选项
    async fn cleanup_candidates(&self, candidates: Vec<CacheInfo>) -> Result<(u64, u64), AppError> {
        let mut cleaned_count = 0;
        let mut freed_space = 0;

        for cache_info in candidates {
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
}
