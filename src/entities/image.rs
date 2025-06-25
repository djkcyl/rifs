use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::Set;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "images")]
pub struct Model {
    /// 文件哈希值（主键）
    #[sea_orm(primary_key, auto_increment = false)]
    pub hash: String,

    /// 文件大小（字节）
    pub size: i64,

    /// MIME 类型
    pub mime_type: String,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 最后访问时间
    pub last_accessed: Option<DateTime<Utc>>,

    /// 文件扩展名
    pub extension: String,

    /// 访问次数
    #[sea_orm(default_value = 0)]
    pub access_count: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for crate::models::ImageInfo {
    fn from(model: Model) -> Self {
        Self {
            hash: model.hash,
            size: model.size as u64,
            mime_type: model.mime_type,
            created_at: model.created_at,
            last_accessed: model.last_accessed,
            extension: model.extension,
            access_count: model.access_count,
        }
    }
}

impl From<&crate::models::ImageInfo> for ActiveModel {
    fn from(info: &crate::models::ImageInfo) -> Self {
        Self {
            hash: Set(info.hash.clone()),
            size: Set(info.size as i64),
            mime_type: Set(info.mime_type.clone()),
            created_at: Set(info.created_at),
            last_accessed: Set(info.last_accessed),
            extension: Set(info.extension.clone()),
            access_count: Set(info.access_count),
        }
    }
}
