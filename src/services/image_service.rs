use chrono::Utc;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use crate::models::{ImageInfo, ImageQuery, ImageStats};
use crate::services::DatabaseService;
use crate::utils::{
    AppError, ensure_upload_dir, ensure_image_dir,
    get_file_path, validate_file_size, detect_file_type, get_extension_from_mime,
    get_upload_dir
};

/// 图片服务结构体
pub struct ImageService;

impl ImageService {
    /// 获取数据库服务实例
    async fn get_db() -> Result<Arc<DatabaseService>, AppError> {
        static DB_SERVICE: tokio::sync::OnceCell<Arc<DatabaseService>> =
            tokio::sync::OnceCell::const_new();

        match DB_SERVICE.get() {
            Some(db) => Ok(db.clone()),
            None => {
                let db = Arc::new(DatabaseService::new().await?);
                let _ = DB_SERVICE.set(db.clone());
                Ok(db)
            }
        }
    }

    /// 保存上传的图片文件
    pub async fn save_image(data: &[u8]) -> Result<ImageInfo, AppError> {
        // 验证文件是否为空
        if data.is_empty() {
            return Err(AppError::InvalidFile);
        }
        
        // 验证文件大小
        validate_file_size(data.len() as u64)?;

        // 基于文件内容检测真实的MIME类型（安全）
        let mime_type = detect_file_type(data)?;
        
        // 计算文件哈希值用于去重
        let file_hash = DatabaseService::calculate_file_hash(data);
        
        // 检查是否已存在相同文件
        let db = Self::get_db().await?;
        if let Some(existing_image) = db.find_by_hash(&file_hash).await? {
            return Ok(existing_image);
        }

        // 确保基础上传目录存在
        ensure_upload_dir().await?;

        // 根据真实MIME类型生成文件扩展名
        let extension = get_extension_from_mime(&mime_type)?;

        // 创建图片信息
        let image_info = ImageInfo {
            hash: file_hash.clone(),
            size: data.len() as u64,
            mime_type,
            created_at: Utc::now(),
            last_accessed: None,
            extension,
            access_count: 0,
        };

        // 计算文件路径
        let stored_name = image_info.stored_name();
        let relative_path = format!("{}/{}/{}", &file_hash[0..2], &file_hash[2..4], stored_name);

        // 确保存储目录结构存在
        ensure_image_dir(std::path::Path::new(&relative_path)).await?;

        // 获取文件完整路径
        let file_path = get_file_path(std::path::Path::new(&relative_path));

        // 写入文件
        let mut file = File::create(&file_path).await?;
        file.write_all(data).await?;
        file.sync_all().await?;

        // 保存到数据库
        db.insert_image(&image_info).await?;

        Ok(image_info)
    }

    /// 根据哈希值获取图片信息
    pub async fn get_image_info(identifier: &str) -> Result<Option<ImageInfo>, AppError> {
        // 验证标识符格式
        if identifier.is_empty() {
            return Err(AppError::BadRequest("标识符不能为空".to_string()));
        }

        let db = Self::get_db().await?;

        // 从数据库查询
        db.get_image(identifier).await
    }

    /// 读取图片文件内容
    pub async fn read_image_file(identifier: &str) -> Result<Vec<u8>, AppError> {
        // 获取图片信息
        let image_info = Self::get_image_info(identifier)
            .await?
            .ok_or(AppError::FileNotFound)?;

        // 更新访问信息
        let db = Self::get_db().await?;
        let _ = db.update_access(identifier).await;

        // 构建文件路径
        let stored_name = image_info.stored_name();
        let relative_path = format!(
            "{}/{}/{}",
            &image_info.hash[0..2],
            &image_info.hash[2..4],
            stored_name
        );
        let file_path = get_upload_dir().join(&relative_path);

        let data = tokio::fs::read(&file_path).await?;
        Ok(data)
    }

    /// 删除图片文件
    pub async fn delete_image(identifier: &str) -> Result<(), AppError> {
        // 获取图片信息
        let image_info = Self::get_image_info(identifier)
            .await?
            .ok_or(AppError::FileNotFound)?;

        // 从数据库删除记录
        let db = Self::get_db().await?;
        db.delete_image(identifier).await?;

        // 删除文件
        let stored_name = image_info.stored_name();
        let relative_path = format!(
            "{}/{}/{}",
            &image_info.hash[0..2],
            &image_info.hash[2..4],
            stored_name
        );
        let file_path = get_upload_dir().join(&relative_path);

        tokio::fs::remove_file(&file_path).await?;

        Ok(())
    }

    /// 查询图片列表
    pub async fn query_images(query: &ImageQuery) -> Result<(Vec<ImageInfo>, u64), AppError> {
        let db = Self::get_db().await?;
        db.query_images(query).await
    }

    /// 获取统计信息
    pub async fn get_stats() -> Result<ImageStats, AppError> {
        let db = Self::get_db().await?;
        db.get_stats().await
    }
}


