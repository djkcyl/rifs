use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 图片信息结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageInfo {
    /// 图片唯一标识符
    pub id: Uuid,
    /// 存储的文件名
    pub stored_name: String,
    /// 文件大小（字节）
    pub size: u64,
    /// MIME 类型
    pub mime_type: String,
    /// 上传时间戳
    pub upload_time: i64,
    /// 文件扩展名
    pub extension: String,
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