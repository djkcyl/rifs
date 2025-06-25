use axum::{
    extract::State,
    response::{IntoResponse, Json},
};
use serde_json::json;
use tracing::info;

use crate::app_state::AppState;
use crate::utils::AppError;

/// 详细健康检查接口（包含数据库和组件状态）
pub async fn health_check_detailed(
    State(app_state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    info!("执行详细健康检查");

    // 执行应用程序健康检查
    let health_status = app_state.health_check().await?;

    // 获取应用统计信息
    let app_stats = app_state.get_app_stats().await?;

    let response = json!({
        "status": if health_status.is_healthy() { "healthy" } else { "unhealthy" },
        "message": "详细健康检查完成",
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        "components": {
            "database": match health_status.database {
                crate::app_state::ComponentStatus::Healthy => "healthy",
                crate::app_state::ComponentStatus::Unhealthy(_) => "unhealthy",
                crate::app_state::ComponentStatus::Unknown => "unknown",
            },
            "overall": match health_status.overall {
                crate::app_state::OverallStatus::Healthy => "healthy",
                crate::app_state::OverallStatus::Unhealthy => "unhealthy",
            }
        },
        "stats": {
            "database_pool": {
                "max_connections": app_stats.database_pool.max_connections,
                "min_connections": app_stats.database_pool.min_connections,
                "utilization_rate": app_stats.database_utilization(),
            }
        }
    });

    Ok(Json(response))
}

/// 系统状态统计接口
pub async fn get_system_stats(
    State(app_state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let health_status = app_state.health_check().await?;
    let app_stats = app_state.get_app_stats().await?;
    let config = app_state.config();

    let response = json!({
        "success": true,
        "message": "系统统计信息获取成功",
        "data": {
            "health": {
                "overall": match health_status.overall {
                    crate::app_state::OverallStatus::Healthy => "healthy",
                    crate::app_state::OverallStatus::Unhealthy => "unhealthy",
                },
                "database": match health_status.database {
                    crate::app_state::ComponentStatus::Healthy => "healthy",
                    crate::app_state::ComponentStatus::Unhealthy(ref _error) => "unhealthy",
                    crate::app_state::ComponentStatus::Unknown => "unknown",
                }
            },
            "database_pool": {
                "max_connections": app_stats.database_pool.max_connections,
                "min_connections": app_stats.database_pool.min_connections,
                "active_connections": app_stats.database_pool.active_connections,
                "idle_connections": app_stats.database_pool.idle_connections,
                "utilization_rate": app_stats.database_utilization(),
            },
            "config": {
                "server": {
                    "host": config.server.host,
                    "port": config.server.port,
                    "enable_cors": config.server.enable_cors,
                },
                "storage": {
                    "upload_dir": config.storage.upload_dir,
                    "max_file_size": config.storage.max_file_size.as_bytes(),
                },
                "cache": {
                    "enable_transform_cache": config.cache.enable_transform_cache,
                    "max_cache_entries": config.cache.max_cache_entries,
                    "max_cache_size": config.cache.max_cache_size.as_bytes(),
                }
            },
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        }
    });

    Ok(Json(response))
}
