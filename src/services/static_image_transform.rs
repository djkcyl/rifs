use image::{
    DynamicImage, ImageFormat, GenericImageView,
    codecs::{
        jpeg::JpegEncoder,
        png::{PngEncoder, CompressionType as PngCompression, FilterType as PngFilter},

    },
    imageops::FilterType,
};
use std::io::Cursor;
use tracing::{info, warn, error};

use crate::models::{ImageTransformParams, BackgroundColor};
use crate::utils::AppError;

/// 静图转换服务
pub struct StaticImageTransform;

impl StaticImageTransform {
    /// 加载图片并获取颜色信息
    pub fn load_image_with_color_info(data: &[u8]) -> Result<DynamicImage, AppError> {
        let img = image::load_from_memory(data)
            .map_err(|e| {
                error!("图片加载失败: {}", e);
                AppError::InvalidFile
            })?;

        let (width, height) = img.dimensions();
        let color_type = img.color();
        info!("图片加载成功: {}x{}, 颜色类型: {:?}", width, height, color_type);

        Ok(img)
    }
    
    /// 从GIF提取第一帧作为静态图片
    pub fn load_gif_first_frame(data: &[u8]) -> Result<DynamicImage, AppError> {
        let mut decoder = gif::DecodeOptions::new();
        decoder.set_color_output(gif::ColorOutput::RGBA);
        
        let mut reader = decoder.read_info(std::io::Cursor::new(data))
            .map_err(|e| {
                error!("GIF解析失败: {}", e);
                AppError::BadRequest("无法解析GIF文件".to_string())
            })?;

        // 读取第一帧
        if let Some(frame) = reader.read_next_frame()
            .map_err(|e| {
                error!("读取GIF第一帧失败: {}", e);
                AppError::BadRequest("无法读取GIF第一帧".to_string())
            })? {
            
            let width = frame.width as u32;
            let height = frame.height as u32;
            let frame_data = frame.buffer.to_vec();
            
            // 检查是否有实际透明度
            let has_actual_transparency = frame_data
                .chunks(4)
                .any(|chunk| chunk.len() >= 4 && chunk[3] < 255);

            if has_actual_transparency {
                // 有透明度，使用RGBA
                let rgba_img = image::RgbaImage::from_raw(width, height, frame_data)
                    .ok_or_else(|| {
                        error!("构建RGBA图像失败");
                        AppError::Internal("构建RGBA图像失败".to_string())
                    })?;

                info!("提取GIF第一帧（RGBA，包含透明度）: {}x{}", width, height);
                Ok(image::DynamicImage::ImageRgba8(rgba_img))
            } else {
                // 无透明度，转换为RGB节省内存
                let expected_rgb_len = (width * height * 3) as usize;
                let mut rgb_data = Vec::with_capacity(expected_rgb_len);
                
                for chunk in frame_data.chunks(4) {
                    if chunk.len() >= 3 {
                        rgb_data.extend_from_slice(&chunk[0..3]);
                    }
                }
                
                // 验证数据长度
                if rgb_data.len() != expected_rgb_len {
                    error!("GIF RGB数据长度不匹配: 期望 {}, 实际 {}", expected_rgb_len, rgb_data.len());
                    // 回退到RGBA格式
                    let rgba_img = image::RgbaImage::from_raw(width, height, frame_data)
                        .ok_or_else(|| {
                            error!("构建RGBA图像失败");
                            AppError::Internal("构建RGBA图像失败".to_string())
                        })?;
                    info!("GIF第一帧回退到RGBA格式: {}x{}", width, height);
                    return Ok(image::DynamicImage::ImageRgba8(rgba_img));
                }
                
                let rgb_img = image::RgbImage::from_raw(width, height, rgb_data)
                    .ok_or_else(|| {
                        error!("构建RGB图像失败");
                        AppError::Internal("构建RGB图像失败".to_string())
                    })?;

                info!("提取GIF第一帧（RGB，无透明度）: {}x{}", width, height);
                Ok(image::DynamicImage::ImageRgb8(rgb_img))
            }
        } else {
            Err(AppError::BadRequest("GIF文件为空或损坏".to_string()))
        }
    }

