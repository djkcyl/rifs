use std::path::{Path, PathBuf};

use crate::config::AppConfig;
use crate::utils::error::AppError;

/// 支持的图片文件类型
const SUPPORTED_IMAGE_TYPES: [&str; 7] = [
    "image/jpeg",
    "image/jpg",
    "image/png",
    "image/gif",
    "image/webp",
    "image/bmp",
    "image/tiff",
];

/// 基于文件内容检测真实的MIME类型
pub fn detect_file_type(data: &[u8]) -> Result<String, AppError> {
    // 检查文件是否为空
    if data.is_empty() {
        return Err(AppError::InvalidFile);
    }

    // 使用infer库检测文件类型
    match infer::get(data) {
        Some(kind) => {
            let mime_type = kind.mime_type();
            // 验证是否为支持的图片类型
            if SUPPORTED_IMAGE_TYPES.contains(&mime_type) {
                Ok(mime_type.to_string())
            } else {
                Err(AppError::UnsupportedFileType)
            }
        }
        None => {
            // 无法检测到文件类型
            Err(AppError::UnsupportedFileType)
        }
    }
}

/// 根据MIME类型获取对应的文件扩展名
pub fn get_extension_from_mime(mime_type: &str) -> Result<String, AppError> {
    match mime_type {
        "image/jpeg" => Ok("jpg".to_string()),
        "image/jpg" => Ok("jpg".to_string()),
        "image/png" => Ok("png".to_string()),
        "image/gif" => Ok("gif".to_string()),
        "image/webp" => Ok("webp".to_string()),
        "image/bmp" => Ok("bmp".to_string()),
        "image/tiff" => Ok("tiff".to_string()),
        _ => Err(AppError::UnsupportedFileType),
    }
}

/// 验证文件大小
pub fn validate_file_size(size: u64) -> Result<(), AppError> {
    let config = AppConfig::get();
    if size > config.storage.max_file_size {
        Err(AppError::FileTooLarge {
            max_size: config.storage.max_file_size,
        })
    } else {
        Ok(())
    }
}

/// 获取基础上传目录路径
pub fn get_upload_dir() -> PathBuf {
    let config = AppConfig::get();
    config.upload_dir_path()
}

/// 确保上传目录存在
pub async fn ensure_upload_dir() -> Result<(), AppError> {
    let upload_dir = get_upload_dir();
    if !upload_dir.exists() {
        tokio::fs::create_dir_all(&upload_dir).await?;
    }
    Ok(())
}

/// 确保特定图片的目录结构存在
pub async fn ensure_image_dir(relative_path: &Path) -> Result<(), AppError> {
    let parent_dir = get_upload_dir().join(relative_path.parent().unwrap_or(Path::new("")));
    if !parent_dir.exists() {
        tokio::fs::create_dir_all(&parent_dir).await?;
    }
    Ok(())
}

/// 获取文件的完整路径
pub fn get_file_path(relative_path: &Path) -> PathBuf {
    get_upload_dir().join(relative_path)
}
