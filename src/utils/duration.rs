use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;

/// 时间长度类型，支持带单位的配置
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Duration(pub u64);

impl Duration {
    /// 创建新的时间长度（秒）
    pub fn new(seconds: u64) -> Self {
        Self(seconds)
    }

    /// 获取秒数
    pub fn as_seconds(self) -> u64 {
        self.0
    }

    /// 从秒创建
    pub fn seconds(seconds: u64) -> Self {
        Self(seconds)
    }

    /// 从分钟创建
    pub fn minutes(minutes: u64) -> Self {
        Self(minutes * 60)
    }

    /// 从小时创建
    pub fn hours(hours: u64) -> Self {
        Self(hours * 3600)
    }

    /// 从天创建
    pub fn days(days: u64) -> Self {
        Self(days * 86400)
    }

    /// 从周创建
    pub fn weeks(weeks: u64) -> Self {
        Self(weeks * 604800)
    }

    /// 格式化为人类可读的字符串
    pub fn to_human_string(self) -> String {
        let seconds = self.0;

        if seconds == 0 {
            return "0s".to_string();
        }

        const UNITS: &[(&str, u64)] = &[
            ("w", 604800), // 周
            ("d", 86400),  // 天
            ("h", 3600),   // 小时
            ("m", 60),     // 分钟
            ("s", 1),      // 秒
        ];

        for (unit, size) in UNITS {
            if seconds >= *size {
                if seconds % size == 0 {
                    return format!("{}{}", seconds / size, unit);
                } else {
                    return format!("{:.1}{}", seconds as f64 / *size as f64, unit);
                }
            }
        }

        format!("{}s", seconds)
    }
}

impl FromStr for Duration {
    type Err = DurationParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if s.is_empty() {
            return Err(DurationParseError::Empty);
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
            return Err(DurationParseError::InvalidNumber);
        }

        let (number_part, unit_part) = s.split_at(digit_end);
        let unit_part = unit_part.trim().to_lowercase();

        // 解析数字部分
        let number: f64 = number_part
            .parse()
            .map_err(|_| DurationParseError::InvalidNumber)?;

        if number < 0.0 {
            return Err(DurationParseError::Negative);
        }

        // 解析单位部分
        let multiplier = match unit_part.as_str() {
            "" | "s" | "sec" | "secs" | "second" | "seconds" => 1,
            "m" | "min" | "mins" | "minute" | "minutes" => 60,
            "h" | "hr" | "hrs" | "hour" | "hours" => 3600,
            "d" | "day" | "days" => 86400,
            "w" | "week" | "weeks" => 604800,
            "mo" | "month" | "months" => 2629746, // 平均月长度
            "y" | "year" | "years" => 31556952,   // 平均年长度
            _ => return Err(DurationParseError::InvalidUnit(unit_part)),
        };

        let seconds = (number * multiplier as f64) as u64;
        Ok(Duration(seconds))
    }
}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_human_string())
    }
}

impl From<u64> for Duration {
    fn from(seconds: u64) -> Self {
        Duration(seconds)
    }
}

impl From<Duration> for u64 {
    fn from(duration: Duration) -> Self {
        duration.0
    }
}

impl Serialize for Duration {
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

impl<'de> Deserialize<'de> for Duration {
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
            let seconds = u64::deserialize(deserializer)?;
            Ok(Duration(seconds))
        }
    }
}

/// 时间长度解析错误
#[derive(Debug, Clone)]
pub enum DurationParseError {
    Empty,
    InvalidNumber,
    InvalidUnit(String),
    Negative,
}

impl fmt::Display for DurationParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DurationParseError::Empty => write!(f, "空字符串"),
            DurationParseError::InvalidNumber => write!(f, "无效的数字格式"),
            DurationParseError::InvalidUnit(unit) => write!(f, "无效的时间单位: {}", unit),
            DurationParseError::Negative => write!(f, "不能为负数"),
        }
    }
}

impl std::error::Error for DurationParseError {}
