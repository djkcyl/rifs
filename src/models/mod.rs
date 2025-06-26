use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 图片信息结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageInfo {
    /// 文件哈希值（主键）
    pub hash: String,
    /// 文件大小（字节）
    pub size: u64,
    /// MIME 类型
    pub mime_type: String,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 最后访问时间
    pub last_accessed: Option<DateTime<Utc>>,
    /// 文件扩展名
    pub extension: String,
    /// 访问次数
    pub access_count: i64,
}

impl ImageInfo {
    /// 获取存储的文件名（基于hash和扩展名）
    pub fn stored_name(&self) -> String {
        format!("{}.{}", self.hash, self.extension)
    }
}

/// 图片查询参数
#[derive(Debug, Deserialize)]
pub struct ImageQuery {
    /// 分页大小
    pub limit: Option<u64>,
    /// 偏移量
    pub offset: Option<u64>,
    /// 按字段排序
    pub order_by: Option<String>,
    /// 排序方向（asc/desc）
    pub order_dir: Option<String>,
    /// 文件类型过滤
    pub mime_type: Option<String>,
    /// 最小文件大小
    pub min_size: Option<u64>,
    /// 最大文件大小
    pub max_size: Option<u64>,
    /// 开始时间
    pub start_time: Option<DateTime<Utc>>,
    /// 结束时间
    pub end_time: Option<DateTime<Utc>>,
    /// 搜索关键词（文件名）
    pub search: Option<String>,
}

/// 图片统计信息
#[derive(Debug, Serialize)]
pub struct ImageStats {
    /// 总图片数量
    pub total_count: i64,
    /// 总存储大小
    pub total_size: i64,
    /// 平均文件大小
    pub average_size: f64,
    /// 按类型分组的统计
    pub by_type: Vec<TypeStat>,
    /// 按时间分组的统计
    pub by_time: Vec<TimeStat>,
}

/// 类型统计
#[derive(Debug, Serialize)]
pub struct TypeStat {
    pub mime_type: String,
    pub count: i64,
    pub total_size: i64,
}

/// 时间统计
#[derive(Debug, Serialize)]
pub struct TimeStat {
    pub date: String,
    pub count: i64,
    pub total_size: i64,
}

/// 上传响应结构体
#[derive(Debug, Serialize)]
pub struct UploadResponse {
    /// 上传是否成功
    pub success: bool,
    /// 响应消息
    pub message: String,
    /// 图片信息（成功时）
    pub data: Option<ImageInfo>,
}

/// 错误响应结构体
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    /// 错误状态
    pub success: bool,
    /// 错误消息
    pub message: String,
    /// 错误代码
    pub code: Option<u16>,
}

/// 图片转换参数
#[derive(Debug, Clone)]
pub struct ImageTransformParams {
    /// 目标宽度
    pub width: Option<u32>,
    /// 目标高度
    pub height: Option<u32>,
    /// 目标格式
    pub format: Option<String>,
    /// 图片质量 (1-100)
    pub quality: Option<u8>,
    /// 是否去除透明通道
    pub no_alpha: bool,
    /// 去除透明通道后的背景色
    pub background_color: Option<BackgroundColor>,
}

/// 背景色选项
#[derive(Debug, Clone)]
pub enum BackgroundColor {
    White,
    Black,
    Custom(u8, u8, u8), // RGB
}

impl ImageTransformParams {
    /// 从URL参数字符串解析转换参数
    /// 格式: w1200_h1200_jpeg_naw_q80
    pub fn parse(params_str: &str) -> Result<Self, String> {
        // 如果参数字符串为空，返回空的转换参数
        if params_str.trim().is_empty() {
            return Ok(Self {
                width: None,
                height: None,
                format: None,
                quality: None,
                no_alpha: false,
                background_color: None,
            });
        }

        let mut params = ImageTransformParams {
            width: None,
            height: None,
            format: None,
            quality: None,
            no_alpha: false,
            background_color: None,
        };

        for param in params_str.split('_') {
            if param.is_empty() {
                continue;
            }

            // 优先检查是否为有效的图片格式
            if Self::is_valid_format(param) {
                // 图片格式
                params.format = Some(param.to_lowercase());
            } else if let Some(bg_part) = param.strip_prefix("na") {
                // 去除透明通道及背景色设置
                params.no_alpha = true;

                if !bg_part.is_empty() {
                    if bg_part == "w" {
                        // naw - 白色背景
                        params.background_color = Some(BackgroundColor::White);
                    } else if bg_part == "b" {
                        // nab - 黑色背景
                        params.background_color = Some(BackgroundColor::Black);
                    } else if bg_part.starts_with('#') && bg_part.len() == 7 {
                        // na#ffffff - 十六进制颜色
                        if let Ok((r, g, b)) = Self::parse_hex_color(bg_part) {
                            params.background_color = Some(BackgroundColor::Custom(r, g, b));
                        }
                    }
                } else {
                    // 只有na，使用默认白色背景
                    params.background_color = Some(BackgroundColor::White);
                }
            } else if let Some(width_str) = param.strip_prefix('w') {
                // 宽度参数
                if let Ok(width) = width_str.parse::<u32>() {
                    params.width = Some(width);
                }
            } else if let Some(height_str) = param.strip_prefix('h') {
                // 高度参数
                if let Ok(height) = height_str.parse::<u32>() {
                    params.height = Some(height);
                }
            } else if let Some(quality_str) = param.strip_prefix('q') {
                // 质量参数
                if let Ok(quality) = quality_str.parse::<u8>() {
                    if quality > 0 && quality <= 100 {
                        params.quality = Some(quality);
                    }
                }
            }
        }

        Ok(params)
    }

