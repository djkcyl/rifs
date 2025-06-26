use std::net::SocketAddr;
use tokio::task::JoinHandle;
use tracing::{error, info};

use crate::app_state::AppState;
use crate::config::AppConfig;
use crate::logging;
use crate::routes::create_routes;
use crate::services;
use crate::utils;

/// 启动缓存清理任务
pub fn start_cache_cleanup_task(app_state: AppState, config: &AppConfig) -> Option<JoinHandle<()>> {
    if !config.cache.enable_transform_cache {
        return None;
    }

    let app_state_for_cleanup = app_state.clone();
    let cleanup_interval = config.cache.auto_cleanup_interval.as_seconds();

    Some(tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(cleanup_interval));
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    let db_connection = app_state_for_cleanup.db_pool().get_connection();
                    match services::cache_service::CacheService::new(db_connection) {
                        Ok(cache_service) => {
                            if let Err(e) = cache_service.ensure_cache_dir().await {
                                error!("创建缓存目录失败: {}", e);
                                continue;
                            }

                            match cache_service.auto_cleanup().await {
                                Ok(result) => {
                                    if result.cleaned_count > 0 || !result.applied_policies.is_empty() {
                                        info!("自动缓存清理完成: 删除{}个缓存，释放{}字节，耗时{}ms",
                                            result.cleaned_count, result.freed_space, result.duration_ms);
                                    }
                                }
                                Err(e) => {
                                    error!("自动缓存清理失败: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            error!("创建缓存服务失败: {}", e);
                        }
                    }
                }
                _ = tokio::signal::ctrl_c() => {
                    break;
                }
            }
        }
    }))
}

/// 打印API接口信息
pub fn print_api_info() {
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
    info!("  热度衰减: POST     /api/cache/decay");
    info!("  清空缓存: DEL      /api/cache/clear");
}

/// 运行服务器
pub async fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化配置
    if let Err(e) = AppConfig::init(None) {
        eprintln!("配置初始化失败: {}", e);
        std::process::exit(1);
    }

    let config = AppConfig::get();

    // 初始化日志
    logging::init_logging(config);

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

    // 启动缓存自动清理任务
    let cleanup_task = start_cache_cleanup_task(app_state.clone(), config);

    // 创建路由
    let app = create_routes(app_state, config);

    // 绑定地址
    let address = config.server_address();
    let listener = tokio::net::TcpListener::bind(&address)
        .await
        .expect("绑定端口失败");

    info!("图床服务已启动，监听地址: http://{}", address);
    print_api_info();

    // 启动服务器并启用ConnectInfo以获取客户端IP
    let server = axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(async {
        tokio::signal::ctrl_c()
            .await
            .expect("无法安装 Ctrl+C 信号处理器");
    });

    info!("服务器已启动，按 Ctrl+C 关闭程序");

    // 运行服务器
    if let Err(e) = server.await {
        error!("服务器运行错误: {}", e);
    }

    // 停止缓存清理任务
    if let Some(task) = cleanup_task {
        task.abort();
    }

    Ok(())
}
