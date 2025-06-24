use image::ImageFormat;
use crate::models::ImageTransformParams;
use crate::utils::AppError;

/// 图片格式工具函数
pub struct ImageFormatUtils;

impl ImageFormatUtils {
    /// 检测是否为动画格式，需要结合数据实际判断
    pub fn is_animated_format(mime_type: &str, data: &[u8]) -> bool {
        match mime_type {
            "image/gif" => {
                // GIF实际检测是否包含多帧
                Self::is_gif_animated(data)
            },
            "image/webp" => {
                // WebP需要实际检测是否包含多帧
                Self::is_webp_animated(data)
            },
            _ => false,
        }
    }
    
    /// 检测GIF是否为动画（包含多帧）
    fn is_gif_animated(data: &[u8]) -> bool {
        // 使用gif库解析检测
        let mut decoder = gif::DecodeOptions::new();
        decoder.set_color_output(gif::ColorOutput::RGBA);
        
        match decoder.read_info(std::io::Cursor::new(data)) {
            Ok(mut reader) => {
                let mut frame_count = 0;
                
                // 尝试读取最多3帧来判断是否为动画
                while frame_count < 3 {
                    match reader.read_next_frame() {
                        Ok(Some(_)) => frame_count += 1,
                        Ok(None) => break,
                        Err(_) => break,
                    }
                }
                
                tracing::info!("GIF解析结果: 检测到{}帧", frame_count);
                
                // 只有多于1帧的才算动画
                frame_count > 1
            },
            Err(_) => {
                // 解析失败，当作静态图片处理
                tracing::info!("GIF解析失败，当作静态图片处理");
                false
            }
        }
    }
    
    /// 检测WebP是否为动画（包含多帧）
    fn is_webp_animated(data: &[u8]) -> bool {
        // 使用webp库解析检测
        match webp::AnimDecoder::new(data).decode() {
            Ok(anim_image) => {
                let frame_count = anim_image.len();
                let is_animated = anim_image.has_animation();
                
                tracing::info!("WebP解析结果: 帧数={}, 动画标志={}", frame_count, is_animated);
                
                // 只有真正包含多帧的才算动画
                frame_count > 1 && is_animated
            },
            Err(_) => {
                // 解析失败，当作静态图片处理
                tracing::info!("WebP动画解析失败，当作静态图片处理");
                false
            }
        }
    }

    /// 检测目标格式是否不支持透明通道
    pub fn format_requires_no_alpha(format: &ImageFormat) -> bool {
        matches!(format, ImageFormat::Jpeg)
    }

    /// 确定目标格式
    pub fn determine_target_format(
        original_mime: &str, 
        target_format: &Option<String>
    ) -> Result<ImageFormat, AppError> {
        let format_str = if let Some(fmt) = target_format {
            fmt.as_str()
        } else {
            // 根据原始MIME类型推断
            match original_mime {
                "image/jpeg" => "jpeg",
                "image/png" => "png",
                "image/gif" => "gif",
                "image/webp" => "webp",
                "image/avif" => "avif",
                "image/x-icon" => "ico",
                _ => return Err(AppError::BadRequest("不支持的图片格式".to_string())),
            }
        };

        match format_str.to_lowercase().as_str() {
            "jpeg" | "jpg" => Ok(ImageFormat::Jpeg),
            "png" => Ok(ImageFormat::Png),
            "gif" => Ok(ImageFormat::Gif),
            "webp" => Ok(ImageFormat::WebP),
            "avif" => Ok(ImageFormat::Avif),
            "ico" => Ok(ImageFormat::Ico),
            _ => Err(AppError::BadRequest(format!("不支持的目标格式: {}", format_str))),
        }
    }

    /// 验证格式兼容性
    pub fn validate_format_compatibility(
        format: &ImageFormat, 
        params: &ImageTransformParams,
        _is_target_format_specified: bool
    ) -> Result<(), AppError> {
        // AVIF格式特殊检查
        if matches!(format, ImageFormat::Avif) {
            if params.quality.is_some() {
                return Err(AppError::BadRequest(
                    "AVIF格式暂不支持质量参数".to_string()
                ));
            }
        }

        // GIF格式检查
        if matches!(format, ImageFormat::Gif) {
            if params.quality.is_some() {
                return Err(AppError::BadRequest(
                    "GIF格式不支持质量参数".to_string()
                ));
            }
        }

        Ok(())
    }

    /// 验证转换参数
    pub fn validate_params(params: &ImageTransformParams) -> Result<(), AppError> {
        // 检查尺寸限制（提升到8K支持）
        if let Some(width) = params.width {
            if width == 0 || width > 8192 {
                return Err(AppError::BadRequest(
                    "宽度必须在1-8192像素之间".to_string()
                ));
            }
        }

        if let Some(height) = params.height {
            if height == 0 || height > 8192 {
                return Err(AppError::BadRequest(
                    "高度必须在1-8192像素之间".to_string()
                ));
            }
        }

        // 检查质量参数
        if let Some(quality) = params.quality {
            if quality == 0 || quality > 100 {
                return Err(AppError::BadRequest(
                    "质量参数必须在1-100之间".to_string()
                ));
            }
        }

        Ok(())
    }
} 