use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::signal;

use tracing::{info, error, warn};

use crate::app_state::AppState;
use crate::utils::AppError;

/// 关闭管理器
pub struct ShutdownManager {
    app_state: Arc<AppState>,
    shutdown_signal: Arc<AtomicBool>,
}

impl ShutdownManager {
    /// 创建新的关闭管理器
    pub fn new(app_state: AppState) -> Self {
        Self {
            app_state: Arc::new(app_state),
            shutdown_signal: Arc::new(AtomicBool::new(false)),
        }
    }

    /// 获取关闭信号标志
    pub fn shutdown_signal(&self) -> Arc<AtomicBool> {
        self.shutdown_signal.clone()
    }

    /// 等待关闭信号
    pub async fn wait_for_shutdown_signal(&self) {
        let ctrl_c = async {
            signal::ctrl_c()
                .await
                .expect("无法安装 Ctrl+C 信号处理器");
        };

        #[cfg(unix)]
        let terminate = async {
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("无法安装 SIGTERM 信号处理器")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {
                info!("接收到 Ctrl+C 信号，开始关闭程序...");
                self.shutdown_signal.store(true, Ordering::SeqCst);
            },
            _ = terminate => {
                info!("接收到 SIGTERM 信号，开始关闭程序...");
                self.shutdown_signal.store(true, Ordering::SeqCst);
            },
        }
    }

    /// 执行程序关闭
    pub async fn shutdown(&self) -> Result<(), AppError> {
        info!("开始执行程序关闭流程...");
        
        let mut errors = Vec::new();

        // 1. 标记正在关闭
        self.shutdown_signal.store(true, Ordering::SeqCst);

        // 2. 执行应用状态关闭
        if let Err(e) = self.app_state.shutdown().await {
            error!("应用状态关闭失败: {}", e);
            errors.push(format!("应用状态关闭失败: {}", e));
        }

        // 3. 清理临时文件
        if let Err(e) = self.cleanup_temp_files().await {
            warn!("临时文件清理失败: {}", e);
            errors.push(format!("临时文件清理失败: {}", e));
        }

        // 如果有错误但不是关键错误，只警告
        if !errors.is_empty() {
            warn!("关闭过程中发生了一些非关键错误: {:?}", errors);
        }

        info!("程序关闭流程完成");
        Ok(())
    }

    /// 清理临时文件
    async fn cleanup_temp_files(&self) -> Result<(), AppError> {
        info!("清理临时文件...");
        
        // 清理可能的临时上传文件
        let temp_patterns = [
            "*.tmp",
            "*.temp", 
            ".upload_*",
        ];

        let config = self.app_state.config();
        let upload_dir = std::path::Path::new(&config.storage.upload_dir);
        
        for pattern in &temp_patterns {
            if let Ok(entries) = tokio::fs::read_dir(upload_dir).await {
                let mut entries = entries;
                while let Ok(Some(entry)) = entries.next_entry().await {
                    let path = entry.path();
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if pattern.replace("*", "").chars().any(|c| name.contains(c)) {
                            if let Err(e) = tokio::fs::remove_file(&path).await {
                                warn!("删除临时文件失败 {}: {}", path.display(), e);
                            } else {
                                info!("删除临时文件: {}", path.display());
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

/// 创建关闭信号处理任务
pub async fn create_shutdown_signal() -> impl std::future::Future<Output = ()> {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("无法安装 Ctrl+C 信号处理器");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("无法安装 SIGTERM 信号处理器")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    async move {
        tokio::select! {
            _ = ctrl_c => {
                info!("接收到 Ctrl+C 信号");
            },
            _ = terminate => {
                info!("接收到 SIGTERM 信号");
            },
        }
    }
}

 