    /// 高质量图片缩放
    pub fn resize_image_hq(
        img: DynamicImage, 
        max_width: Option<u32>, 
        max_height: Option<u32>
    ) -> Result<DynamicImage, AppError> {
        let (current_width, current_height) = img.dimensions();
        
        // 如果没有指定尺寸，返回原图
        let (target_width, target_height) = match (max_width, max_height) {
            (None, None) => return Ok(img),
            (Some(w), None) => {
                // 只指定宽度，等比缩放
                let ratio = w as f32 / current_width as f32;
                let h = (current_height as f32 * ratio) as u32;
                (w, h)
            },
            (None, Some(h)) => {
                // 只指定高度，等比缩放
                let ratio = h as f32 / current_height as f32;
                let w = (current_width as f32 * ratio) as u32;
                (w, h)
            },
            (Some(w), Some(h)) => {
                // 同时指定宽高，等比缩放到指定范围内
                let width_ratio = w as f32 / current_width as f32;
                let height_ratio = h as f32 / current_height as f32;
                let ratio = width_ratio.min(height_ratio);
                
                let final_w = (current_width as f32 * ratio) as u32;
                let final_h = (current_height as f32 * ratio) as u32;
                (final_w, final_h)
            }
        };

        // 防止放大小图：如果目标尺寸大于原始尺寸，保持原始尺寸
        let (final_width, final_height) = if target_width > current_width || target_height > current_height {
            warn!("目标尺寸大于原始尺寸，保持原始尺寸: {}x{}", current_width, current_height);
            (current_width, current_height)
        } else {
            (target_width, target_height)
        };

        // 如果尺寸没有变化，返回原图
        if final_width == current_width && final_height == current_height {
            return Ok(img);
        }

        info!("高质量缩放: {}x{} -> {}x{}", current_width, current_height, final_width, final_height);

        // 使用Lanczos3算法进行高质量缩放
        let resized = img.resize(final_width, final_height, FilterType::Lanczos3);
        
        Ok(resized)
    }

    /// 高级透明通道移除
    pub fn remove_alpha_channel_advanced(
        img: DynamicImage, 
        bg_color: &Option<BackgroundColor>
    ) -> Result<DynamicImage, AppError> {
        // 如果图片本身没有透明通道，直接返回
        if !img.color().has_alpha() {
            return Ok(img);
        }

        info!("移除透明通道，应用背景色");

        // 确定背景色
        let (bg_r, bg_g, bg_b) = match bg_color {
            Some(BackgroundColor::White) => (255, 255, 255),
            Some(BackgroundColor::Black) => (0, 0, 0),
            Some(BackgroundColor::Custom(r, g, b)) => (*r, *g, *b),
            None => (255, 255, 255), // 默认白色背景
        };

        let (width, height) = img.dimensions();
        let rgba_img = img.to_rgba8();
        
        // 创建新的RGB图像
        let mut rgb_data = Vec::with_capacity((width * height * 3) as usize);
        
        for pixel in rgba_img.pixels() {
            let [r, g, b, a] = pixel.0;
            let alpha = a as f32 / 255.0;
            
            // Alpha blending with background
            let final_r = ((r as f32 * alpha) + (bg_r as f32 * (1.0 - alpha))) as u8;
            let final_g = ((g as f32 * alpha) + (bg_g as f32 * (1.0 - alpha))) as u8;
            let final_b = ((b as f32 * alpha) + (bg_b as f32 * (1.0 - alpha))) as u8;
            
            rgb_data.extend_from_slice(&[final_r, final_g, final_b]);
        }

        let rgb_img = image::RgbImage::from_raw(width, height, rgb_data)
            .ok_or_else(|| {
                error!("创建RGB图像失败");
                AppError::Internal("创建RGB图像失败".to_string())
            })?;

        info!("透明通道移除完成，背景色: RGB({}, {}, {})", bg_r, bg_g, bg_b);
        Ok(DynamicImage::ImageRgb8(rgb_img))
    }

