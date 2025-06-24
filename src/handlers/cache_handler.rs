use axum::{
    extract::State,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};

use crate::app_state::AppState;
use crate::models::{CacheCleanupPolicy, CacheCleanupResult};
use crate::services::CacheService;
use crate::utils::AppError;
use crate::handlers::static_files::CACHE_MANAGEMENT_HTML;

/// 通用API响应结构
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    pub fn success(message: &str, data: Option<T>) -> Self {
        Self {
            success: true,
            message: message.to_string(),
            data,
        }
    }
}

/// 缓存管理响应
#[derive(Debug, Serialize)]
pub struct CacheManagementResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
}

/// 缓存清理请求参数
#[derive(Debug, Deserialize)]
pub struct CacheCleanupRequest {
    /// 缓存最大数量
    pub max_entries: Option<u64>,
    /// 缓存最大总大小（字节）
    pub max_total_size: Option<u64>,
    /// 缓存项最大生存时间（秒）
    pub max_age: Option<u64>,
    /// 最小热度评分阈值
    pub min_heat_score: Option<f64>,
    /// 是否启用LRU清理
    pub enable_lru: Option<bool>,
}

impl From<CacheCleanupRequest> for CacheCleanupPolicy {
    fn from(req: CacheCleanupRequest) -> Self {
        Self {
            max_entries: req.max_entries,
            max_total_size: req.max_total_size,
            max_age: req.max_age,
            min_heat_score: req.min_heat_score,
            enable_lru: req.enable_lru.unwrap_or(true),
        }
    }
}

/// 获取缓存统计信息
pub async fn get_cache_stats(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let db_connection = state.db_pool().get_connection();
    let cache_service = CacheService::new(db_connection)?;
    
    let stats = cache_service.get_stats().await?;
    
    Ok(Json(ApiResponse::success("获取缓存统计成功", Some(stats))))
}

/// 执行热度衰减
pub async fn decay_heat_scores(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let db_connection = state.db_pool().get_connection();
    let cache_service = CacheService::new(db_connection)?;
    
    let updated_count = cache_service.decay_heat_scores().await?;
    
    Ok(Json(ApiResponse::success(
        &format!("热度衰减完成，更新了 {} 个缓存项", updated_count),
        Some(updated_count)
    )))
}

/// 智能缓存清理（包含热度衰减）
pub async fn smart_cleanup(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let db_connection = state.db_pool().get_connection();
    let cache_service = CacheService::new(db_connection)?;
    
    let result = cache_service.smart_cleanup().await?;
    
    Ok(Json(ApiResponse::success("智能清理完成", Some(result))))
}

/// 智能空间管理清理：分层清理策略
pub async fn smart_space_cleanup(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let db_connection = state.db_pool().get_connection();
    let cache_service = CacheService::new(db_connection)?;
    
    let mut total_cleaned = 0;
    let mut total_freed = 0;
    let mut applied_policies = Vec::new();
    
    // 1. 清理完全无热度的缓存（总是执行）
    let (cleaned, freed) = cache_service.cleanup_zero_heat_caches().await?;
    if cleaned > 0 {
        total_cleaned += cleaned;
        total_freed += freed;
        applied_policies.push(format!("零热度清理: {} 项", cleaned));
    }
    
    // 2. 清理低热度缓存（仅在空间不足时）
    let (cleaned, freed) = cache_service.cleanup_low_heat_caches_by_threshold().await?;
    if cleaned > 0 {
        total_cleaned += cleaned;
        total_freed += freed;
        applied_policies.push(format!("低热度清理: {} 项", cleaned));
    }
    
    let result = crate::models::CacheCleanupResult {
        cleaned_count: total_cleaned,
        freed_space: total_freed,
        applied_policies,
        duration_ms: 0,
    };
    
    Ok(Json(ApiResponse::success(
        if total_cleaned > 0 {
            "智能空间管理清理完成"
        } else {
            "无需清理"
        },
        Some(result)
    )))
}

/// 自动清理缓存
pub async fn auto_cleanup_cache(
    State(app_state): State<AppState>
) -> Result<Json<ApiResponse<CacheCleanupResult>>, AppError> {
    let connection = app_state.db_pool().get_connection();
    let cache_service = CacheService::new(connection)?;
    
    let result = cache_service.auto_cleanup().await?;
    
    Ok(Json(ApiResponse::success("自动清理完成", Some(result))))
}

/// 使用策略清理缓存
pub async fn cleanup_cache_with_policy(
    State(app_state): State<AppState>,
    Json(policy): Json<CacheCleanupPolicy>
) -> Result<Json<ApiResponse<CacheCleanupResult>>, AppError> {
    let connection = app_state.db_pool().get_connection();
    let cache_service = CacheService::new(connection)?;
    
    let result = cache_service.cleanup_with_policy(&policy).await?;
    
    Ok(Json(ApiResponse::success("策略清理完成", Some(result))))
}

/// 清空所有缓存
pub async fn clear_all_cache(
    State(app_state): State<AppState>
) -> Result<Json<ApiResponse<CacheCleanupResult>>, AppError> {
    let connection = app_state.db_pool().get_connection();
    let cache_service = CacheService::new(connection)?;
    
    let result = cache_service.clear_all().await?;
    
    Ok(Json(ApiResponse::success("清理完成", Some(result))))
}

/// 缓存管理面板（返回HTML页面）
pub async fn cache_management_dashboard() -> impl IntoResponse {
    axum::response::Html(CACHE_MANAGEMENT_HTML)
} 