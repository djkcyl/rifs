use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;

/// 字节大小类型，支持带单位的配置
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ByteSize(pub u64);

impl ByteSize {
    /// 创建新的字节大小
    pub fn new(bytes: u64) -> Self {
        Self(bytes)
    }

    /// 获取字节数
    pub fn as_bytes(self) -> u64 {
        self.0
    }

    /// 从 KB 创建
    pub fn kb(kb: u64) -> Self {
        Self(kb * 1024)
    }

    /// 从 MB 创建
    pub fn mb(mb: u64) -> Self {
        Self(mb * 1024 * 1024)
    }

    /// 从 GB 创建
    pub fn gb(gb: u64) -> Self {
        Self(gb * 1024 * 1024 * 1024)
    }

    /// 格式化为人类可读的字符串
    pub fn to_human_string(self) -> String {
        let bytes = self.0;

        if bytes == 0 {
            return "0B".to_string();
        }

        const UNITS: &[(&str, u64)] = &[
            ("GB", 1024 * 1024 * 1024),
            ("MB", 1024 * 1024),
            ("KB", 1024),
            ("B", 1),
        ];

        for (unit, size) in UNITS {
            if bytes >= *size {
                if bytes % size == 0 {
                    return format!("{}{}", bytes / size, unit);
                } else {
                    return format!("{:.1}{}", bytes as f64 / *size as f64, unit);
                }
            }
        }

        format!("{}B", bytes)
    }
}

impl FromStr for ByteSize {
    type Err = ByteSizeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if s.is_empty() {
            return Err(ByteSizeParseError::Empty);
        }

        // 查找数字部分的结束位置
        let mut digit_end = 0;
        let mut has_decimal = false;

        for (i, c) in s.char_indices() {
            if c.is_ascii_digit() {
                digit_end = i + 1;
            } else if c == '.' && !has_decimal {
                has_decimal = true;
                digit_end = i + 1;
            } else {
                break;
            }
        }

        if digit_end == 0 {
            return Err(ByteSizeParseError::InvalidNumber);
        }

        let (number_part, unit_part) = s.split_at(digit_end);
        let unit_part = unit_part.trim().to_uppercase();

        // 解析数字部分
        let number: f64 = number_part
            .parse()
            .map_err(|_| ByteSizeParseError::InvalidNumber)?;

        if number < 0.0 {
            return Err(ByteSizeParseError::Negative);
        }

        // 解析单位部分
        let multiplier = match unit_part.as_str() {
            "" | "B" | "BYTE" | "BYTES" => 1,
            "K" | "KB" | "KIB" | "KILOBYTE" | "KILOBYTES" => 1024,
            "M" | "MB" | "MIB" | "MEGABYTE" | "MEGABYTES" => 1024 * 1024,
            "G" | "GB" | "GIB" | "GIGABYTE" | "GIGABYTES" => 1024 * 1024 * 1024,
            "T" | "TB" | "TIB" | "TERABYTE" | "TERABYTES" => 1024_u64.pow(4),
            "P" | "PB" | "PIB" | "PETABYTE" | "PETABYTES" => 1024_u64.pow(5),
            _ => return Err(ByteSizeParseError::InvalidUnit(unit_part)),
        };

        let bytes = (number * multiplier as f64) as u64;
        Ok(ByteSize(bytes))
    }
}

impl fmt::Display for ByteSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_human_string())
    }
}

impl From<u64> for ByteSize {
    fn from(bytes: u64) -> Self {
        ByteSize(bytes)
    }
}

impl From<ByteSize> for u64 {
    fn from(size: ByteSize) -> Self {
        size.0
    }
}

impl Serialize for ByteSize {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            // 对于人类可读格式（如 TOML），序列化为带单位的字符串
            serializer.serialize_str(&self.to_human_string())
        } else {
            // 对于二进制格式，序列化为数字
            serializer.serialize_u64(self.0)
        }
    }
}

impl<'de> Deserialize<'de> for ByteSize {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;

        if deserializer.is_human_readable() {
            // 尝试作为字符串解析
            let s = String::deserialize(deserializer)?;
            s.parse().map_err(D::Error::custom)
        } else {
            // 作为数字解析
            let bytes = u64::deserialize(deserializer)?;
            Ok(ByteSize(bytes))
        }
    }
}

/// 字节大小解析错误
#[derive(Debug, Clone)]
pub enum ByteSizeParseError {
    Empty,
    InvalidNumber,
    InvalidUnit(String),
    Negative,
}

impl fmt::Display for ByteSizeParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ByteSizeParseError::Empty => write!(f, "空字符串"),
            ByteSizeParseError::InvalidNumber => write!(f, "无效的数字格式"),
            ByteSizeParseError::InvalidUnit(unit) => write!(f, "无效的单位: {}", unit),
            ByteSizeParseError::Negative => write!(f, "不能为负数"),
        }
    }
}

impl std::error::Error for ByteSizeParseError {}
