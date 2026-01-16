use crate::types::{KlineData, KlinePeriod, StockInfo};
use anyhow::Result;
use chrono::{NaiveDate, NaiveTime, Utc};
use clickhouse::Client;
use rustdx_complete::tcp::stock::Kline;
use rustdx_complete::tcp::{Tcp, Tdx};
use tracing::{debug, info};

/// K线数据修正器（收盘后修正当天数据）
pub struct KlineCorrector {
    ch_client: Client,
    correction_time: NaiveTime,
    max_concurrent: usize,
}

impl KlineCorrector {
    /// 创建新的修正器
    pub fn new(ch_client: Client, correction_time: &str, max_concurrent: usize) -> Result<Self> {
        let correction_time = NaiveTime::parse_from_str(correction_time, "%H:%M")?;
        Ok(Self {
            ch_client,
            correction_time,
            max_concurrent,
        })
    }

    /// 启动定时修正任务
    pub async fn start(&self) -> Result<()> {
        info!("启动K线数据修正服务，修正时间: {}", self.correction_time);

        // TODO: 实现定时任务逻辑
        // 下一步任务实现
        info!("K线数据修正服务已启动（待实现定时任务）");

        Ok(())
    }

    /// 修正指定日期的K线数据
    async fn correct_date(
        &self,
        date: NaiveDate,
        _stock_batches: &[Vec<StockInfo>],
    ) -> Result<CorrectionReport> {
        // TODO: 实现修正逻辑
        // 下一步任务实现

        Ok(CorrectionReport {
            date,
            total_klines: 0,
            corrected_klines: 0,
            correction_rate: 0.0,
        })
    }

    /// 查询当日需要修正的K线数据
    async fn fetch_realtime_klines(&self, _date: NaiveDate) -> Result<Vec<KlineData>> {
        // TODO: 从ClickHouse查询source='realtime'的K线
        // 下一步任务实现
        Ok(Vec::new())
    }

    /// 从通达信获取官方K线数据
    async fn fetch_official_klines(
        &self,
        stock: &StockInfo,
        period: KlinePeriod,
        date: NaiveDate,
    ) -> Result<Vec<KlineData>> {
        let mut tcp = Tcp::new()?;
        let market = stock.market as u16;
        let code = &stock.code;

        let kline_period = match period {
            KlinePeriod::OneMinute => 7,
            KlinePeriod::FiveMinutes => 8,
        };

        let mut kline_req = Kline::new(market, code, kline_period, 0, 800);

        match kline_req.recv_parsed(&mut tcp) {
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
                            source: "corrected".to_string(),
                        })
                    })
                    .collect();

                debug!("从通达信获取到 {} 条官方K线", klines.len());
                Ok(klines)
            }
            Err(e) => Err(anyhow::anyhow!("获取官方K线失败: {}", e)),
        }
    }

    /// 对比并修正K线数据
    fn compare_and_correct(&self, realtime: &KlineData, official: &KlineData) -> Option<KlineData> {
        // 异常判定标准
        let price_diff = (realtime.close - official.close).abs() / official.close;
        let volume_diff = (realtime.volume - official.volume).abs() / official.volume;

        // 价格偏差 > 0.01% 或 成交量偏差 > 1%
        if price_diff > 0.0001 || volume_diff > 0.01 {
            debug!(
                "发现异常K线 {} {} {}: 价格偏差 {:.4}%, 成交量偏差 {:.2}%",
                realtime.code,
                realtime.period.as_str(),
                realtime.timestamp.format("%Y-%m-%d %H:%M"),
                price_diff * 100.0,
                volume_diff * 100.0
            );

            Some(KlineData {
                source: "corrected".to_string(),
                ..official.clone()
            })
        } else {
            None
        }
    }
}

/// 修正报告
#[derive(Debug)]
pub struct CorrectionReport {
    pub date: NaiveDate,
    pub total_klines: usize,
    pub corrected_klines: usize,
    pub correction_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_corrector_new() {
        let ch_client = Client::default().with_url("http://localhost:8123");
        let corrector = KlineCorrector::new(ch_client, "15:30", 3);
        assert!(corrector.is_ok());

        let corrector = corrector.unwrap();
        assert_eq!(
            corrector.correction_time,
            NaiveTime::from_hms_opt(15, 30, 0).unwrap()
        );
        assert_eq!(corrector.max_concurrent, 3);
    }

    #[test]
    fn test_corrector_invalid_time() {
        let ch_client = Client::default().with_url("http://localhost:8123");
        let corrector = KlineCorrector::new(ch_client, "invalid", 3);
        assert!(corrector.is_err());
    }

    #[test]
    fn test_correction_report() {
        let report = CorrectionReport {
            date: Utc::now().date_naive(),
            total_klines: 1000,
            corrected_klines: 10,
            correction_rate: 0.01,
        };

        assert_eq!(report.total_klines, 1000);
        assert_eq!(report.corrected_klines, 10);
        assert_eq!(report.correction_rate, 0.01);
    }
}
