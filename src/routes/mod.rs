use axum::{
    extract::DefaultBodyLimit,
    http::Method,
    middleware,
    routing::{delete, get, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};

use crate::app_state::AppState;
use crate::config::AppConfig;
use crate::handlers::{
    api_docs, auto_cleanup_cache, cache_management_dashboard, 
    clear_all_cache, decay_heat_scores, delete_image, get_cache_stats, get_image, get_image_info,
    get_stats, get_system_stats, health_check_detailed, query_images_get, query_images_post,
    upload_image,
};
use crate::middleware::{log_requests, request_timeout};

/// 创建应用路由
pub fn create_routes(app_state: AppState, config: &AppConfig) -> Router {
    let mut app = Router::new()
        // API文档根路径
        .route("/", get(api_docs))
        // 健康检查
        .route("/health", get(health_check_detailed))
        .route("/health/detailed", get(health_check_detailed))
        // 系统管理接口
        .route("/api/system/stats", get(get_system_stats))
        // 图片上传
        .route("/upload", post(upload_image))
        // 获取图片 - 直接返回图片数据
        .route("/images/{filename}", get(get_image))
        // 获取图片信息 - 返回JSON格式的图片元数据
        .route("/images/{filename}/info", get(get_image_info))
        // 查询图片列表 - 同时支持GET和POST
        .route(
            "/api/images/query",
            get(query_images_get).post(query_images_post),
        )
        // 获取统计信息
        .route("/api/stats", get(get_stats))
        // 删除图片
        .route("/images/{filename}", delete(delete_image))
        // 缓存管理接口（简化版）
        .route("/api/cache/stats", get(get_cache_stats))
        .route("/api/cache/cleanup/auto", post(auto_cleanup_cache))
        .route("/api/cache/decay", post(decay_heat_scores))
        .route("/api/cache/clear", delete(clear_all_cache))
        .route("/cache/management", get(cache_management_dashboard))
        // 注入应用状态
        .with_state(app_state.clone())
        // 添加文件大小限制中间件
        .layer(DefaultBodyLimit::max(
            config.storage.max_file_size.as_bytes() as usize,
        ))
        // 添加请求超时中间件
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            request_timeout,
        ))
        // 添加自定义请求日志中间件
        .layer(middleware::from_fn(log_requests));

    // 添加CORS中间件（如果启用）
    if config.server.enable_cors {
        let cors = CorsLayer::new()
            .allow_methods([Method::GET, Method::POST, Method::DELETE])
            .allow_headers(Any)
            .allow_origin(Any);

        app = app.layer(cors);
    }

    app
}
