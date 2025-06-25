use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::Set;
use serde::{Deserialize, Serialize};

/// 缓存实体模型
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "cache")]
pub struct Model {
    /// 缓存键（主键）- 基于原图hash和转换参数生成
    #[sea_orm(primary_key, auto_increment = false)]
    pub cache_key: String,

    /// 原图hash
    pub original_hash: String,

    /// 转换参数字符串
    pub transform_params: String,

    /// 缓存文件路径
    pub file_path: String,

    /// 缓存文件大小（字节）
    pub file_size: i64,

    /// 缓存结果的MIME类型
    pub mime_type: String,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 最后访问时间
    pub last_accessed: DateTime<Utc>,

    /// 访问次数
    #[sea_orm(default_value = 0)]
    pub access_count: i64,

    /// 缓存热度评分（基于访问频率和时间的综合评分）
    #[sea_orm(default_value = 0.0)]
    pub heat_score: f64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for crate::models::CacheInfo {
    fn from(model: Model) -> Self {
        Self {
            cache_key: model.cache_key,
            original_hash: model.original_hash,
            transform_params: model.transform_params,
            file_path: model.file_path,
            file_size: model.file_size as u64,
            mime_type: model.mime_type,
            created_at: model.created_at,
            last_accessed: model.last_accessed,
            access_count: model.access_count,
            heat_score: model.heat_score,
        }
    }
}

impl From<&crate::models::CacheInfo> for ActiveModel {
    fn from(info: &crate::models::CacheInfo) -> Self {
        Self {
            cache_key: Set(info.cache_key.clone()),
            original_hash: Set(info.original_hash.clone()),
            transform_params: Set(info.transform_params.clone()),
            file_path: Set(info.file_path.clone()),
            file_size: Set(info.file_size as i64),
            mime_type: Set(info.mime_type.clone()),
            created_at: Set(info.created_at),
            last_accessed: Set(info.last_accessed),
            access_count: Set(info.access_count),
            heat_score: Set(info.heat_score),
        }
    }
}
