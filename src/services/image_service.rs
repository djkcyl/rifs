use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::models::ImageInfo;
use crate::utils::{
    AppError, ensure_upload_dir, ensure_image_dir, file_exists, generate_unique_filename, 
    get_file_path, parse_file_path, guess_mime_type, validate_file_size, validate_image_type,
    get_file_extension, get_upload_dir
};

/// 图片服务结构体
pub struct ImageService;

impl ImageService {
    /// 保存上传的图片文件
    pub async fn save_image(
        data: &[u8],
        content_type: Option<&str>,
    ) -> Result<ImageInfo, AppError> {
        // 验证文件大小
        validate_file_size(data.len() as u64)?;

        // 猜测或验证MIME类型
        let mime_type = content_type
            .unwrap_or("application/octet-stream")
            .to_string();
        
        // 验证文件类型
        validate_image_type(&mime_type)?;

        // 确保基础上传目录存在
        ensure_upload_dir().await?;

        // 生成唯一文件名和路径，根据MIME类型确定扩展名
        let extension = mime_type_to_extension(&mime_type);
        let (id_str, stored_name, relative_path) = generate_unique_filename(&extension);
        let id = Uuid::parse_str(&id_str)
            .map_err(|e| AppError::Internal(format!("UUID解析失败: {}", e)))?;

        // 确保存储目录结构存在
        ensure_image_dir(&relative_path).await?;

        // 获取文件完整路径
        let file_path = get_file_path(&relative_path);

        // 写入文件
        let mut file = File::create(&file_path).await?;
        file.write_all(data).await?;
        file.sync_all().await?;

        // 获取当前时间戳
        let upload_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // 创建图片信息
        let image_info = ImageInfo {
            id,
            stored_name,
            size: data.len() as u64,
            mime_type,
            upload_time,
            extension,
        };

        Ok(image_info)
    }

    /// 根据存储文件名获取图片信息
    pub async fn get_image_info(stored_name: &str) -> Result<Option<ImageInfo>, AppError> {
        if !file_exists(stored_name).await {
            return Ok(None);
        }

        // 获取文件路径
        let relative_path = parse_file_path(stored_name);
        let file_path = get_upload_dir().join(&relative_path);
        
        let metadata = tokio::fs::metadata(&file_path).await?;
        
        // 从文件名解析UUID（如果可能）
        let id = stored_name
            .split('.')
            .next()
            .and_then(|id_str| Uuid::parse_str(id_str).ok())
            .unwrap_or_else(Uuid::new_v4);

        let mime_type = guess_mime_type(stored_name);
        let extension = get_file_extension(stored_name);

        let image_info = ImageInfo {
            id,
            stored_name: stored_name.to_string(),
            size: metadata.len(),
            mime_type,
            upload_time: metadata
                .created()
                .or_else(|_| metadata.modified())
                .unwrap_or_else(|_| std::time::SystemTime::now())
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            extension,
        };

        Ok(Some(image_info))
    }

    /// 读取图片文件内容
    pub async fn read_image_file(stored_name: &str) -> Result<Vec<u8>, AppError> {
        if !file_exists(stored_name).await {
            return Err(AppError::FileNotFound);
        }

        let relative_path = parse_file_path(stored_name);
        let file_path = get_upload_dir().join(&relative_path);
        let data = tokio::fs::read(&file_path).await?;
        Ok(data)
    }

    /// 删除图片文件
    pub async fn delete_image(stored_name: &str) -> Result<(), AppError> {
        if !file_exists(stored_name).await {
            return Err(AppError::FileNotFound);
        }

        let relative_path = parse_file_path(stored_name);
        let file_path = get_upload_dir().join(&relative_path);
        tokio::fs::remove_file(&file_path).await?;
        Ok(())
    }
}

/// 根据MIME类型获取对应的文件扩展名
fn mime_type_to_extension(mime_type: &str) -> String {
    match mime_type {
        "image/jpeg" | "image/jpg" => "jpg".to_string(),
        "image/png" => "png".to_string(),
        "image/gif" => "gif".to_string(),
        "image/webp" => "webp".to_string(),
        "image/bmp" => "bmp".to_string(),
        "image/tiff" => "tiff".to_string(),
        _ => "bin".to_string(),
    }
} 