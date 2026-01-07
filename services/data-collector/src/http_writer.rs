// HTTP 客户端写入器（备选方案：绕过 ClickHouse Rust 客户端 schema 验证问题）
use crate::types::WriteMethod;
use anyhow::Result;
use serde::Serialize;
use tracing::{debug, info, warn};

/// HTTP 客户端配置
pub struct HttpClientConfig {
    pub base_url: String,
    pub database: String,
    pub write_method: WriteMethod,
}

/// HTTP 客户端
pub struct HttpClient {
    config: HttpClientConfig,
}

impl HttpClient {
    /// 创建新的 HTTP 客户端
    pub fn new(config: HttpClientConfig) -> Self {
        Self { config }
    }

    /// 写入数据到 ClickHouse（使用 HTTP 接口）
    pub async fn write_to_clickhouse<T: Serialize>(
        &self,
        table: &str,
        data: &[T],
    ) -> Result<()> {
        // 使用 HTTP POST 直接插入数据
        info!("使用 HTTP 客户端写入 {} 行数据到 {}", data.len(), table);

        // 构建请求 URL
        let url = format!(
            "{}/?query=INSERT+INTO+{}+FORMAT+JSON",
            self.config.base_url,
            table
        );

        // 序列化数据
        let json_body = serde_json::to_string(data)
            .map_err(|e| anyhow::anyhow!("序列化数据失败: {}", e))?;

        debug!("HTTP POST 请求 URL: {}", url);
        debug!("请求体大小: {} bytes", json_body.len());

        // 使用 reqwest 发送 HTTP POST 请求
        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .body(json_body)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("HTTP 请求失败: {}", e))?;

        // 检查响应状态
        let status = response.status();
        if status.is_success() {
            info!("✅ HTTP 写入成功: {} 行数据", data.len());
            Ok(())
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "无法读取错误响应".to_string());
            Err(anyhow::anyhow!(
                "HTTP 写入失败: 状态码 {}, 错误: {}",
                status,
                error_text
            ))
        }
    }

    /// 测试连接
    pub async fn test_connection(&self) -> Result<()> {
        info!("测试 ClickHouse HTTP 连接...");

        let url = format!("{}/?query=SELECT+1", self.config.base_url);
        let client = reqwest::Client::new();

        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("HTTP 连接测试失败: {}", e))?;

        if response.status().is_success() {
            info!("✅ ClickHouse HTTP 连接测试成功");
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "ClickHouse HTTP 连接测试失败: 状态码 {}",
                response.status()
            ))
        }
    }
}
