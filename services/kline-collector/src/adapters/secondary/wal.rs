//! WAL (Write-Ahead Log) 日志机制
//!
//! 提供数据持久化保证，防止 ClickHouse 写入失败时数据丢失

use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
use tracing::{debug, error, info, warn};

use crate::domain::entities::KlineData;

/// WAL 日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
struct WalEntry {
    /// 时间戳
    timestamp: i64,
    /// 序列号
    sequence: u64,
    /// 数据类型
    data_type: String,
    /// K线数据（JSON序列化）
    kline_data: String,
}

/// WAL 日志管理器
pub struct WalManager {
    /// WAL 文件路径
    wal_path: PathBuf,
    /// 当前序列号
    sequence: u64,
    /// 是否启用
    enabled: bool,
    /// 写入器
    writer: Option<BufWriter<File>>,
}

impl WalManager {
    /// 创建新的 WAL 管理器
    pub fn new(wal_dir: &str, enabled: bool) -> Result<Self> {
        let wal_path = PathBuf::from(wal_dir).join("kline.wal");

        if enabled {
            // 确保目录存在
            if let Some(parent) = wal_path.parent() {
                std::fs::create_dir_all(parent)
                    .context("创建WAL目录失败")?;
            }

            info!("WAL 日志已启用: {:?}", wal_path);
        } else {
            info!("WAL 日志已禁用");
        }

        Ok(Self {
            wal_path,
            sequence: 0,
            enabled,
            writer: None,
        })
    }

    /// 初始化 WAL（打开文件）
    pub fn init(&mut self) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.wal_path)
            .context("打开WAL文件失败")?;

        self.writer = Some(BufWriter::new(file));

        // 读取现有条目以获取当前序列号
        self.sequence = self.get_last_sequence()?;

        debug!("WAL 初始化完成，当前序列号: {}", self.sequence);

        Ok(())
    }

    /// 获取最后一个序列号
    fn get_last_sequence(&self) -> Result<u64> {
        if !self.wal_path.exists() {
            return Ok(0);
        }

        let file = File::open(&self.wal_path).context("打开WAL文件失败")?;
        let reader = BufReader::new(file);
        let mut last_seq = 0u64;

        for line in reader.lines() {
            if let Ok(entry_json) = line {
                if let Ok(entry) = serde_json::from_str::<WalEntry>(&entry_json) {
                    last_seq = last_seq.max(entry.sequence);
                }
            }
        }

        Ok(last_seq)
    }

    /// 写入K线数据到 WAL
    pub fn write_kline(&mut self, kline: &KlineData) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let entry = WalEntry {
            timestamp: Utc::now().timestamp(),
            sequence: self.sequence + 1,
            data_type: "kline".to_string(),
            kline_data: serde_json::to_string(kline)?,
        };

        self.write_entry(&entry)?;

        self.sequence += 1;
        debug!("写入WAL: 序列号={}, 代码={}, 周期={}",
            entry.sequence, kline.code, kline.period
        );

        Ok(())
    }

    /// 写入 WAL 条目
    fn write_entry(&mut self, entry: &WalEntry) -> Result<()> {
        let writer = self.writer.as_mut().context("WAL写入器未初始化")?;

        let entry_json = serde_json::to_string(entry)?;
        writeln!(writer, "{}", entry_json).context("写入WAL条目失败")?;
        writer.flush().context("刷新WAL缓冲区失败")?;

        Ok(())
    }

    /// 重放 WAL（从指定的序列号开始）
    pub fn replay_from(&self, start_sequence: u64) -> Result<Vec<KlineData>> {
        if !self.enabled || !self.wal_path.exists() {
            return Ok(Vec::new());
        }

        info!("开始重放WAL，起始序列号: {}", start_sequence);

        let file = File::open(&self.wal_path).context("打开WAL文件失败")?;
        let reader = BufReader::new(file);
        let mut klines = Vec::new();

        for line in reader.lines() {
            if let Ok(entry_json) = line {
                if let Ok(entry) = serde_json::from_str::<WalEntry>(&entry_json) {
                    if entry.sequence >= start_sequence {
                        if let Ok(kline) = serde_json::from_str::<KlineData>(&entry.kline_data) {
                            klines.push(kline);
                        } else {
                            warn!("解析WAL中的K线数据失败: 序列号={}", entry.sequence);
                        }
                    }
                }
            }
        }

        info!("WAL重放完成: 加载 {} 条K线", klines.len());

        Ok(klines)
    }

    /// 清理 WAL（删除已确认的数据）
    pub fn cleanup(&mut self, confirmed_sequence: u64) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        // 读取所有条目
        let mut entries = Vec::new();
        let file = File::open(&self.wal_path).context("打开WAL文件失败")?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            if let Ok(entry_json) = line {
                if let Ok(entry) = serde_json::from_str::<WalEntry>(&entry_json) {
                    if entry.sequence > confirmed_sequence {
                        entries.push(entry_json);
                    }
                }
            }
        }

        // 重写文件（只保留未确认的条目）
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.wal_path)
            .context("创建WAL文件失败")?;

        let mut writer = BufWriter::new(file);
        for entry in &entries {
            writeln!(writer, "{}", entry).context("写入WAL条目失败")?;
        }
        writer.flush().context("刷新WAL缓冲区失败")?;

        debug!("WAL清理完成: 保留 {} 条未确认条目", entries.len());

        Ok(())
    }

    /// 刷新缓冲区
    pub fn flush(&mut self) -> Result<()> {
        if let Some(writer) = self.writer.as_mut() {
            writer.flush().context("刷新WAL缓冲区失败")?;
        }
        Ok(())
    }

    /// 获取当前序列号
    pub fn get_sequence(&self) -> u64 {
        self.sequence
    }

    /// 是否启用
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// 获取WAL文件大小
    pub fn get_file_size(&self) -> Result<u64> {
        if !self.wal_path.exists() {
            return Ok(0);
        }

        Ok(std::fs::metadata(&self.wal_path)
            .context("获取WAL文件大小失败")?
            .len())
    }
}