    /// 使用专用编码器进行高级编码
    pub async fn encode_with_specialized_encoder(
        img: DynamicImage,
        format: ImageFormat,
        params: &ImageTransformParams,
    ) -> Result<Vec<u8>, AppError> {
        let mut buffer = Cursor::new(Vec::new());
        let quality = params.quality.unwrap_or(85);

        match format {
            ImageFormat::Jpeg => {
                info!("使用JPEG专用编码器，质量: {}", quality);
                let encoder = JpegEncoder::new_with_quality(&mut buffer, quality);
                img.write_with_encoder(encoder)
                    .map_err(|e| {
                        error!("JPEG编码失败: {}", e);
                        AppError::Internal("JPEG编码失败".to_string())
                    })?;
            },

            ImageFormat::Png => {
                info!("使用PNG专用编码器，智能压缩优化");
                let compression = Self::map_quality_to_png_compression(quality);
                let filter = Self::select_png_filter(&img);
                let encoder = PngEncoder::new_with_quality(&mut buffer, compression, filter);
                img.write_with_encoder(encoder)
                    .map_err(|e| {
                        error!("PNG编码失败: {}", e);
                        AppError::Internal("PNG编码失败".to_string())
                    })?;
            },

            ImageFormat::WebP => {
                info!("使用WebP专用编码器，质量: {}", quality);
                // 使用webp crate进行静态WebP编码
                let (width, height) = img.dimensions();
                
                // 智能选择像素格式：只在需要时使用RGBA
                let (pixel_data, layout) = if img.color().has_alpha() {
                    // 有透明通道：使用RGBA
                    info!("WebP编码使用RGBA格式（保留透明通道）");
                    (img.to_rgba8().into_raw(), webp::PixelLayout::Rgba)
                } else {
                    // 无透明通道：使用RGB（节省25%内存和处理时间）
                    info!("WebP编码使用RGB格式（无透明通道）");
                    (img.to_rgb8().into_raw(), webp::PixelLayout::Rgb)
                };
                
                // 根据质量选择有损或无损编码
                let webp_data = if quality >= 95 {
                    // 高质量使用无损编码
                    info!("WebP使用无损编码");
                    webp::Encoder::new(&pixel_data, layout, width, height)
                        .encode_lossless()
                } else {
                    // 使用有损编码
                    info!("WebP使用有损编码，质量: {}", quality);
                    webp::Encoder::new(&pixel_data, layout, width, height)
                        .encode(quality as f32)
                };
                
                buffer.get_mut().extend_from_slice(&*webp_data);
            },





            ImageFormat::Avif => {
                info!("使用AVIF原生编码器，质量: {}", quality);
                // AVIF需要特殊处理，使用通用编码器
                img.write_to(&mut buffer, format)
                    .map_err(|e| {
                        error!("AVIF编码失败: {}", e);
                        AppError::Internal("AVIF编码失败".to_string())
                    })?;
            },

            ImageFormat::Gif => {
                info!("使用GIF通用编码器");
                img.write_to(&mut buffer, format)
                    .map_err(|e| {
                        error!("GIF编码失败: {}", e);
                        AppError::Internal("GIF编码失败".to_string())
                    })?;
            },

            _ => {
                info!("使用通用编码器处理格式: {:?}", format);
                img.write_to(&mut buffer, format)
                    .map_err(|e| {
                        error!("图片编码失败: {:?} - {}", format, e);
                        AppError::Internal("图片编码失败".to_string())
                    })?;
            }
        }

        let encoded_size = buffer.get_ref().len();
        info!("专用编码器完成，格式: {:?}，大小: {} 字节", format, encoded_size);

        Ok(buffer.into_inner())
    }

    /// 智能PNG压缩级别映射
    fn map_quality_to_png_compression(quality: u8) -> PngCompression {
        match quality {
            95..=100 => PngCompression::Best,      // 最高质量
            85..=94 => PngCompression::Default,    // 平衡模式
            70..=84 => PngCompression::Fast,       // 快速压缩
            1..=69 => PngCompression::Fast,        // 最快压缩
            _ => PngCompression::Default,
        }
    }

    /// 智能PNG滤波器选择
    fn select_png_filter(img: &DynamicImage) -> PngFilter {
        let (width, height) = img.dimensions();
        let pixel_count = width * height;
        
        // 根据图像大小和特征选择最佳滤波器
        if pixel_count > 1_000_000 {
            // 大图像使用自适应滤波器
            PngFilter::Adaptive
        } else if img.color().has_alpha() {
            // 有透明度的图像使用Paeth滤波器
            PngFilter::Paeth
        } else {
            // 一般图像使用Sub滤波器
            PngFilter::Sub
        }
    }
} 