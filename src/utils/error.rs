use axum::{http::StatusCode, response::IntoResponse, Json};
use thiserror::Error;

use crate::models::ErrorResponse;

/// 图床服务的自定义错误类型
#[derive(Error, Debug)]
pub enum AppError {
    #[error("文件IO错误: {0}")]
    FileIo(#[from] std::io::Error),
    
    #[error("不支持的文件类型")]
    UnsupportedFileType,
    
    #[error("文件过大，最大允许 {max_size} 字节")]
    FileTooLarge { max_size: u64 },
    
    #[error("文件不存在")]
    FileNotFound,
    
    #[error("无效的文件")]
    InvalidFile,
    
    #[error("服务器内部错误: {0}")]
    Internal(String),
    
    #[error("请求格式错误: {0}")]
    BadRequest(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_response) = match self {
            AppError::FileIo(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse {
                    success: false,
                    message: "文件操作失败".to_string(),
                    code: Some(500),
                },
            ),
            AppError::UnsupportedFileType => (
                StatusCode::BAD_REQUEST,
                ErrorResponse {
                    success: false,
                    message: "不支持的文件类型，请上传图片文件".to_string(),
                    code: Some(400),
                },
            ),
            AppError::FileTooLarge { max_size } => (
                StatusCode::BAD_REQUEST,
                ErrorResponse {
                    success: false,
                    message: format!("文件过大，最大允许 {} MB", max_size / 1024 / 1024),
                    code: Some(400),
                },
            ),
            AppError::FileNotFound => (
                StatusCode::NOT_FOUND,
                ErrorResponse {
                    success: false,
                    message: "文件不存在".to_string(),
                    code: Some(404),
                },
            ),
            AppError::InvalidFile => (
                StatusCode::BAD_REQUEST,
                ErrorResponse {
                    success: false,
                    message: "无效的文件".to_string(),
                    code: Some(400),
                },
            ),
            AppError::Internal(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse {
                    success: false,
                    message: format!("服务器内部错误: {}", msg),
                    code: Some(500),
                },
            ),
            AppError::BadRequest(msg) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse {
                    success: false,
                    message: msg,
                    code: Some(400),
                },
            ),
        };

        (status, Json(error_response)).into_response()
    }
} 