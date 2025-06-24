use axum::{
    routing::{get, post, delete},
    Router,
    http::Method,
    extract::{ConnectInfo, DefaultBodyLimit},
    middleware::{self, Next},
    response::Response,

};
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, error, Level};
use tracing_subscriber::{self, EnvFilter};
use std::net::SocketAddr;

mod config;
mod entities;
mod migrations;
mod models;
mod utils;
mod services;
mod handlers;
mod database;
mod repositories;
mod app_state;

use handlers::{
    api_docs, upload_image, get_image, get_image_info, 
    query_images_post, query_images_get, get_stats, delete_image,
    get_cache_stats, auto_cleanup_cache, cleanup_cache_with_policy,
    clear_all_cache, cache_management_dashboard, 
    health_check_detailed, get_system_stats, smart_cleanup, decay_heat_scores,
    smart_space_cleanup,
};
use config::AppConfig;
use app_state::AppState;
use utils::{create_shutdown_signal, ShutdownManager};

/// 简单的HTTP请求日志中间件
async fn log_requests(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: axum::http::Request<axum::body::Body>,
    next: Next,
) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let client_ip = addr.ip();
    
    let start = std::time::Instant::now();
    let response = next.run(request).await;
    let latency = start.elapsed();
    
    // 根据延迟时间选择合适的单位
    let (time_value, time_unit) = if latency.as_millis() >= 1 {
        (latency.as_millis(), "ms")
    } else if latency.as_micros() >= 1 {
        (latency.as_micros(), "µs")
    } else {
        (latency.as_nanos() as u128, "ns")
    };
    
    // 根据状态码选择背景色和前景色（加粗）
    let status_code = response.status().as_u16();
    let status_display = match status_code {
        200..=299 => format!("\x1b[1;30;42m {:>3} \x1b[0m", status_code), // 加粗黑字绿底
        300..=399 => format!("\x1b[1;30;43m {:>3} \x1b[0m", status_code), // 加粗黑字黄底
        400..=499 => format!("\x1b[1;37;41m {:>3} \x1b[0m", status_code), // 加粗白字红底
        500..=599 => format!("\x1b[1;37;45m {:>3} \x1b[0m", status_code), // 加粗白字紫底
        _ => format!(" {:>3} ", status_code),                              // 无色
    };
    
    info!(
        "{} {:>4} | {:<15} | {:>4}{} | {}",
        status_display,
        method,
        client_ip,
        time_value,
        time_unit,
        uri
    );
    
    response
}

