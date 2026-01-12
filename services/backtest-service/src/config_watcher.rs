use crate::config::ConfigManager;
use notify::{RecommendedWatcher, RecursiveMode, Watcher, Event, EventKind};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use log::{info, error};

/// 配置文件监视器
pub struct ConfigWatcher {
    _watcher: Option<RecommendedWatcher>,
}

impl ConfigWatcher {
    /// 创建新的配置监视器
    pub fn new(
        config_path: String,
        config_manager: Arc<ConfigManager>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let path = Path::new(&config_path);

        if !path.exists() {
            return Err(format!("配置文件不存在: {}", config_path).into());
        }

        // 获取配置文件的父目录
        let parent_dir = path.parent().unwrap_or(Path::new("."));

        // 克隆 config_path 以便在闭包中使用
        let config_path_clone = config_path.clone();

        let mut watcher: RecommendedWatcher = Watcher::new(
            move |res: Result<Event, _>| {
                if let Ok(event) = res {
                    Self::handle_event(event, &config_path_clone, config_manager.clone());
                }
            },
            notify::Config::default(),
        )?;

        // 监视配置文件所在目录
        watcher.watch(parent_dir, RecursiveMode::NonRecursive)?;

        info!("📁 正在监视配置文件变化: {}", config_path);

        Ok(Self {
            _watcher: Some(watcher),
        })
    }

    /// 处理文件系统事件
    fn handle_event(event: Event, config_path: &str, config_manager: Arc<ConfigManager>) {
        match event.kind {
            EventKind::Modify(_) | EventKind::Create(_) => {
                // 检查是否是目标配置文件
                for path in event.paths {
                    if path.as_os_str() == config_path {
                        info!("🔄 检测到配置文件变化，等待文件写入完成...");
                        Self::schedule_reload(config_manager.clone());
                        break;
                    }
                }
            }
            _ => {}
        }
    }

    /// 调度配置重新加载（延迟执行以避免重复加载）
    fn schedule_reload(config_manager: Arc<ConfigManager>) {
        tokio::spawn(async move {
            // 等待 2 秒以确保文件写入完成
            tokio::time::sleep(Duration::from_secs(2)).await;

            match config_manager.reload().await {
                Ok(_) => {
                    info!("✅ 配置热重载成功");
                }
                Err(e) => {
                    error!("❌ 配置热重载失败: {}", e);
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_config_reload() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test.toml");

        // 创建初始配置
        let initial_config = r#"
[database]
clickhouse_url = "http://localhost:8123"
pool_size = 10

[server]
host = "0.0.0.0"
port = 8086
metrics_port = 9091
max_body_size = 10

[backtest]
max_backtest_days = 90
default_commission_rate = 0.0003
min_initial_capital = 10000.0
max_concurrent_tasks = 5

[logging]
level = "info"
log_to_file = false
log_file = "logs/backtest-service.log"
"#;

        let mut file = fs::File::create(&config_path).unwrap();
        file.write_all(initial_config.as_bytes()).unwrap();

        let config_manager = Arc::new(
            ConfigManager::new(config_path.to_str().unwrap().to_string()).unwrap()
        );

        let initial = config_manager.get_config().await;
        assert_eq!(initial.server.port, 8086);

        // 修改配置
        let modified_config = initial_config.replace("port = 8086", "port = 9090");
        let mut file = fs::File::create(&config_path).unwrap();
        file.write_all(modified_config.as_bytes()).unwrap();

        // 重新加载
        config_manager.reload().await.unwrap();

        let reloaded = config_manager.get_config().await;
        assert_eq!(reloaded.server.port, 9090);
    }
}
