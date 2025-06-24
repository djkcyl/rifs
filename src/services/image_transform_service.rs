use tracing::info;

use crate::models::ImageTransformParams;
use crate::utils::AppError;
use super::{
    image_format_utils::ImageFormatUtils,
    static_image_transform::StaticImageTransform,
};

/// 图片转换服务 - 支持所有image库编解码器
pub struct ImageTransformService;

impl ImageTransformService {
    /// 高级图片转换 - 使用专用编码器和优化参数
    pub async fn transform_image(
        image_data: &[u8],
        original_mime: &str,
        params: &ImageTransformParams,
    ) -> Result<(Vec<u8>, String), AppError> {
        // 如果不需要转换，直接返回原始数据
        if !params.needs_transform() {
            return Ok((image_data.to_vec(), original_mime.to_string()));
        }

        info!("开始高级图片转换: {:?}", params);

        // 检测是否为动图格式（需要实际解析数据）
        let is_animated_format = ImageFormatUtils::is_animated_format(original_mime, image_data);
        
        // 如果是动图格式且用户没有要求格式转换，直接返回原图
        if is_animated_format && params.format.is_none() {
            info!("检测到多帧动图且无格式转换要求，直接返回原图: {}", original_mime);
            return Ok((image_data.to_vec(), original_mime.to_string()));
        }
        
        // 如果是动图但用户明确要求转换格式，提取第一帧进行静图转换
        if is_animated_format && params.format.is_some() {
            info!("检测到多帧动图但用户要求格式转换，将提取第一帧: {} -> {:?}", 
                original_mime, params.format);
        }
        
        // 加载静图（如果是动图但要求转换，提取第一帧）
        let mut img = if original_mime == "image/gif" {
            // GIF格式使用专门的第一帧提取函数，无论单帧还是多帧
            info!("使用GIF第一帧提取器");
            StaticImageTransform::load_gif_first_frame(image_data)?
        } else if original_mime == "image/webp" && is_animated_format {
            // 多帧WebP要求转换时，也提取第一帧（暂时使用通用加载器）
            info!("多帧WebP转换，使用通用加载器提取第一帧");
            StaticImageTransform::load_image_with_color_info(image_data)?
        } else {
            // 其他格式使用通用加载器
            StaticImageTransform::load_image_with_color_info(image_data)?
        };
        
        // 调整尺寸（使用高质量重采样）
        if params.width.is_some() || params.height.is_some() {
            img = StaticImageTransform::resize_image_hq(img, params.width, params.height)?;
        }

        // 确定目标格式
        let target_format = ImageFormatUtils::determine_target_format(original_mime, &params.format)?;
        let target_mime = params.target_mime_type()
            .unwrap_or_else(|| original_mime.to_string());

        // 处理透明通道
        if params.no_alpha || ImageFormatUtils::format_requires_no_alpha(&target_format) {
            img = StaticImageTransform::remove_alpha_channel_advanced(img, &params.background_color)?;
        }
        
        // 验证格式编码能力和参数兼容性
        ImageFormatUtils::validate_format_compatibility(&target_format, params, params.format.is_some())?;

        // 使用专用编码器编码静图
        let encoded_data = StaticImageTransform::encode_with_specialized_encoder(img, target_format, params).await?;

        info!("高级图片转换完成: {} -> {}, 原始大小: {}字节, 转换后: {}字节", 
            original_mime, target_mime, image_data.len(), encoded_data.len());

        Ok((encoded_data, target_mime))
    }

    /// 验证转换参数
    pub fn validate_params(params: &ImageTransformParams) -> Result<(), AppError> {
        ImageFormatUtils::validate_params(params)
    }
} 