    /// 检查是否为有效的图片格式
    fn is_valid_format(format: &str) -> bool {
        matches!(
            format.to_lowercase().as_str(),
            "jpeg" | "jpg" | "png" | "gif" | "webp" | "avif" | "ico"
        )
    }

    /// 检查是否需要进行转换
    pub fn needs_transform(&self) -> bool {
        self.width.is_some()
            || self.height.is_some()
            || self.format.is_some()
            || self.quality.is_some()
            || self.no_alpha
    }

    /// 生成标准化的参数字符串（用于缓存键生成）
    /// 按固定顺序排列参数，确保相同功能的转换生成相同的缓存键
    pub fn to_normalized_string(&self) -> String {
        let mut parts = Vec::new();

        // 按固定顺序添加参数
        if let Some(width) = self.width {
            parts.push(format!("w{}", width));
        }

        if let Some(height) = self.height {
            parts.push(format!("h{}", height));
        }

        if let Some(ref format) = self.format {
            parts.push(format.clone());
        }

        if let Some(quality) = self.quality {
            parts.push(format!("q{}", quality));
        }

        if self.no_alpha {
            match &self.background_color {
                Some(BackgroundColor::White) => parts.push("naw".to_string()),
                Some(BackgroundColor::Black) => parts.push("nab".to_string()),
                Some(BackgroundColor::Custom(r, g, b)) => {
                    parts.push(format!("na#{:02x}{:02x}{:02x}", r, g, b));
                }
                None => parts.push("na".to_string()),
            }
        }

        parts.join("_")
    }

    /// 获取目标MIME类型
    pub fn target_mime_type(&self) -> Option<String> {
        self.format.as_ref().map(|f| {
            match f.as_str() {
                // 传统格式
                "jpeg" | "jpg" => "image/jpeg",
                "png" => "image/png",
                "gif" => "image/gif",

                // 现代格式
                "webp" => "image/webp",
                "avif" => "image/avif",

                // 图标格式
                "ico" => "image/x-icon",

                _ => "image/jpeg", // 默认JPEG
            }
            .to_string()
        })
    }

    /// 解析十六进制颜色
    fn parse_hex_color(hex: &str) -> Result<(u8, u8, u8), String> {
        if hex.len() != 7 || !hex.starts_with('#') {
            return Err("十六进制颜色格式必须为#RRGGBB".to_string());
        }

        let r = u8::from_str_radix(&hex[1..3], 16).map_err(|_| "无效的红色分量".to_string())?;
        let g = u8::from_str_radix(&hex[3..5], 16).map_err(|_| "无效的绿色分量".to_string())?;
        let b = u8::from_str_radix(&hex[5..7], 16).map_err(|_| "无效的蓝色分量".to_string())?;

        Ok((r, g, b))
    }
}

/// 缓存信息结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheInfo {
    /// 缓存键（主键）
    pub cache_key: String,
    /// 原图hash
    pub original_hash: String,
    /// 转换参数字符串
    pub transform_params: String,
    /// 缓存文件路径
    pub file_path: String,
    /// 缓存文件大小（字节）
    pub file_size: u64,
    /// 缓存结果的MIME类型
    pub mime_type: String,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 最后访问时间
    pub last_accessed: DateTime<Utc>,
    /// 访问次数
    pub access_count: i64,
    /// 缓存热度评分
    pub heat_score: f64,
}

/// 缓存统计信息
#[derive(Debug, Serialize)]
pub struct CacheStats {
    /// 缓存总数
    pub total_count: i64,
    /// 缓存总大小（字节）
    pub total_size: i64,
    /// 平均文件大小
    pub average_size: f64,
    /// 命中率（过去24小时）
    pub hit_rate: f64,
    /// 热门缓存项（按访问次数排序）
    pub top_cached: Vec<CacheInfo>,
    /// 最近清理时间
    pub last_cleanup: Option<DateTime<Utc>>,
}

/// 缓存清理结果
#[derive(Debug, Serialize)]
pub struct CacheCleanupResult {
    /// 清理的缓存项数量
    pub cleaned_count: u64,
    /// 释放的存储空间（字节）
    pub freed_space: u64,
    /// 清理策略应用情况
    pub applied_policies: Vec<String>,
    /// 清理耗时（毫秒）
    pub duration_ms: u64,
}
