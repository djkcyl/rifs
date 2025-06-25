use axum::{extract::ConnectInfo, middleware::Next, response::Response};
use std::net::SocketAddr;
use tracing::info;

/// 简单的HTTP请求日志中间件
pub async fn log_requests(
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
        (latency.as_nanos(), "ns")
    };

    // 根据状态码选择背景色和前景色（加粗）
    let status_code = response.status().as_u16();
    let status_display = match status_code {
        200..=299 => format!("\x1b[1;30;42m {:>3} \x1b[0m", status_code), // 加粗黑字绿底
        300..=399 => format!("\x1b[1;30;43m {:>3} \x1b[0m", status_code), // 加粗黑字黄底
        400..=499 => format!("\x1b[1;37;41m {:>3} \x1b[0m", status_code), // 加粗白字红底
        500..=599 => format!("\x1b[1;37;45m {:>3} \x1b[0m", status_code), // 加粗白字紫底
        _ => format!(" {:>3} ", status_code),                             // 无色
    };

    info!(
        "{} {:>4} | {:<15} | {:>4}{} | {}",
        status_display, method, client_ip, time_value, time_unit, uri
    );

    response
}
