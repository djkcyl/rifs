use std::path::{Path, PathBuf};

use crate::config::AppConfig;
use crate::utils::error::AppError;

/// 支持的图片格式 - 现代主流格式支持
/// 专注于web友好和广泛支持的格式
pub const SUPPORTED_IMAGE_TYPES: &[&str] = &[
    // 传统格式
    "image/jpeg", "image/png", "image/gif",
    // 现代格式  
    "image/webp", "image/avif",
    // 图标格式
    "image/x-icon"
];

/// 基于文件内容检测真实的MIME
pub fn detect_file_type(data: &[u8]) -> Result<String, AppError> {
    // 检查文件是否为空
    if data.is_empty() {
        return Err(AppError::InvalidFile);
    }

    // 首先使用infer库检测文件类型
    if let Some(kind) = infer::get(data) {
        let mime_type = kind.mime_type();
        if SUPPORTED_IMAGE_TYPES.contains(&mime_type) {
            return Ok(mime_type.to_string());
        }
    }

    // 如果infer检测失败，尝试使用image库进行格式推断
    match image::guess_format(data) {
        Ok(format) => {
            let mime_type = match format {
                image::ImageFormat::Jpeg => "image/jpeg",
                image::ImageFormat::Png => "image/png",
                image::ImageFormat::Gif => "image/gif",
                image::ImageFormat::WebP => "image/webp",
                image::ImageFormat::Ico => "image/x-icon",
                image::ImageFormat::Avif => "image/avif",
                _ => return Err(AppError::UnsupportedFileType),
            };
            
            if SUPPORTED_IMAGE_TYPES.contains(&mime_type) {
                Ok(mime_type.to_string())
            } else {
                Err(AppError::UnsupportedFileType)
            }
        }
        Err(_) => {
                Err(AppError::UnsupportedFileType)
            }
    }
}



/// 根据MIME类型获取对应的文件扩展名
pub fn get_extension_from_mime(mime_type: &str) -> Result<String, AppError> {
    match mime_type {
        // JPEG 格式
        "image/jpeg" => Ok("jpg".to_string()),
        "image/jpg" => Ok("jpg".to_string()),
        
        // PNG 格式
        "image/png" => Ok("png".to_string()),
        
        // GIF 格式
        "image/gif" => Ok("gif".to_string()),
        
        // WebP 格式
        "image/webp" => Ok("webp".to_string()),
        
        // AVIF 格式 (现代高效格式)
        "image/avif" => Ok("avif".to_string()),
        
        // ICO 格式 (图标)
        "image/x-icon" => Ok("ico".to_string()),
        "image/vnd.microsoft.icon" => Ok("ico".to_string()),
        "image/ico" => Ok("ico".to_string()),
        
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
