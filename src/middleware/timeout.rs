use axum::{
    extract::State,
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde_json;

use crate::app_state::AppState;

/// 请求超时中间件
pub async fn request_timeout(
    State(app_state): State<AppState>,
    request: axum::http::Request<axum::body::Body>,
    next: Next,
) -> Response {
    let timeout_duration =
        std::time::Duration::from_secs(app_state.config().server.request_timeout.as_seconds());

    match tokio::time::timeout(timeout_duration, next.run(request)).await {
        Ok(response) => response,
        Err(_) => {
            // 超时处理
            let error_response = serde_json::json!({
                "success": false,
                "message": "请求处理超时",
                "code": 408
            });

            (
                axum::http::StatusCode::REQUEST_TIMEOUT,
                axum::response::Json(error_response),
            )
                .into_response()
        }
    }
}