#[tokio::main]
async fn main() {
    // 初始化配置
    if let Err(e) = AppConfig::init(None) {
        eprintln!("配置初始化失败: {}", e);
        std::process::exit(1);
    }

    let config = AppConfig::get();

    // 初始化日志
    let log_level = match config.logging.level.to_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };

    // 创建过滤器，隐藏数据库查询日志
    let filter = EnvFilter::new(format!("rifs={},[sqlx::query]=off,[sea_orm_migration::migrator]=off", 
        config.logging.level.to_lowercase()));

    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_env_filter(filter)
        .init();

    info!("启动图床服务...");
    info!("使用配置: {:#?}", config);

    // 确保上传目录存在
    if let Err(e) = utils::ensure_upload_dir().await {
        eprintln!("创建上传目录失败: {}", e);
        std::process::exit(1);
    }

    // 初始化应用状态
    let app_state = match AppState::new().await {
        Ok(state) => {
            info!("应用状态初始化成功");
            state
        }
        Err(e) => {
            eprintln!("应用状态初始化失败: {}", e);
            std::process::exit(1);
        }
    };

    // 创建关闭管理器
    let shutdown_manager = ShutdownManager::new(app_state.clone());
    let shutdown_signal = shutdown_manager.shutdown_signal();

    // 启动缓存自动清理任务
    let cleanup_task = if config.cache.enable_transform_cache {
        let app_state_for_cleanup = app_state.clone();
        let cleanup_interval = config.cache.auto_cleanup_interval;
        let shutdown_signal_for_cleanup = shutdown_signal.clone();
        
        Some(tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(cleanup_interval));
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        // 检查是否需要关闭
                        if shutdown_signal_for_cleanup.load(std::sync::atomic::Ordering::SeqCst) {
                            info!("缓存清理任务检测到关闭信号，正在停止...");
                            break;
                        }

                        let db_connection = app_state_for_cleanup.db_pool().get_connection();
                        match services::cache_service::CacheService::new(db_connection) {
                            Ok(cache_service) => {
                                // 确保缓存目录存在
                                if let Err(e) = cache_service.ensure_cache_dir().await {
                                    error!("创建缓存目录失败: {}", e);
                                    continue;
                                }
                                
                                match cache_service.smart_cleanup().await {
                                    Ok(result) => {
                                        if result.cleaned_count > 0 || !result.applied_policies.is_empty() {
                                            info!("智能缓存清理完成: 删除{}个缓存，释放{}字节，耗时{}ms", 
                                                result.cleaned_count, result.freed_space, result.duration_ms);
                                            info!("应用的策略: {:?}", result.applied_policies);
                                        }
                                    }
                                    Err(e) => {
                                        error!("智能缓存清理失败: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                error!("创建缓存服务失败: {}", e);
                            }
                        }
                    }
                    // 接收到取消信号时退出循环  
                    _ = tokio::signal::ctrl_c() => {
                        info!("缓存清理任务收到关闭信号，正在停止...");
                        break;
                    }
                }
            }
            info!("缓存清理任务已停止");
        }))
    } else {
        None
    };

    if cleanup_task.is_some() {
        info!("缓存自动清理任务已启动，间隔: {}秒", config.cache.auto_cleanup_interval);
    }

    // 创建路由
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
        .route("/api/images/query", get(query_images_get).post(query_images_post))
        
        // 获取统计信息
        .route("/api/stats", get(get_stats))
        
        // 删除图片
        .route("/images/{filename}", delete(delete_image))
        
        // 缓存管理接口
        .route("/api/cache/stats", get(get_cache_stats))
        .route("/api/cache/cleanup/auto", post(auto_cleanup_cache))
        .route("/api/cache/cleanup/policy", post(cleanup_cache_with_policy))
        .route("/api/cache/cleanup/smart", post(smart_cleanup))
        .route("/api/cache/cleanup/space", post(smart_space_cleanup))
        .route("/api/cache/decay", post(decay_heat_scores))
        .route("/api/cache/clear", delete(clear_all_cache))
        .route("/cache/management", get(cache_management_dashboard))
        
        // 注入应用状态
        .with_state(app_state.clone())
        
        // 添加文件大小限制中间件
        .layer(DefaultBodyLimit::max(config.storage.max_file_size as usize))
        
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

    // 绑定地址
    let address = config.server_address();
    let listener = tokio::net::TcpListener::bind(&address)
        .await
        .expect("绑定端口失败");

    info!("图床服务已启动，监听地址: http://{}", address);
    info!("API接口:");
    info!("  API文档:  GET      /");
    info!("  健康检查: GET      /health");
    info!("  上传图片: POST     /upload");
    info!("  获取图片: GET      /images/<filename>");
    info!("  图片信息: GET      /images/<filename>/info");
    info!("  查询列表: GET/POST /api/images/query");
    info!("  统计信息: GET      /api/stats");
    info!("  删除图片: DEL      /images/<filename>");
    info!("  缓存管理: GET      /cache/management");
    info!("  缓存统计: GET      /api/cache/stats");
    info!("  缓存清理: POST     /api/cache/cleanup/auto");
    info!("  智能清理: POST     /api/cache/cleanup/smart");
    info!("  空间清理: POST     /api/cache/cleanup/space");
    info!("  热度衰减: POST     /api/cache/decay");

    // 启动服务器并启用ConnectInfo以获取客户端IP
    let server = axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(create_shutdown_signal().await);

    info!("服务器已启动，按 Ctrl+C 关闭程序");

    // 等待服务器结束或关闭信号
    tokio::select! {
        result = server => {
            match result {
                Ok(_) => info!("服务器正常结束"),
                Err(e) => error!("服务器运行错误: {}", e),
            }
        }
        _ = shutdown_manager.wait_for_shutdown_signal() => {
            info!("接收到关闭信号");
        }
    }
    
    // 停止缓存清理任务
    if let Some(task) = cleanup_task {
        info!("正在停止缓存清理任务...");
        task.abort();
        // 等待任务结束（如果它还在运行）
        let _ = task.await;
        info!("缓存清理任务已停止");
    }
    
    // 执行程序关闭
    if let Err(e) = shutdown_manager.shutdown().await {
        error!("程序关闭失败: {}", e);
        std::process::exit(1);
    }
    
    info!("应用程序已完全关闭");
} 