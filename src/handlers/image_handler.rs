use axum::{
    extract::{Multipart, Path, Query, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use base64::{engine::general_purpose, Engine as _};
use tracing::{error, info, warn};

use crate::app_state::AppState;
use crate::config::AppConfig;
use crate::models::{Base64ImageResponse, ImageQuery, ImageTransformParams, UploadResponse};
use crate::services::{CacheService, ImageService, ImageTransformService};
use crate::utils::AppError;

/// 图片上传接口
pub async fn upload_image(
    State(app_state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
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
            let image_info = ImageService::save_image(app_state.db_pool(), &data).await?;

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

/// 获取图片接口（通过哈希值，支持格式转换）
pub async fn get_image(
    State(app_state): State<AppState>,
    Path(identifier): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    // 解析标识符，检查是否包含转换参数
    let (hash, transform_params) = if let Some(at_pos) = identifier.find('@') {
        let hash = &identifier[..at_pos];
        let params_str = &identifier[at_pos + 1..];

        info!("解析转换参数: {}", params_str);

        let params = ImageTransformParams::parse(params_str)
            .map_err(|e| AppError::BadRequest(format!("转换参数解析失败: {}", e)))?;

        // 检查是否真的需要转换
        if !params.needs_transform() {
            info!("转换参数为空，返回原图: {}", hash);
            (hash, None)
        } else {
            // 验证参数合理性
            ImageTransformService::validate_params(&params)?;
            (hash, Some(params))
        }
    } else {
        (identifier.as_str(), None)
    };

    // 读取原始图片文件
    let image_data = ImageService::read_image_file(app_state.db_pool(), hash).await?;

    // 获取图片信息
    let image_info = ImageService::get_image_info(app_state.db_pool(), hash)
        .await?
        .ok_or(AppError::FileNotFound)?;

    let config = AppConfig::get();

    // 根据是否需要转换决定处理方式
    let (final_data, final_mime) = if let Some(ref params) = transform_params {
        let config = AppConfig::get();

        // 检查是否启用缓存
        if config.cache.enable_transform_cache {
            let cache_key = CacheService::generate_cache_key(hash, params);

            // 尝试从缓存获取
            let connection = app_state.db_pool().get_connection();
            let cache_service = CacheService::new(connection.clone())?;
            cache_service.ensure_cache_dir().await?;

            if let Ok(Some(cached)) = cache_service.get_cache(&cache_key).await {
                info!("缓存命中: {}", cache_key);
                let cached_data = cache_service.read_cache(&cached).await?;
                (cached_data, cached.mime_type)
            } else {
                // 缓存未命中，进行转换
                info!(
                    "缓存未命中，开始图片转换: {} -> {:?}",
                    image_info.mime_type, params
                );
                let (transformed_data, transformed_mime) = ImageTransformService::transform_image(
                    &image_data,
                    &image_info.mime_type,
                    params,
                )
                .await?;

                // 保存到缓存
                if let Err(e) = cache_service
                    .save_cache(hash, params, &transformed_data, &transformed_mime)
                    .await
                {
                    warn!("保存缓存失败: {}", e);
                }

                (transformed_data, transformed_mime)
            }
        } else {
            // 缓存未启用，直接转换
            info!(
                "开始图片转换（缓存未启用）: {} -> {:?}",
                image_info.mime_type, params
            );
            ImageTransformService::transform_image(&image_data, &image_info.mime_type, params)
                .await?
        }
    } else {
        // 不需要转换，返回原始数据
        (image_data, image_info.mime_type.clone())
    };

    // 生成文件名（如果进行了转换，使用新的扩展名）
    let filename = if let Some(ref params) = transform_params {
        if let Some(format) = &params.format {
            let ext = match format.as_str() {
                "jpeg" | "jpg" => "jpg",
                "png" => "png",
                "gif" => "gif",
                "webp" => "webp",
                "avif" => "avif",
                "ico" => "ico",
                _ => "jpg",
            };
            format!("{}.{}", hash, ext)
        } else {
            image_info.stored_name()
        }
    } else {
        image_info.stored_name()
    };

    let content_disposition = format!(r#"inline; filename="{}""#, filename);
    let cache_control = config.cache_control_header();

    // 构建扩展的响应头，包含图片信息
    use axum::http::HeaderMap;
    let mut headers = HeaderMap::new();

    // 基础响应头
    headers.insert(header::CONTENT_TYPE, final_mime.parse().unwrap());
    headers.insert(
        header::CONTENT_DISPOSITION,
        content_disposition.parse().unwrap(),
    );
    headers.insert(header::CACHE_CONTROL, cache_control.parse().unwrap());

    // 原始图片信息
    headers.insert("x-original-hash", image_info.hash.parse().unwrap());
    headers.insert(
        "x-original-size",
        image_info.size.to_string().parse().unwrap(),
    );
    headers.insert("x-original-mime", image_info.mime_type.parse().unwrap());
    headers.insert(
        "x-original-extension",
        image_info.extension.parse().unwrap(),
    );
    headers.insert(
        "x-upload-time",
        image_info
            .created_at
            .format("%Y-%m-%d %H:%M:%S UTC")
            .to_string()
            .parse()
            .unwrap(),
    );
    headers.insert(
        "x-access-count",
        image_info.access_count.to_string().parse().unwrap(),
    );

    // 处理后的信息
    headers.insert("x-final-mime", final_mime.parse().unwrap());
    headers.insert(
        "x-final-size",
        final_data.len().to_string().parse().unwrap(),
    );

            // 如果有转换参数，添加转换信息
        if let Some(ref params) = transform_params {
            headers.insert("x-transform-applied", "true".parse().unwrap());
            
            match params.base64_mode {
                crate::models::Base64OutputMode::Structured => {
                    headers.insert("x-output-format", "base64-json".parse().unwrap());
                }
                crate::models::Base64OutputMode::Raw => {
                    headers.insert("x-output-format", "base64-raw".parse().unwrap());
                }
                crate::models::Base64OutputMode::None => {}
            }

        if let Some(width) = params.width {
            headers.insert("x-transform-width", width.to_string().parse().unwrap());
        }
        if let Some(height) = params.height {
            headers.insert("x-transform-height", height.to_string().parse().unwrap());
        }
        if let Some(ref format) = params.format {
            headers.insert("x-transform-format", format.parse().unwrap());
        }
        if let Some(quality) = params.quality {
            headers.insert("x-transform-quality", quality.to_string().parse().unwrap());
        }

        if params.no_alpha {
            headers.insert("x-transform-noalpha", "true".parse().unwrap());
            if let Some(ref bg_color) = params.background_color {
                let bg_str = match bg_color {
                    crate::models::BackgroundColor::White => "white".to_string(),
                    crate::models::BackgroundColor::Black => "black".to_string(),
                    crate::models::BackgroundColor::Custom(r, g, b) => {
                        format!("#{:02x}{:02x}{:02x}", r, g, b)
                    }
                };
                headers.insert("x-transform-background", bg_str.parse().unwrap());
            }
        }

        // 生成转换参数的完整字符串
        let params_summary = identifier.split('@').nth(1).unwrap_or("").to_string();
        headers.insert("x-transform-params", params_summary.parse().unwrap());

        // 检查是否从GIF提取了第一帧
        if image_info.mime_type == "image/gif" && params.format.is_some() {
            headers.insert("x-gif-first-frame", "true".parse().unwrap());
        }
    } else {
        headers.insert("x-transform-applied", "false".parse().unwrap());
    }

    // 添加服务信息
    headers.insert("x-served-by", "RIFS".parse().unwrap());
    headers.insert(
        "x-response-time",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
            .to_string()
            .parse()
            .unwrap(),
    );

        // 检查是否需要返回base64格式
    if let Some(ref params) = transform_params {
        match params.base64_mode {
            crate::models::Base64OutputMode::Structured => {
                // 使用base64编码并返回JSON结构体
                let base64_data = general_purpose::STANDARD.encode(&final_data);
                
                let response = Base64ImageResponse {
                    success: true,
                    message: "图片获取成功".to_string(),
                    data: base64_data,
                    mime_type: final_mime.clone(),
                    size: final_data.len(),
                    original: image_info,
                };

                return Ok(Json(response).into_response());
            }
            crate::models::Base64OutputMode::Raw => {
                // 只返回纯base64字符串
                let base64_data = general_purpose::STANDARD.encode(&final_data);
                
                return Ok((
                    [(
                        header::CONTENT_TYPE,
                        axum::http::HeaderValue::from_static("text/plain; charset=utf-8"),
                    )],
                    base64_data,
                )
                    .into_response());
            }
            crate::models::Base64OutputMode::None => {
                // 不需要base64处理，继续正常流程
            }
        }
    }

    Ok((headers, final_data).into_response())
}

/// 获取图片信息接口（通过哈希值）
pub async fn get_image_info(
    State(app_state): State<AppState>,
    Path(identifier): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let image_info = ImageService::get_image_info(app_state.db_pool(), &identifier)
        .await?
        .ok_or(AppError::FileNotFound)?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "获取图片信息成功",
        "data": image_info
    })))
}

/// 查询图片列表 (POST - JSON请求体)
pub async fn query_images_post(
    State(app_state): State<AppState>,
    Json(query): Json<ImageQuery>,
) -> Result<impl IntoResponse, AppError> {
    info!("收到查询图片列表请求 (POST): {:?}", query);

    let (images, total) = ImageService::query_images(app_state.db_pool(), &query).await?;

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
pub async fn query_images_get(
    State(app_state): State<AppState>,
    Query(query): Query<ImageQuery>,
) -> Result<impl IntoResponse, AppError> {
    info!("收到查询图片列表请求 (GET): {:?}", query);

    let (images, total) = ImageService::query_images(app_state.db_pool(), &query).await?;

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
pub async fn get_stats(State(app_state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    info!("收到获取统计信息请求");

    let stats = ImageService::get_stats(app_state.db_pool()).await?;

    info!("返回统计信息");

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "获取统计信息成功",
        "data": stats
    })))
}

/// 删除图片接口（通过哈希值）
pub async fn delete_image(
    State(app_state): State<AppState>,
    Path(identifier): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    info!("收到删除图片请求: {}", identifier);

    ImageService::delete_image(app_state.db_pool(), &identifier).await?;

    let config = AppConfig::get();
    let mut cache_count = 0;

    // 如果启用了缓存，同时删除相关缓存
    if config.cache.enable_transform_cache {
        let connection = app_state.db_pool().get_connection();
        let cache_service = CacheService::new(connection)?;

        match cache_service.remove_by_original_hash(&identifier).await {
            Ok(count) => {
                cache_count = count;
                if count > 0 {
                    info!("删除图片{}的{}个相关缓存", identifier, count);
                }
            }
            Err(e) => {
                warn!("删除图片缓存失败: {}", e);
            }
        }
    }

    info!("图片删除成功: {}", identifier);

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "图片删除成功",
        "cache_cleaned": cache_count
    })))
}
