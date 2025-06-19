use axum::{
    routing::{get, post, delete},
    Router,
    http::Method,
    extract::{ConnectInfo, DefaultBodyLimit},
    middleware::{self, Next},
    response::Response,
};
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, Level};
use tracing_subscriber::{self, EnvFilter};
use std::net::SocketAddr;

mod config;
mod entities;
mod migrations;
mod models;
mod utils;
mod services;
mod handlers;

use handlers::{api_docs, health_check, upload_image, get_image, get_image_info, query_images_post, query_images_get, get_stats, delete_image};
use config::AppConfig;

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

    // 测试数据库连接
    match services::database_service::DatabaseService::new().await {
        Ok(_) => {
            info!("数据库连接测试成功");
        }
        Err(e) => {
            eprintln!("数据库连接失败: {}", e);
            std::process::exit(1);
        }
    }

    // 创建路由
    let mut app = Router::new()
        // API文档根路径
        .route("/", get(api_docs))
        
        // 健康检查
        .route("/health", get(health_check))
        
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
    info!("  API文档:  GET  /");
    info!("  健康检查: GET  /health");
    info!("  上传图片: POST /upload");
    info!("  获取图片: GET  /images/<filename>");
    info!("  图片信息: GET  /images/<filename>/info");
    info!("  查询列表: GET/POST /api/images/query");
    info!("  统计信息: GET  /api/stats");
    info!("  删除图片: DEL  /images/<filename>");

    // 启动服务器并启用ConnectInfo以获取客户端IP
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .expect("服务器启动失败");
} 