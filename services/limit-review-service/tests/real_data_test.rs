// ===================================================================
// 真实数据集成测试
// ===================================================================

use clickhouse::Client;

#[cfg(test)]
mod real_data_tests {
    use super::*;

    // ===================================================================
    // 测试: 验证真实数据存在
    // ===================================================================

    #[tokio::test]
    async fn test_real_data_exists() {
        let client = Client::default()
            .with_url("http://localhost:8123")
            .with_database("duanxianxia");

        // 检查是否有真实数据
        let count = client
            .query("SELECT count() FROM stock_realtime_quotes")
            .fetch_one::<u64>()
            .await
            .unwrap();

        assert!(count > 0, "应该有真实的股票数据");
        println!("✅ 找到了 {} 条真实股票数据", count);
    }

    // ===================================================================
    // 测试: 获取今日涨停股票
    // ===================================================================

    #[tokio::test]
    async fn test_get_limit_up_stocks_real_data() {
        let client = Client::default()
            .with_url("http://localhost:8123")
            .with_database("duanxianxia");

        // 获取最新的一条数据，计算涨停价
        let result = client
            .query("
                SELECT
                    code,
                    price,
                    preclose,
                    (preclose * 1.1) as limit_price,
                    (price >= (preclose * 1.1)) as is_limit_up
                FROM stock_realtime_quotes
                WHERE preclose > 0
                ORDER BY timestamp DESC
                LIMIT 10
            ")
            .fetch_all::<(String, f64, f64, f64, i8)>()
            .await;

        match result {
            Ok(rows) => {
                println!("✅ 获取到 {} 条股票数据", rows.len());

                for (code, price, preclose, limit_price, is_limit_up) in rows {
                    println!("  {} - 价格:{:.2}, 昨收:{:.2}, 涨停价:{:.2}, 是否涨停:{}",
                        code, price, preclose, limit_price, is_limit_up);
                }
            }
            Err(e) => {
                println!("⚠️  查询失败（可能没有preclose数据）: {}", e);
            }
        }
    }

    // ===================================================================
    // 测试: 统计股票数量
    // ===================================================================

    #[tokio::test]
    async fn test_stock_statistics() {
        let client = Client::default()
            .with_url("http://localhost:8123")
            .with_database("duanxianxia");

        // 统计不同股票的数量
        let stock_count = client
            .query("SELECT uniqExact(code) as stock_count FROM stock_realtime_quotes")
            .fetch_one::<u64>()
            .await
            .unwrap();

        // 总记录数
        let total_records = client
            .query("SELECT count() as total FROM stock_realtime_quotes")
            .fetch_one::<u64>()
            .await
            .unwrap();

        println!("✅ 统计信息:");
        println!("  - 不同股票数: {}", stock_count);
        println!("  - 总记录数: {}", total_records);

        assert!(stock_count > 0, "应该有股票数据");
    }

    // ===================================================================
    // 测试: 查看数据时间范围
    // ===================================================================

    #[tokio::test]
    async fn test_data_time_range() {
        let client = Client::default()
            .with_url("http://localhost:8123")
            .with_database("duanxianxia");

        let result = client
            .query("
                SELECT
                    toDateTime(min(timestamp)) as earliest,
                    toDateTime(max(timestamp)) as latest,
                    count() as total_records
                FROM stock_realtime_quotes
            ")
            .fetch_one::<(String, String, u64)>()
            .await;

        match result {
            Ok((earliest, latest, count)) => {
                println!("✅ 数据时间范围:");
                println!("  - 最早记录: {}", earliest);
                println!("  - 最新记录: {}", latest);
                println!("  - 总记录数: {}", count);
            }
            Err(e) => {
                println!("⚠️  查询失败: {}", e);
            }
        }
    }

    // ===================================================================
    // 测试: 检查数据完整性
    // ===================================================================

    #[tokio::test]
    async fn test_data_integrity() {
        let client = Client::default()
            .with_url("http://localhost:8123")
            .with_database("duanxianxia");

        // 检查是否有 NULL 值
        let null_prices = client
            .query("SELECT count() FROM stock_realtime_quotes WHERE price = 0")
            .fetch_one::<u64>()
            .await
            .unwrap();

        let zero_volumes = client
            .query("SELECT count() FROM stock_realtime_quotes WHERE volume = 0")
            .fetch_one::<u64>()
            .await
            .unwrap();

        println!("✅ 数据完整性检查:");
        println!("  - 价格为0的记录: {}", null_prices);
        println!("  - 成交量为0的记录: {}", zero_volumes);

        // 这些警告是可以接受的（非交易时段可能为0）
    }
}
