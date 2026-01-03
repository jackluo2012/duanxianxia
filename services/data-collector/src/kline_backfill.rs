use crate::types::{KlineData, KlinePeriod, StockInfo};
use anyhow::Result;
use chrono::{DateTime, Duration, NaiveDate, Utc};
use clickhouse::Client;
use rustdx_complete::tcp::stock::Kline;
use rustdx_complete::tcp::{Tcp, Tdx};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{debug, info, warn};

/// K线历史回填管理器
pub struct KlineBackfill {
    ch_client: Client,
    max_concurrent: usize,
    batch_size: usize,
    timeout_seconds: u64,
}

impl KlineBackfill {
    /// 创建新的回填管理器
    pub fn new(
        ch_client: Client,
        max_concurrent: usize,
        batch_size: usize,
        timeout_seconds: u64,
    ) -> Self {
        Self {
            ch_client,
            max_concurrent,
            batch_size,
            timeout_seconds,
        }
    }

    /// 为单只股票回填K线数据（静态方法版本）
    async fn backfill_stock_static(
        stock: &StockInfo,
        period: KlinePeriod,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<KlineData>> {
        let mut all_klines = Vec::new();
        let mut tcp = Tcp::new()?;
        let mut failed_days = 0usize;

        let mut current_date = start_date.date_naive();
        let end_date_naive = end_date.date_naive();
        let total_days = (end_date_naive - current_date).num_days() as usize;

        while current_date <= end_date_naive {
            match Self::fetch_daily_klines_static(&mut tcp, stock, period, current_date).await {
                Ok(klines) => {
                    all_klines.extend(klines);
                }
                Err(e) => {
                    failed_days += 1;
                    warn!(
                        "{} {} K线获取失败 ({}/{}): {}",
                        stock.code,
                        period.as_str(),
                        failed_days,
                        total_days,
                        e
                    );
                }
            }

            current_date = current_date + Duration::days(1);
        }

        if failed_days > 0 {
            warn!(
                "{} {} 回填完成，成功 {} 天，失败 {} 天",
                stock.code,
                period.as_str(),
                total_days - failed_days,
                failed_days
            );
        }

        // 如果失败率超过50%，记录ERROR
        if total_days > 0 && failed_days > total_days / 2 {
            warn!(
                "警告: {} {} 回填失败率超过50% ({}/{}",
                stock.code,
                period.as_str(),
                failed_days,
                total_days
            );
        }

        Ok(all_klines)
    }

    /// 查询已回填的日期范围
    ///
    /// TODO: 实现ClickHouse查询以获取已回填的日期范围
    /// 此功能用于增量回填，避免重复获取已有数据
    async fn get_backfilled_range(
        &self,
        _code: &str,
        _period: KlinePeriod,
    ) -> Result<Option<(DateTime<Utc>, DateTime<Utc>)>> {
        // 暂时返回None，表示未找到已回填数据
        // 后续实现：
        // 1. 使用ClickHouse Client查询最小和最大时间戳
        // 2. 返回已回填的日期范围
        Ok(None)
    }

    /// 获取单日K线数据（静态方法版本）
    async fn fetch_daily_klines_static(
        tcp: &mut Tcp,
        stock: &StockInfo,
        period: KlinePeriod,
        date: NaiveDate,
    ) -> Result<Vec<KlineData>> {
        debug!(
            "获取 {} {} K线，日期: {}",
            stock.code,
            period.as_str(),
            date
        );

        let market = stock.market as u16;
        let code = &stock.code;

        let kline_period = match period {
            KlinePeriod::OneMinute => 7,
            KlinePeriod::FiveMinutes => 8,
        };

        let mut kline_req = Kline::new(market, code, kline_period, 0, 800);

        match kline_req.recv_parsed(tcp) {
            Ok(_) => {
                let raw_klines = kline_req.result();
                let klines: Vec<KlineData> = raw_klines
                    .iter()
                    .filter_map(|k| {
                        let kline_date = chrono::NaiveDate::from_ymd_opt(
                            k.dt.year as i32,
                            k.dt.month as u32,
                            k.dt.day as u32,
                        )?;

                        if kline_date != date {
                            return None;
                        }

                        let timestamp = chrono::NaiveDateTime::new(
                            kline_date,
                            chrono::NaiveTime::from_hms_opt(
                                k.dt.hour as u32,
                                k.dt.minute as u32,
                                0,
                            )?,
                        );
                        let timestamp_utc = timestamp.and_utc();

                        Some(KlineData {
                            timestamp: timestamp_utc,
                            code: stock.code.clone(),
                            name: stock.name.clone(),
                            period,
                            open: k.open,
                            high: k.high,
                            low: k.low,
                            close: k.close,
                            volume: k.vol / 100.0,
                            amount: k.amount,
                            trade_count: 0,
                            source: "backfill".to_string(),
                        })
                    })
                    .collect();

                debug!("获取到 {} 条K线数据", klines.len());
                Ok(klines)
            }
            Err(e) => Err(anyhow::anyhow!("获取K线失败: {}", e)),
        }
    }

    /// 执行历史回填（最近3个月）
    pub async fn backfill(&self, stock_batches: &[Vec<StockInfo>]) -> Result<()> {
        info!("开始K线历史数据回填（最近3个月）");

        let end_date = Utc::now();
        let start_date = end_date - Duration::days(90);

        info!(
            "回填日期范围: {} 到 {}",
            start_date.format("%Y-%m-%d"),
            end_date.format("%Y-%m-%d")
        );

        let periods = vec![KlinePeriod::OneMinute, KlinePeriod::FiveMinutes];
        let mut total_klines = 0usize;
        let mut successful_stocks = 0usize;
        let mut failed_stocks = 0usize;

        // 并行回填所有股票和周期
        let semaphore = Arc::new(Semaphore::new(self.max_concurrent));
        let mut tasks = Vec::new();

        for batch in stock_batches {
            for stock in batch {
                for period in &periods {
                    let semaphore = semaphore.clone();
                    let stock = stock.clone();
                    let period = *period;
                    let start_date = start_date;
                    let end_date = end_date;

                    tasks.push(tokio::spawn(async move {
                        let _permit = semaphore.acquire().await.unwrap();

                        // 调用backfill_stock方法而不是内联逻辑
                        match Self::backfill_stock_static(&stock, period, start_date, end_date)
                            .await
                        {
                            Ok(klines) => {
                                if !klines.is_empty() {
                                    info!(
                                        "{} {} 回填 {} 条",
                                        stock.code,
                                        period.as_str(),
                                        klines.len()
                                    );
                                }
                                Ok((stock.code, period, klines.len()))
                            }
                            Err(e) => {
                                warn!("{} {} 回填失败: {}", stock.code, period.as_str(), e);
                                Err(anyhow::anyhow!("{} {} 失败", stock.code, period.as_str()))
                            }
                        }
                    }));
                }
            }
        }

        // 收集结果
        for task in tasks {
            match task.await {
                Ok(Ok((_code, _period, count))) => {
                    total_klines += count;
                    successful_stocks += 1;
                }
                Ok(Err(_)) => {
                    failed_stocks += 1;
                }
                Err(e) => {
                    warn!("任务执行失败: {}", e);
                    failed_stocks += 1;
                }
            }
        }

        info!(
            "K线历史数据回填完成，成功 {} 只，失败 {} 只，共 {} 条K线",
            successful_stocks, failed_stocks, total_klines
        );

        if failed_stocks > 0 {
            warn!("有 {} 只股票回填失败，建议检查日志", failed_stocks);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backfill_new() {
        let ch_client = Client::default().with_url("http://localhost:8123");
        let backfill = KlineBackfill::new(ch_client, 3, 80, 10);
        assert_eq!(backfill.max_concurrent, 3);
        assert_eq!(backfill.batch_size, 80);
        assert_eq!(backfill.timeout_seconds, 10);
    }
}
