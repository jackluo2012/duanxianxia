use crate::config::{AppConfig, ClickHouseConfig};
use crate::models::*;
use anyhow::{Result, Context};
use clickhouse::Client;
use chrono::{Date, DateTime, Utc};
use std::collections::HashMap;

pub struct DataLoader {
    client: Client,
}

impl DataLoader {
    pub fn new(config: &ClickHouseConfig) -> Result<Self> {
        let mut client = Client::default();

        client
            .with_url(&config.url)
            .with_database(&config.database)
            .with_user(&config.user);

        if let Some(password) = &config.password {
            client = client.with_password(password);
        }

        Ok(Self { client })
    }

    /// 加载指定日期的所有股票行情数据
    ///
    /// # 注意
    /// 从stock_realtime_quotes表获取,聚合取每个股票的当日最后一条记录
    pub async fn load_day_quotes(&self, date: Date) -> Result<Vec<StockQuote>> {
        let mut cursor = self
            .client
            .query("SELECT
                argMax(code, timestamp) as code,
                argMax(name, timestamp) as name,
                argMax(open, timestamp) as open,
                argMax(high, timestamp) as high,
                argMax(low, timestamp) as low,
                argMax(price, timestamp) as close,
                argMax(preclose, timestamp) as pre_close,
                argMax(volume, timestamp) as volume,
                argMax(amount, timestamp) as amount,
                argMax(change_percent, timestamp) as change_percent
            FROM stock_realtime_quotes
            WHERE toDate(toDateTime(timestamp, 'Asia/Shanghai')) = ?
            GROUP BY code
            ORDER BY code
        ")
        .fetch::<StockQuoteRow>()
        .await
            .context("Failed to load day quotes")?;

        let mut quotes = Vec::new();
        while let Some(row) = cursor.next().await? {
            quotes.push(self.row_to_quote(row, date)?);
        }

        Ok(quotes)
    }

    /// 加载单只股票的3秒级行情(用于精确计算开板次数)
    pub async fn load_tick_data(&self, code: &str, date: Date) -> Result<Vec<Tick>> {
        let date_str = date.format("%Y-%m-%d").to_string();
        let start_time = format!("{} 09:30:00", date_str);
        let end_time = format!("{} 15:00:00", date_str);

        let sql = format!("SELECT
                timestamp,
                code,
                price,
                volume,
                amount
            FROM stock_realtime_quotes
            WHERE code = '{}'
              AND toDate(toDateTime(timestamp, 'Asia/Shanghai')) = '{}'
              AND toDateTime(timestamp, 'Asia/Shanghai') >= toDateTime('{}', 'Asia/Shanghai')
              AND toDateTime(timestamp, 'Asia/Shanghai') < toDateTime('{}', 'Asia/Shanghai')
            ORDER BY timestamp", code, date_str, start_time, end_time);

        let mut cursor = self
            .client
            .query(&sql)
            .fetch::<TickRow>()
            .await
            .context("Failed to load tick data")?;

        let mut ticks = Vec::new();
        while let Some(row) = cursor.next().await? {
            ticks.push(Tick {
                datetime: DateTime::from_timestamp(row.timestamp as i64, 0).unwrap(),
                code: row.code,
                price: row.price,
                volume: row.volume as u32,
                amount: row.amount,
                bid1: 0.0,    // 当前表没有盘口数据
                bid1_vol: 0,
                ask1: 0.0,
                ask1_vol: 0,
            });
        }

        Ok(ticks)
    }

    /// 获取前收盘价
    pub async fn get_prev_close(&self, code: &str, date: Date) -> Result<f64> {
        let prev_date = self.prev_trading_day(date).await?;

        let result = self
            .client
            .query("SELECT argMax(preclose, timestamp) as close FROM stock_realtime_quotes
                    WHERE code = ?
                      AND toDate(toDateTime(timestamp, 'Asia/Shanghai')) = ?
                    LIMIT 1
        ")
        .fetch_optional::<PrevCloseRow>()
        .await
            .context("Failed to get prev close")?;

        match result {
            Some(row) => Ok(row.close),
            None => {
                tracing::warn!("未找到股票 {} 在 {} 的前收盘价", code, prev_date);
                Ok(0.0)
            }
        }
    }

    /// 获取股票基本信息
    pub async fn get_stock_info(&self, code: &str) -> Result<StockInfo> {
        let result = self
            .client
            .query("SELECT code, name, market FROM stock_list WHERE code = ? LIMIT 1")
            .fetch_optional::<StockInfoRow>()
            .await
            .context("Failed to get stock info")?;

        match result {
            Some(row) => Ok(StockInfo {
                code: row.code,
                name: row.name,
                market: row.market,
                industry: Some("未分类").to_string(), // TODO: 从行业分类表获取
            }),
            None => {
                tracing::warn!("未找到股票 {} 的基本信息", code);
                Ok(StockInfo {
                    code: code.to_string(),
                    name: "未知".to_string(),
                    market: 0,
                    industry: Some("未分类").to_string(),
                })
            }
        }
    }

    /// 获取近60日最高价(用于is_new_high判断)
    pub async fn get_60d_high(&self, code: &str, date: Date) -> Result<Option<f64>> {
        let result = self
            .client
            .query("SELECT MAX(high) as max_high FROM (
                SELECT argMax(high, timestamp) as high
                FROM stock_realtime_quotes
                WHERE code = ?
                  AND toDate(toDateTime(timestamp, 'Asia/Shanghai')) < ?
                GROUP BY toDate(toDateTime(timestamp, 'Asia/Shanghai'))
                ORDER BY toDate(toDateTime(timestamp, 'Asia/Shanghai')) DESC
                LIMIT 60
            ) AS recent_60d
        ")
        .fetch_optional::<MaxHighRow>()
        .await
            .context("Failed to get 60d high")?;

        Ok(result.and_then(|r| Some(r.max_high)))
    }

    /// 获取前一交易日(简化版,实际应使用trading-calendar)
    async fn prev_trading_day(&self, date: Date) -> Result<Date> {
        use chrono::Duration;
        let mut prev = date - Duration::days(1);

        // 简单跳过周末
        while prev.weekday().num_days_from_monday() >= 5 {
            prev = prev - Duration::days(1);
        }

        Ok(prev)
    }

    /// 将数据库行转换为StockQuote
    fn row_to_quote(&self, row: StockQuoteRow, date: Date) -> Result<StockQuote> {
        Ok(StockQuote {
            code: row.code,
            name: row.name,
            date,
            datetime: Utc::now(), // 没有精确时间
            open: row.open,
            high: row.high,
            low: row.low,
            close: row.close,
            pre_close: row.pre_close,
            volume: row.volume as u64,
            amount: row.amount,
            turnover_rate: 0.0, // TODO: 计算换手率
            buy1_price: 0.0,    // 当前表没有盘口数据
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
            change_percent: row.change_percent,
        })
    }
}

// ClickHouse查询结果行结构
#[derive(Debug, clickhouse::Row)]
struct StockQuoteRow {
    code: String,
    name: String,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    pre_close: f64,
    volume: f64,
    amount: f64,
    change_percent: f64,
}

#[derive(Debug, clickhouse::Row)]
struct TickRow {
    timestamp: u64,
    code: String,
    price: f64,
    volume: f64,
    amount: f64,
}

#[derive(Debug, clickhouse::Row)]
struct PrevCloseRow {
    close: f64,
}

#[derive(Debug, clickhouse::Row)]
struct StockInfoRow {
    code: String,
    name: String,
    market: u8,
}

#[derive(Debug, clickhouse::Row)]
struct MaxHighRow {
    max_high: f64,
}
