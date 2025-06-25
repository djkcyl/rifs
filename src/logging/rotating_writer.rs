use chrono;
use std::fs::{metadata, File, OpenOptions};
use std::io::{Result as IoResult, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tracing::info;

/// 复合轮转的日志写入器（同时支持按大小和按天轮转）
pub struct HybridRotatingWriter {
    log_dir: PathBuf,
    max_size: u64,
    current_file: Arc<Mutex<Option<File>>>,
    current_size: Arc<Mutex<u64>>,
    current_date: Arc<Mutex<String>>,
}

impl HybridRotatingWriter {
    /// 创建新的复合轮转写入器
    ///
    /// # 参数
    /// * `log_dir` - 日志目录路径
    /// * `max_size` - 最大文件大小（字节），0表示不按大小轮转
    pub fn new(log_dir: &Path, max_size: u64) -> IoResult<Self> {
        // 确保目录存在
        std::fs::create_dir_all(log_dir)?;

        let current_date = Self::get_current_date();
        let base_path = log_dir.join(format!("rifs.{}.log", current_date));

        let current_size = if base_path.exists() {
            metadata(&base_path)?.len()
        } else {
            0
        };

        Ok(Self {
            log_dir: log_dir.to_path_buf(),
            max_size,
            current_file: Arc::new(Mutex::new(None)),
            current_size: Arc::new(Mutex::new(current_size)),
            current_date: Arc::new(Mutex::new(current_date)),
        })
    }

    /// 获取当前日期字符串（UTC格式：YYYY-MM-DD）
    fn get_current_date() -> String {
        chrono::Utc::now().format("%Y-%m-%d").to_string()
    }

    /// 获取当前日志文件的路径
    fn get_current_file_path(&self) -> PathBuf {
        let date = self.current_date.lock().unwrap();
        self.log_dir.join(format!("rifs.{}.log", *date))
    }

    /// 检查是否需要轮转，并执行轮转
    fn rotate_if_needed(&self) -> IoResult<()> {
        let current_date = Self::get_current_date();
        let stored_date = self.current_date.lock().unwrap().clone();
        let current_size = *self.current_size.lock().unwrap();

        // 检查是否需要按天轮转
        if current_date != stored_date {
            self.rotate_by_date(current_date)?;
            return Ok(());
        }

        // 检查是否需要按大小轮转
        if self.max_size > 0 && current_size >= self.max_size {
            self.rotate_by_size()?;
        }

        Ok(())
    }

    /// 按天轮转：新的一天开始时创建新的日志文件
    fn rotate_by_date(&self, new_date: String) -> IoResult<()> {
        info!(
            "日志按天轮转: {} -> {}",
            self.current_date.lock().unwrap(),
            new_date
        );

        // 关闭当前文件
        {
            let mut file_guard = self.current_file.lock().unwrap();
            if let Some(file) = file_guard.take() {
                drop(file);
            }
        }

        // 更新日期和重置大小
        *self.current_date.lock().unwrap() = new_date;
        *self.current_size.lock().unwrap() = 0;

        Ok(())
    }

    /// 按大小轮转：当前文件大小超过限制时轮转
    /// 文件命名格式：rifs.YYYY-MM-DD.N.log，其中N是序号
    fn rotate_by_size(&self) -> IoResult<()> {
        let current_date = self.current_date.lock().unwrap().clone();

        // 找到当前日期下一个可用的序号
        let mut sequence = 1;
        loop {
            let rotated_path = self
                .log_dir
                .join(format!("rifs.{}.{}.log", current_date, sequence));
            if !rotated_path.exists() {
                break;
            }
            sequence += 1;
        }

        // 关闭当前文件
        {
            let mut file_guard = self.current_file.lock().unwrap();
            if let Some(file) = file_guard.take() {
                drop(file);
            }
        }

        // 移动当前文件到新的序号文件
        let current_path = self.get_current_file_path();
        let rotated_path = self
            .log_dir
            .join(format!("rifs.{}.{}.log", current_date, sequence));

        if current_path.exists() {
            std::fs::rename(&current_path, &rotated_path)?;
            info!(
                "日志按大小轮转: {} -> {}",
                current_path.display(),
                rotated_path.display()
            );
        }

        // 重置大小计数器
        *self.current_size.lock().unwrap() = 0;

        Ok(())
    }

    /// 获取或创建当前日志文件
    fn get_or_create_file(&self) -> IoResult<()> {
        let mut file_guard = self.current_file.lock().unwrap();

        if file_guard.is_none() {
            let file_path = self.get_current_file_path();
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&file_path)?;
            *file_guard = Some(file);
        }

        Ok(())
    }
}

impl Write for HybridRotatingWriter {
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        self.rotate_if_needed()?;
        self.get_or_create_file()?;

        let bytes_written = {
            let mut file_guard = self.current_file.lock().unwrap();
            if let Some(ref mut file) = *file_guard {
                file.write(buf)?
            } else {
                return Err(std::io::Error::new(std::io::ErrorKind::Other, "文件未打开"));
            }
        };

        // 更新大小计数器
        {
            let mut size_guard = self.current_size.lock().unwrap();
            *size_guard += bytes_written as u64;
        }

        Ok(bytes_written)
    }

    fn flush(&mut self) -> IoResult<()> {
        let mut file_guard = self.current_file.lock().unwrap();
        if let Some(ref mut file) = *file_guard {
            file.flush()?;
        }
        Ok(())
    }
}
