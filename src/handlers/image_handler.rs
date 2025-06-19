use axum::{
    extract::{Multipart, Path, Query},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use tracing::{error, info};

use crate::models::{UploadResponse, ImageQuery};
use crate::services::ImageService;
use crate::utils::AppError;
use crate::config::AppConfig;



/// 健康检查接口
pub async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "message": "图床服务运行正常",
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }))
}

/// 上传图片接口
pub async fn upload_image(mut multipart: Multipart) -> Result<impl IntoResponse, AppError> {
    info!("收到图片上传请求");

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        error!("解析multipart数据失败: {}", e);
        AppError::BadRequest("无效的multipart数据".to_string())
    })? {
        let name = field.name().unwrap_or("").to_string();
        
        // 只处理名为 "file" 的字段
        if name == "file" {
            // 读取文件数据
            let data = field.bytes().await.map_err(|e| {
                error!("读取文件数据失败: {}", e);
                AppError::BadRequest("读取文件数据失败".to_string())
            })?;

            if data.is_empty() {
                error!("上传的文件为空");
                return Err(AppError::InvalidFile);
            }

            info!("开始保存图片: {}字节", data.len());

            // 保存图片（后端会自动检测真实文件类型）
            let image_info = ImageService::save_image(&data).await?;

            info!("图片保存成功: {}", image_info.stored_name());

            let response = UploadResponse {
                success: true,
                message: "图片上传成功".to_string(),
                data: Some(image_info),
            };

            return Ok((StatusCode::OK, Json(response)));
        }
    }

    error!("未找到有效的文件字段");
    Err(AppError::BadRequest("请选择要上传的图片文件".to_string()))
}

/// 获取图片接口（通过哈希值）
pub async fn get_image(Path(identifier): Path<String>) -> Result<impl IntoResponse, AppError> {
    // 读取图片文件
    let image_data = ImageService::read_image_file(&identifier).await?;
    
    // 获取图片信息
    let image_info = ImageService::get_image_info(&identifier).await?
        .ok_or(AppError::FileNotFound)?;

    let config = AppConfig::get();
    
    // 将所有字符串转为 String 类型，避免生命周期问题
    let content_type = image_info.mime_type.clone();
    let content_disposition = format!(r#"inline; filename="{}""#, image_info.stored_name());
    let cache_control = config.cache_control_header();

    // 设置响应头
    let headers = [
        (header::CONTENT_TYPE, content_type),
        (header::CONTENT_DISPOSITION, content_disposition),
        (header::CACHE_CONTROL, cache_control),
    ];

    Ok((headers, image_data))
}

/// 获取图片信息接口（通过哈希值）
pub async fn get_image_info(Path(identifier): Path<String>) -> Result<impl IntoResponse, AppError> {
    let image_info = ImageService::get_image_info(&identifier).await?
        .ok_or(AppError::FileNotFound)?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "获取图片信息成功",
        "data": image_info
    })))
}

/// 查询图片列表 (POST - JSON请求体)
pub async fn query_images_post(Json(query): Json<ImageQuery>) -> Result<impl IntoResponse, AppError> {
    info!("收到查询图片列表请求 (POST): {:?}", query);

    let (images, total) = ImageService::query_images(&query).await?;

    info!("返回图片列表: {} 条记录，总计 {} 条", images.len(), total);

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "查询图片列表成功",
        "data": {
            "items": images,
            "total": total,
            "limit": query.limit.unwrap_or(20),
            "offset": query.offset.unwrap_or(0)
        }
    })))
}

/// 查询图片列表 (GET - URL查询参数)
pub async fn query_images_get(Query(query): Query<ImageQuery>) -> Result<impl IntoResponse, AppError> {
    info!("收到查询图片列表请求 (GET): {:?}", query);

    let (images, total) = ImageService::query_images(&query).await?;

    info!("返回图片列表: {} 条记录，总计 {} 条", images.len(), total);

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "查询图片列表成功",
        "data": {
            "items": images,
            "total": total,
            "limit": query.limit.unwrap_or(20),
            "offset": query.offset.unwrap_or(0)
        }
    })))
}

/// 获取统计信息
pub async fn get_stats() -> Result<impl IntoResponse, AppError> {
    info!("收到获取统计信息请求");

    let stats = ImageService::get_stats().await?;

    info!("返回统计信息");

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "获取统计信息成功",
        "data": stats
    })))
}

/// 删除图片接口（通过哈希值）
pub async fn delete_image(Path(identifier): Path<String>) -> Result<impl IntoResponse, AppError> {
    info!("收到删除图片请求: {}", identifier);

    ImageService::delete_image(&identifier).await?;

    info!("图片删除成功: {}", identifier);

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "图片删除成功"
    })))
} 