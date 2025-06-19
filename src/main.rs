use axum::{
    routing::{get, post, delete},
    Router,
    http::Method,
};
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, Level};
use tracing_subscriber;

mod config;
mod models;
mod utils;
mod services;
mod handlers;

use handlers::{health_check, upload_image, get_image, get_image_info, delete_image};
use config::AppConfig;

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

    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .init();

    info!("启动图床服务...");
    info!("使用配置: {:#?}", config);

    // 确保上传目录存在
    if let Err(e) = utils::ensure_upload_dir().await {
        eprintln!("创建上传目录失败: {}", e);
        std::process::exit(1);
    }

    // 创建路由
    let mut app = Router::new()
        // 健康检查
        .route("/health", get(health_check))
        
        // 图片上传
        .route("/upload", post(upload_image))
        
        // 获取图片 - 直接返回图片数据
        .route("/images/{filename}", get(get_image))
        
        // 获取图片信息 - 返回JSON格式的图片元数据
        .route("/images/{filename}/info", get(get_image_info))
        
        // 删除图片
        .route("/images/{filename}", delete(delete_image));

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
    info!("  健康检查: GET  /health");
    info!("  上传图片: POST /upload");
    info!("  获取图片: GET  /images/<filename>");
    info!("  图片信息: GET  /images/<filename>/info");
    info!("  删除图片: DEL  /images/<filename>");

    // 启动服务器
    axum::serve(listener, app)
        .await
        .expect("服务器启动失败");
} 