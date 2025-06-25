pub mod rotating_writer;

pub use rotating_writer::HybridRotatingWriter;

use std::io::Write;
use std::path::Path;
use tracing::info;
use tracing_appender::non_blocking;
use tracing_subscriber::{self, fmt, prelude::*, EnvFilter};

use crate::config::AppConfig;

/// 初始化日志系统
pub fn init_logging(config: &AppConfig) {
    // 创建过滤器，包含日志级别
    let filter = EnvFilter::new(format!("rifs={}", config.logging.level.to_lowercase()));

    let registry = tracing_subscriber::registry().with(filter);

    if config.logging.log_dir.is_empty() {
        // 只输出到控制台
        let fmt_layer = fmt::layer()
            .with_ansi(config.logging.enable_color)
            .with_target(false)
            .with_thread_ids(false)
            .with_thread_names(false)
            .compact();

        registry.with(fmt_layer).init();

        if config.logging.enable_color {
            info!(
                "日志系统已初始化: 级别={}, 输出=控制台(彩色)",
                config.logging.level
            );
        } else {
            info!(
                "日志系统已初始化: 级别={}, 输出=控制台(无色)",
                config.logging.level
            );
        }
    } else {
        // 同时输出到控制台和文件
        let log_dir = Path::new(&config.logging.log_dir);

        // 确保日志目录存在
        if let Err(e) = std::fs::create_dir_all(log_dir) {
            eprintln!("创建日志目录失败: {}", e);
            std::process::exit(1);
        }

        // 创建控制台输出层
        let console_layer = fmt::layer()
            .with_ansi(config.logging.enable_color)
            .with_target(false)
            .with_thread_ids(false)
            .with_thread_names(false)
            .compact();

        // 创建文件输出层，使用复合轮转策略
        let hybrid_writer =
            HybridRotatingWriter::new(log_dir, config.logging.max_log_size.as_bytes())
                .map_err(|e| {
                    eprintln!("创建日志写入器失败: {}", e);
                    std::process::exit(1);
                })
                .unwrap();

        let file_appender = Box::new(hybrid_writer) as Box<dyn Write + Send>;
        let (non_blocking_appender, _guard) = non_blocking(file_appender);

        let file_layer = fmt::layer()
            .with_writer(non_blocking_appender)
            .with_ansi(false) // 文件输出不使用颜色
            .with_target(true)
            .json(); // 文件使用JSON格式便于日志分析

        registry.with(console_layer).with(file_layer).init();

        // 防止 _guard 被丢弃，需要存储它
        std::mem::forget(_guard);

        info!(
            "日志系统已初始化: 级别={}, 输出=控制台+文件目录({})",
            config.logging.level, config.logging.log_dir
        );

        if config.logging.max_log_size.as_bytes() > 0 {
            info!("日志轮转: 复合策略 (按天+按大小{}), 文件=rifs.YYYY-MM-DD.log, rifs.YYYY-MM-DD.N.log", config.logging.max_log_size);
        } else {
            info!("日志轮转: 仅按天轮转, 文件=rifs.YYYY-MM-DD.log");
        }

        info!("日志格式: 控制台=紧凑格式, 文件=JSON格式");
    }
}
