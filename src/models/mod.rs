use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

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