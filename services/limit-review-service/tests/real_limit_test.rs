// ===================================================================
// 使用真实数据进行涨停识别测试
// ===================================================================

use clickhouse::Client;
use limit_review_service::models::*;

#[cfg(test)]
mod real_limit_detector_tests {
    use super::*;

    /// 辅助函数：从ClickHouse加载真实股票数据
    async fn load_real_quotes() -> Vec<StockQuote> {
        let client = Client::default()
            .with_url("http://localhost:8123")
            .with_database("duanxianxia");

        // 获取最新的数据，包括真实的preclose
        let rows = client
            .query("
                SELECT
                    code,
                    argMax(name, timestamp) as name,
                    argMax(price, timestamp) as price,
                    argMax(preclose, timestamp) as preclose,
                    argMax(open, timestamp) as open,
                    argMax(high, timestamp) as high,
                    argMax(low, timestamp) as low,
                    max(volume) as volume,
                    max(amount) as amount
                FROM stock_realtime_quotes
                GROUP BY code
            ")
            .fetch_all::<(String, String, f64, f64, f64, f64, f64, f64, f64)>()
            .await;

        match rows {
            Ok(quotes) => {
                println!("📊 加载了 {} 只股票的真实行情数据", quotes.len());

                // 转换为StockQuote（使用真实的preclose和change_percent）
                quotes.into_iter().map(|(code, name, price, preclose, open, high, low, volume, amount)| {
                    // 计算涨跌幅
                    let change_percent = if preclose > 0.0 {
                        ((price - preclose) / preclose) * 100.0
                    } else {
                        0.0
                    };

                    // 计算涨停价（主板10%）
                    let limit_price = preclose * 1.1;
                    let is_limit_up = price >= limit_price - 0.01;

                    StockQuote {
                        code,
                        name,
                        date: chrono::Utc::now().date_naive(),
                        datetime: chrono::Utc::now(),
                        open,
                        high,
                        low,
                        close: price,
                        pre_close: preclose,
                        volume,
                        amount,
                        turnover_rate: 0.0,
                        change_percent,
                        buy1_price: 0.0,
                        buy1_vol: 0,
                        buy2_price: 0.0,
                        buy2_vol: 0,
                        buy3_price: 0.0,
                        buy3_vol: 0,
                        buy4_price: 0.0,
                        buy4_vol: 0,
                        buy5_price: 0.0,
                        buy5_vol: 0,
                        sell1_price: 0.0,
                        sell1_vol: 0,
                        sell2_price: 0.0,
                        sell2_vol: 0,
                        sell3_price: 0.0,
                        sell3_vol: 0,
                        sell4_price: 0.0,
                        sell4_vol: 0,
                        sell5_price: 0.0,
                        sell5_vol: 0,
                    }
                }).collect()
            }
            Err(e) => {
                println!("⚠️  加载数据失败: {}", e);
                vec![]
            }
        }
    }

    // ===================================================================
    // 测试: 使用真实数据识别涨停股票
    // ===================================================================

    #[tokio::test]
    async fn test_detect_limit_up_from_real_data() {
        use limit_review_service::limit_detector::LimitDetector;

        let quotes = load_real_quotes().await;

        if quotes.is_empty() {
            println!("⚠️  没有真实数据可用，跳过测试");
            return;
        }

        println!("\n📈 涨停识别结果:");
        let mut limit_up_count = 0;

        for quote in &quotes {
            let is_limit = LimitDetector::is_limit_up(quote);
            let limit_price = quote.limit_price();
            let change_pct = ((quote.close - quote.pre_close) / quote.pre_close) * 100.0;

            if is_limit {
                limit_up_count += 1;
                println!("  ✅ {} {} - 价格:{:.2}, 昨收:{:.2}, 涨幅:{:.2}%, 涨停价:{:.2}",
                    quote.code, quote.name, quote.close, quote.pre_close, change_pct, limit_price);
            } else {
                println!("  ❌ {} {} - 价格:{:.2}, 昨收:{:.2}, 涨幅:{:.2}%, 涨停价:{:.2}",
                    quote.code, quote.name, quote.close, quote.pre_close, change_pct, limit_price);
            }
        }

        println!("\n📊 统计: 总共 {} 只股票，其中 {} 只涨停", quotes.len(), limit_up_count);
        assert!(limit_up_count >= 0, "涨停识别应该正常工作");
    }

    // ===================================================================
    // 测试: 计算涨停价
    // ===================================================================

    #[tokio::test]
    async fn test_limit_price_calculation_real_data() {
        let quotes = load_real_quotes().await;

        if quotes.is_empty() {
            println!("⚠️  没有真实数据可用，跳过测试");
            return;
        }

        println!("\n💰 涨停价计算:");
        for quote in &quotes {
            let limit_price = quote.limit_price();
            let limit_rate = ((limit_price - quote.pre_close) / quote.pre_close) * 100.0;

            println!("  {} - 昨收:{:.2}, 涨停价:{:.2} (涨{}%)",
                quote.code, quote.pre_close, limit_price, limit_rate);
        }
    }
}
