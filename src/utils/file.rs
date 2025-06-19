use std::path::{Path, PathBuf};
use uuid::Uuid;
use mime_guess;

use crate::utils::error::AppError;
use crate::config::AppConfig;

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

/// 验证文件是否为支持的图片类型
pub fn validate_image_type(content_type: &str) -> Result<(), AppError> {
    if SUPPORTED_IMAGE_TYPES.contains(&content_type) {
        Ok(())
    } else {
        Err(AppError::UnsupportedFileType)
    }
}

/// 验证文件大小
pub fn validate_file_size(size: u64) -> Result<(), AppError> {
    let config = AppConfig::get();
    if size > config.storage.max_file_size {
        Err(AppError::FileTooLarge { max_size: config.storage.max_file_size })
    } else {
        Ok(())
    }
}

/// 根据文件名猜测MIME类型
pub fn guess_mime_type(filename: &str) -> String {
    mime_guess::from_path(filename)
        .first_or_octet_stream()
        .to_string()
}

/// 获取文件扩展名
pub fn get_file_extension(filename: &str) -> String {
    Path::new(filename)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase()
}

/// 生成唯一的文件名
pub fn generate_unique_filename(extension: &str) -> (String, String, PathBuf) {
    let id = Uuid::new_v4();
    let id_str = id.to_string();
    
    // 获取UUID前两位和第3-4位用于目录结构
    let first_dir = &id_str[0..2];
    let second_dir = &id_str[2..4];
    
    let stored_name = if extension.is_empty() {
        id_str.clone()
    } else {
        format!("{}.{}", id_str, extension)
    };
    
    // 生成相对路径
    let relative_path = Path::new(first_dir).join(second_dir).join(&stored_name);
    
    (id_str, stored_name, relative_path)
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

/// 根据存储名称解析文件路径
pub fn parse_file_path(stored_name: &str) -> PathBuf {
    if stored_name.len() < 4 {
        // 处理旧的格式或无效的名称，直接返回根目录下的文件
        return get_upload_dir().join(stored_name);
    }
    
    let first_dir = &stored_name[0..2];
    let second_dir = &stored_name[2..4];
    
    Path::new(first_dir).join(second_dir).join(stored_name)
}

/// 检查文件是否存在
pub async fn file_exists(stored_name: &str) -> bool {
    let relative_path = parse_file_path(stored_name);
    let file_path = get_upload_dir().join(relative_path);
    file_path.exists()
} 