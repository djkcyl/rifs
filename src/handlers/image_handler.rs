use axum::{
    extract::{Multipart, Path},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use tracing::{error, info};

use crate::models::UploadResponse;
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
            // 获取内容类型
            let content_type = field.content_type().map(|ct| ct.to_string());
            
            // 读取文件数据
            let data = field.bytes().await.map_err(|e| {
                error!("读取文件数据失败: {}", e);
                AppError::BadRequest("读取文件数据失败".to_string())
            })?;

            if data.is_empty() {
                error!("上传的文件为空");
                return Err(AppError::BadRequest("上传的文件为空".to_string()));
            }

            info!("开始保存图片: {}字节", data.len());

            // 保存图片
            let image_info = ImageService::save_image(
                &data,
                content_type.as_deref(),
            ).await?;

            info!("图片保存成功: {}", image_info.stored_name);

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

/// 获取图片接口
pub async fn get_image(Path(filename): Path<String>) -> Result<impl IntoResponse, AppError> {
    info!("收到获取图片请求: {}", filename);

    // 读取图片文件
    let image_data = ImageService::read_image_file(&filename).await?;
    
    // 获取图片信息
    let image_info = ImageService::get_image_info(&filename).await?
        .ok_or(AppError::FileNotFound)?;

    info!("返回图片: {} ({}字节)", filename, image_data.len());

    let config = AppConfig::get();
    
    // 将所有字符串转为 String 类型，避免生命周期问题
    let content_type = image_info.mime_type.clone();
    let content_disposition = format!(r#"inline; filename="{}""#, filename);
    let cache_control = config.cache_control_header();

    // 设置响应头
    let headers = [
        (header::CONTENT_TYPE, content_type),
        (header::CONTENT_DISPOSITION, content_disposition),
        (header::CACHE_CONTROL, cache_control),
    ];

    Ok((headers, image_data))
}

/// 获取图片信息接口
pub async fn get_image_info(Path(filename): Path<String>) -> Result<impl IntoResponse, AppError> {
    info!("收到获取图片信息请求: {}", filename);

    let image_info = ImageService::get_image_info(&filename).await?
        .ok_or(AppError::FileNotFound)?;

    info!("返回图片信息: {}", filename);

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "获取图片信息成功",
        "data": image_info
    })))
}

/// 删除图片接口
pub async fn delete_image(Path(filename): Path<String>) -> Result<impl IntoResponse, AppError> {
    info!("收到删除图片请求: {}", filename);

    ImageService::delete_image(&filename).await?;

    info!("图片删除成功: {}", filename);

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "图片删除成功"
    })))
} 