impl Drop for WalManager {
    fn drop(&mut self) {
        if let Err(e) = self.flush() {
            error!("WAL刷新失败（Drop中）: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_wal_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let wal_dir = temp_dir.path().to_str().unwrap();

        let manager = WalManager::new(wal_dir, true).unwrap();
        assert!(manager.is_enabled());
        assert_eq!(manager.get_sequence(), 0);
    }

    #[test]
    fn test_wal_disabled() {
        let temp_dir = TempDir::new().unwrap();
        let wal_dir = temp_dir.path().to_str().unwrap();

        let mut manager = WalManager::new(wal_dir, false).unwrap();
        manager.init().unwrap();

        assert!(!manager.is_enabled());
        assert_eq!(manager.get_sequence(), 0);
    }

    #[test]
    fn test_wal_write_and_replay() {
        let temp_dir = TempDir::new().unwrap();
        let wal_dir = temp_dir.path().to_str().unwrap();

        let mut manager = WalManager::new(wal_dir, true).unwrap();
        manager.init().unwrap();

        // 写入测试K线数据
        let kline = KlineData {
            timestamp: Utc::now().timestamp(),
            code: "000001".to_string(),
            name: "测试".to_string(),
            period: "1m".to_string(),
            open: 10.0,
            high: 11.0,
            low: 9.5,
            close: 10.5,
            volume: 1000.0,
            amount: 10500.0,
            trade_count: 10,
            source: "test".to_string(),
        };

        manager.write_kline(&kline).unwrap();
        assert_eq!(manager.get_sequence(), 1);

        // 重放WAL
        let replayed = manager.replay_from(0).unwrap();
        assert_eq!(replayed.len(), 1);
        assert_eq!(replayed[0].code, "000001");
    }

    #[test]
    fn test_wal_cleanup() {
        let temp_dir = TempDir::new().unwrap();
        let wal_dir = temp_dir.path().to_str().unwrap();

        let mut manager = WalManager::new(wal_dir, true).unwrap();
        manager.init().unwrap();

        // 写入3条数据
        for i in 1..=3 {
            let kline = KlineData {
                timestamp: Utc::now().timestamp() + i as i64,
                code: format!("00000{}", i),
                name: "测试".to_string(),
                period: "1m".to_string(),
                open: 10.0,
                high: 11.0,
                low: 9.5,
                close: 10.5,
                volume: 1000.0,
                amount: 10500.0,
                trade_count: 10,
                source: "test".to_string(),
            };
            manager.write_kline(&kline).unwrap();
        }

        // 清理前2条
        manager.cleanup(2).unwrap();

        // 重放应该只返回第3条
        let replayed = manager.replay_from(0).unwrap();
        assert_eq!(replayed.len(), 1);
    }
}
