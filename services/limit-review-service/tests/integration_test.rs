// ===================================================================
// 涨停复盘系统集成测试
// ===================================================================

use anyhow::Result;
use clickhouse::Client;

// 注意: 集成测试需要ClickHouse运行
// 运行测试前确保 ClickHouse 已启动并初始化

#[cfg(test)]
mod integration_tests {
    use super::*;

    // ===================================================================
    // 集成测试: 数据库连接
    // ===================================================================

    #[tokio::test]
    async fn test_clickhouse_connection() {
        // 测试ClickHouse连接
        let client = Client::default()
            .with_url("http://localhost:8123")
            .with_database("limit_review");

        // 执行简单查询测试连接 - 使用 count()
        let result = client
            .query("SELECT count() FROM limit_up_review")
            .fetch_one::<u64>()
            .await;

        assert!(result.is_ok(), "应成功连接ClickHouse");

        println!("✅ ClickHouse连接测试通过，总记录数: {}", result.unwrap());
    }

    // ===================================================================
    // 集成测试: 数据读取
    // ===================================================================

    #[tokio::test]
    async fn test_load_day_quotes() {
        let client = Client::default()
            .with_url("http://localhost:8123")
            .with_database("limit_review");

        // 读取测试数据 - 使用简单计数
        let result = client
            .query("SELECT count() FROM limit_up_review WHERE trade_date = '2026-01-13'")
            .fetch_one::<u64>()
            .await;

        assert!(result.is_ok(), "应成功读取数据");
        let count = result.unwrap();
        assert_eq!(count, 3, "应有3条记录");

        println!("✅ 数据读取测试通过，读取了{}条记录", count);
    }

    // ===================================================================
    // 集成测试: 端到端流程
    // ===================================================================

    #[tokio::test]
    #[ignore = "需要完整的数据加载器实现"]
    async fn test_end_to_end_daily_review() {
        // TODO: 实现端到端测试
        // 1. 初始化测试数据
        // 2. 运行复盘生成
        // 3. 验证结果

        println!("⚠️  端到端测试待实现");
    }

    // ===================================================================
    // 集成测试: API端点
    // ===================================================================

    #[tokio::test]
    #[ignore = "需要启动HTTP服务"]
    async fn test_api_get_daily_review() {
        // TODO: 测试API端点
        // let response = reqwest::get("http://localhost:8086/api/review/2026-01-13")
        //     .await
        //     .unwrap();
        // assert_eq!(response.status(), 200);

        println!("⚠️  API测试待实现（需要启动服务）");
    }

    // ===================================================================
    // 性能测试
    // ===================================================================

    #[tokio::test]
    #[ignore = "需要完整实现和更多测试数据"]
    async fn test_performance_1000_stocks() {
        // TODO: 测试1000只股票的处理性能
        let start = std::time::Instant::now();

        // 加载并处理1000只股票
        // let count = generator.generate_daily_review(test_date).await.unwrap();

        let duration = start.elapsed();

        println!("处理耗时: {:?}", duration);
        assert!(duration.as_secs() < 10, "1000只股票应在10秒内处理完成");

        println!("⚠️  性能测试待实现");
    }
}

// ===================================================================
// 手动测试指南
// ===================================================================

#[doc = "
## 手动测试指南

### 1. 启动ClickHouse
```bash
docker run -d \
  --name limit-review-clickhouse \
  -p 8123:8123 \
  -p 9000:9000 \
  clickhouse/clickhouse-server:24.11
```

### 2. 初始化测试数据
```bash
# 创建数据库和表
docker exec -i duanxianxia-clickhouse-1 clickhouse-client <<'SQL'
CREATE DATABASE IF NOT EXISTS limit_review;

USE limit_review;

CREATE TABLE IF NOT EXISTS limit_up_review (
    trade_date Date COMMENT '交易日',
    code String COMMENT '股票代码',
    name String COMMENT '股票名称',
    is_limit_up UInt8 DEFAULT 1 COMMENT '是否涨停',
    limit_type String COMMENT '涨停类型',
    first_limit_time DateTime COMMENT '首次涨停时间',
    last_limit_time DateTime COMMENT '最后封板时间',
    open_times UInt8 DEFAULT 0 COMMENT '开板次数',
    consecutive_days UInt8 DEFAULT 0 COMMENT '连板数',
    sealed_amount Decimal(20,2) DEFAULT 0 COMMENT '封单金额',
    created_at DateTime DEFAULT now() COMMENT '创建时间'
)
ENGINE = MergeTree()
ORDER BY (trade_date, code);
SQL

# 插入测试数据
docker exec duanxianxia-clickhouse-1 clickhouse-client --query \"
INSERT INTO limit_review.limit_up_review (trade_date, code, name, is_limit_up, limit_type, open_times, consecutive_days, sealed_amount) VALUES
    ('2026-01-13', '000001', '平安银行', 1, 'straight', 0, 3, 1000000.00),
    ('2026-01-13', '000002', '万科A', 1, 'natural', 2, 1, 500000.00),
    ('2026-01-13', '600000', '浦发银行', 1, 't', 1, 2, 750000.00);
\"
```

### 3. 运行集成测试
```bash
cd services/limit-review-service
cargo test --test integration_test
```

### 4. 运行服务
```bash
cargo run
```

### 5. 测试API
```bash
# 健康检查
curl http://localhost:8086/health

# 查询复盘数据
curl http://localhost:8086/api/review/2026-01-13
```
"]

pub struct ManualTestGuide;
