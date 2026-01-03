// review_collector.rs - 涨停复盘数据采集模块
//
// 功能:
// 1. 实时监控涨停事件
// 2. 追踪连板高度和历史
// 3. 统计板块强度
// 4. 检测炸板情况
// 5. 定时写入ClickHouse

use crate::types::{
    ConsecutiveBoardHistory, ConsecutiveRecord, DailyLimitUpSummary, LimitType, LimitUpEvent,
    SectorDailyStrength, SectorStats, StockQuote,
};
use anyhow::Result;
use chrono::{Datelike, Duration, Local, NaiveDate, NaiveTime, Timelike, Utc};
use clickhouse::Client;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

/// ReviewCollector - 涨停复盘采集器
pub struct ReviewCollector {
    /// ClickHouse客户端
    ch_client: Client,

    /// 连板记录 (code -> record)
    consecutive_records: Arc<Mutex<HashMap<String, ConsecutiveRecord>>>,

    /// 涨停事件列表 (当日)
    limit_up_events: Arc<Mutex<Vec<LimitUpEvent>>>,

    /// 炸板事件列表 (当日)
    broken_events: Arc<Mutex<Vec<LimitUpEvent>>>,

    /// 板块统计 (sector_name -> stats)
    sector_stats: Arc<Mutex<HashMap<String, SectorStats>>>,

    /// 当前日期
    current_date: Arc<Mutex<NaiveDate>>,

    /// 炸板阈值（涨停价回落超过此比例算炸板）
    broken_threshold: f64,
}

impl ReviewCollector {
    /// 创建新的采集器
    pub fn new(ch_client: Client) -> Self {
        Self {
            ch_client,
            consecutive_records: Arc::new(Mutex::new(HashMap::new())),
            limit_up_events: Arc::new(Mutex::new(Vec::new())),
            broken_events: Arc::new(Mutex::new(Vec::new())),
            sector_stats: Arc::new(Mutex::new(HashMap::new())),
            current_date: Arc::new(Mutex::new(Local::now().naive_local().date())),
            broken_threshold: 0.02, // 2%回落算炸板
        }
    }

    /// 处理实时行情，检测涨停
    pub async fn process_quote(&self, quote: &StockQuote) -> Result<()> {
        // 检查日期切换
        self.check_date_change().await?;

        let current_time = chrono::DateTime::from_timestamp(quote.timestamp, 0)
            .unwrap_or_else(|| chrono::Utc::now())
            .with_timezone(&Local)
            .naive_local();

        // 检测涨停
        if let Some(limit_type) = self.detect_limit_up(quote) {
            let limit_event = self
                .create_limit_up_event(quote, limit_type, current_time)
                .await?;

            // 记录涨停事件
            self.record_limit_up(limit_event.clone()).await?;

            // 更新连板记录
            self.update_consecutive_record(&limit_event).await?;

            // 更新板块统计
            self.update_sector_stats(&limit_event).await?;

            info!(
                "检测到涨停: {} ({}) - 类型: {:?}, 时间: {}, 封单: {:.2}万",
                quote.name,
                quote.code,
                limit_type,
                current_time.format("%H:%M:%S"),
                limit_event.sealed_amount / 10000.0
            );
        } else {
            // 检查是否炸板
            self.check_broken_board(quote, current_time).await?;
        }

        Ok(())
    }

    /// 检测涨停
    fn detect_limit_up(&self, quote: &StockQuote) -> Option<LimitType> {
        let limit_price = self.calculate_limit_price(quote.preclose);
        let current_price = quote.price;
        let eps = limit_price * 0.0001; // 浮点数容差

        // 检查是否涨停（价格接近涨停价）
        if (current_price - limit_price).abs() < eps {
            // 判断涨停类型
            if quote.open >= limit_price - eps {
                // 开盘即涨停 -> 一字板
                Some(LimitType::Straight)
            } else if quote.low < limit_price - eps {
                // 曾低于涨停价 -> 自然板
                Some(LimitType::Natural)
            } else {
                // 开盘未涨停，但从未跌破涨停价 -> T字板
                Some(LimitType::T)
            }
        } else {
            None
        }
    }

    /// 计算涨停价
    fn calculate_limit_price(&self, preclose: f64) -> f64 {
        // A股主板涨跌停限制: 10% (ST股票5%, 科创板/创业板20%)
        // 这里简化处理，统一按10%计算
        let limit_ratio = 0.10;
        (preclose * (1.0 + limit_ratio) * 100.0).round() / 100.0
    }

    /// 创建涨停事件
    async fn create_limit_up_event(
        &self,
        quote: &StockQuote,
        limit_type: LimitType,
        limit_time: chrono::NaiveDateTime,
    ) -> Result<LimitUpEvent> {
        let limit_price = self.calculate_limit_price(quote.preclose);

        // 获取板块信息（暂时使用默认值）
        let sector_name = "未知板块".to_string();

        // 判断是否首板
        let is_first_board = {
            let records = self.consecutive_records.lock().await;
            !records.contains_key(&quote.code)
        };

        Ok(LimitUpEvent {
            code: quote.code.clone(),
            name: quote.name.clone(),
            limit_time: limit_time.and_utc(),
            limit_type,
            open_price: quote.open,
            limit_price,
            sealed_amount: 0.0, // 需要从Level-2数据获取，暂时设为0
            sealed_volume: 0.0,
            buy1_volume: 0.0,
            volume: quote.volume,
            amount: quote.amount,
            turnover_rate: 0.0, // 需要流通股本数据
            sector_name,
            is_first_board,
            preclose: quote.preclose,
        })
    }

    /// 记录涨停事件
    async fn record_limit_up(&self, event: LimitUpEvent) -> Result<()> {
        let mut events = self.limit_up_events.lock().await;
        events.push(event);
        Ok(())
    }

    /// 更新连板记录
    async fn update_consecutive_record(&self, event: &LimitUpEvent) -> Result<()> {
        let mut records = self.consecutive_records.lock().await;
        let current_date = *self.current_date.lock().await;

        if let Some(record) = records.get_mut(&event.code) {
            // 已有记录，更新连板信息
            record.consecutive_days += 1;
            record.last_limit_date = current_date;
            record.last_limit_time = event.limit_time;
            record.limit_events.push(event.clone());
        } else {
            // 新连板记录
            let record = ConsecutiveRecord {
                code: event.code.clone(),
                name: event.name.clone(),
                consecutive_days: 1,
                start_date: current_date,
                last_limit_date: current_date,
                last_limit_time: event.limit_time,
                is_active: true,
                limit_events: vec![event.clone()],
            };
            records.insert(event.code.clone(), record);
        }

        Ok(())
    }

    /// 更新板块统计
    async fn update_sector_stats(&self, event: &LimitUpEvent) -> Result<()> {
        let mut stats_map = self.sector_stats.lock().await;

        if let Some(stats) = stats_map.get_mut(&event.sector_name) {
            // 更新现有板块统计
            stats.limit_up_count += 1;
            stats.total_amount += event.amount;
            stats.total_volume += event.volume;
            stats.limit_up_stocks.push(event.code.clone());
        } else {
            // 创建新板块统计
            let stats = SectorStats {
                sector_code: event.sector_name.clone(),
                sector_name: event.sector_name.clone(),
                limit_up_count: 1,
                total_stocks: 0, // 需要从股票列表获取
                total_amount: event.amount,
                total_volume: event.volume,
                avg_change_percent: 0.0,
                max_change_percent: 0.0,
                min_change_percent: 0.0,
                net_inflow: 0.0,
                consecutive_score: 0.0,
                limit_up_stocks: vec![event.code.clone()],
            };
            stats_map.insert(event.sector_name.clone(), stats);
        }

        Ok(())
    }

    /// 检查炸板
    async fn check_broken_board(
        &self,
        quote: &StockQuote,
        current_time: chrono::NaiveDateTime,
    ) -> Result<()> {
        let events = self.limit_up_events.lock().await;

        // 查找该股票是否已有涨停记录
        if let Some(event) = events.iter().find(|e| e.code == quote.code) {
            let limit_price = event.limit_price;
            let current_price = quote.price;
            let drop_ratio = (limit_price - current_price) / limit_price;

            // 如果回落超过阈值，记录为炸板
            if drop_ratio > self.broken_threshold {
                drop(events); // 释放锁

                let mut broken_events = self.broken_events.lock().await;
                // 移除涨停记录，添加到炸板列表
                let mut all_events = self.limit_up_events.lock().await;
                if let Some(pos) = all_events.iter().position(|e| e.code == quote.code) {
                    let broken_event = all_events.remove(pos);
                    broken_events.push(broken_event.clone());

                    warn!(
                        "检测到炸板: {} ({}) - 回落: {:.2}%",
                        quote.name,
                        quote.code,
                        drop_ratio * 100.0
                    );
                }
            }
        }

        Ok(())
    }

    /// 检查日期切换
    async fn check_date_change(&self) -> Result<()> {
        let today = Local::now().naive_local().date();
        let mut current_date = self.current_date.lock().await;

        if *current_date != today {
            info!(
                "日期切换: {} -> {}，准备前一日数据汇总",
                current_date, today
            );

            // 执行前一日数据汇总
            drop(current_date); // 释放锁
            self.daily_summary().await?;

            // 更新当前日期
            let mut current_date = self.current_date.lock().await;
            *current_date = today;

            // 清空当日数据
            self.limit_up_events.lock().await.clear();
            self.broken_events.lock().await.clear();
            self.sector_stats.lock().await.clear();
        }

        Ok(())
    }

    /// 每日数据汇总
    async fn daily_summary(&self) -> Result<()> {
        let current_date = *self.current_date.lock().await;
        info!("开始执行每日涨停汇总: {}", current_date);

        // 获取当日统计数据
        let limit_up_events = self.limit_up_events.lock().await.clone();
        let broken_events = self.broken_events.lock().await.clone();
        let sector_stats = self.sector_stats.lock().await.clone();
        let consecutive_records = self.consecutive_records.lock().await.clone();

        // 1. 生成每日涨停汇总
        let daily_summary =
            self.generate_daily_summary(current_date, &limit_up_events, &broken_events)?;

        // 2. 生成连板历史记录
        let consecutive_history =
            self.generate_consecutive_history(current_date, &consecutive_records)?;

        // 3. 生成板块强度记录
        let sector_strength =
            self.generate_sector_strength(current_date, &sector_stats, &consecutive_records)?;

        // 4. 写入ClickHouse
        self.write_daily_summary(&daily_summary).await?;
        self.write_consecutive_history(&consecutive_history).await?;
        self.write_sector_strength(&sector_strength).await?;

        info!(
            "每日涨停汇总完成: 总计{}只涨停，{}只炸板，{}只连板",
            daily_summary.total_count,
            daily_summary.broken_count,
            consecutive_history.len()
        );

        Ok(())
    }

    /// 生成每日涨停汇总
    fn generate_daily_summary(
        &self,
        date: NaiveDate,
        limit_up_events: &[LimitUpEvent],
        broken_events: &[LimitUpEvent],
    ) -> Result<DailyLimitUpSummary> {
        let total_count = limit_up_events.len() as u32;

        // 统计首板数量
        let first_board = limit_up_events.iter().filter(|e| e.is_first_board).count() as u32;

        // 统计各时段涨停（简化处理）
        let auction_limit = 0; // 竞价涨停（9:25之前）
        let morning_limit = limit_up_events
            .iter()
            .filter(|e| e.limit_time.hour() >= 9 && e.limit_time.hour() < 13)
            .count() as u32;
        let afternoon_limit = limit_up_events
            .iter()
            .filter(|e| e.limit_time.hour() >= 13)
            .count() as u32;

        // 统计涨停类型
        let straight_limit = limit_up_events
            .iter()
            .filter(|e| e.limit_type == LimitType::Straight)
            .count() as u32;
        let t_limit = limit_up_events
            .iter()
            .filter(|e| e.limit_type == LimitType::T)
            .count() as u32;
        let natural_limit = limit_up_events
            .iter()
            .filter(|e| e.limit_type == LimitType::Natural)
            .count() as u32;

        // 炸板统计
        let broken_count = broken_events.len() as u32;
        let broken_rate = if total_count > 0 {
            (broken_count as f32) / (total_count as f32)
        } else {
            0.0
        };

        // 计算市场情绪指数
        let market_sentiment_index = Self::calculate_market_sentiment(total_count);

        Ok(DailyLimitUpSummary {
            date,
            total_count,
            first_board,
            auction_limit,
            morning_limit,
            afternoon_limit,
            straight_limit,
            t_limit,
            natural_limit,
            broken_count,
            broken_rate,
            market_sentiment_index,
        })
    }

    /// 计算市场情绪指数 (0-100)
    fn calculate_market_sentiment(total_count: u32) -> f32 {
        const BASE: f64 = 5000.0; // 全市场股票数
        let ratio = total_count as f64 / BASE;

        // 使用对数函数将涨停数量映射到0-100
        let score = (ratio.ln() * 25.0) as f32;
        score.min(100.0).max(0.0)
    }

    /// 生成连板历史记录
    fn generate_consecutive_history(
        &self,
        date: NaiveDate,
        consecutive_records: &HashMap<String, ConsecutiveRecord>,
    ) -> Result<Vec<ConsecutiveBoardHistory>> {
        let mut history = Vec::new();

        for record in consecutive_records.values() {
            if record.is_active && record.last_limit_date == date {
                // 获取最后涨停事件
                if let Some(last_event) = record.limit_events.last() {
                    let history_item = ConsecutiveBoardHistory {
                        date,
                        code: record.code.clone(),
                        name: record.name.clone(),
                        consecutive_days: record.consecutive_days,
                        start_date: record.start_date,
                        end_date: None, // 仍在连板中
                        is_active: 1,
                        limit_time: last_event.limit_time,
                        limit_type: last_event.limit_type.as_str().to_string(),
                        open_price: last_event.open_price,
                        limit_price: last_event.limit_price,
                        sealed_amount: last_event.sealed_amount,
                        sealed_volume: last_event.sealed_volume,
                        buy1_volume: last_event.buy1_volume as u32,
                        volume: last_event.volume,
                        amount: last_event.amount,
                        turnover_rate: last_event.turnover_rate,
                        sector_name: last_event.sector_name.clone(),
                    };
                    history.push(history_item);
                }
            }
        }

        Ok(history)
    }

    /// 生成板块强度记录
    fn generate_sector_strength(
        &self,
        date: NaiveDate,
        sector_stats: &HashMap<String, SectorStats>,
        consecutive_records: &HashMap<String, ConsecutiveRecord>,
    ) -> Result<Vec<SectorDailyStrength>> {
        let mut strength_list = Vec::new();

        for stats in sector_stats.values() {
            // 计算连板加权评分
            let mut consecutive_score = 0.0;
            for code in &stats.limit_up_stocks {
                if let Some(record) = consecutive_records.get(code) {
                    // 连板天数 × 封单金额（亿）作为评分
                    let sealed_amount_yi = record
                        .limit_events
                        .last()
                        .map(|e| e.sealed_amount / 100_000_000.0)
                        .unwrap_or(0.0);
                    consecutive_score += (record.consecutive_days as f64) * sealed_amount_yi;
                }
            }

            // 计算涨停股比例
            let limit_up_ratio = if stats.total_stocks > 0 {
                (stats.limit_up_count as f32) / (stats.total_stocks as f32)
            } else {
                0.0
            };

            // 计算强度评分（综合得分）
            let strength_score = (stats.limit_up_count as f64) * 10.0
                + consecutive_score
                + (stats.total_amount / 100_000_000.0); // 成交额（亿）

            let strength_item = SectorDailyStrength {
                date,
                sector_code: stats.sector_code.clone(),
                sector_name: stats.sector_name.clone(),
                limit_up_count: stats.limit_up_count,
                limit_up_ratio,
                consecutive_score,
                avg_change_percent: stats.avg_change_percent,
                max_change_percent: stats.max_change_percent,
                min_change_percent: stats.min_change_percent,
                total_amount: stats.total_amount,
                total_volume: stats.total_volume,
                avg_turnover_rate: 0.0, // 需要流通股本数据
                net_inflow: stats.net_inflow,
                net_inflow_ratio: 0.0,
                strength_rank: 0, // 后续排序后更新
                strength_score,
                trend_3d: 0.0, // 需要3日历史数据
                trend_5d: 0.0, // 需要5日历史数据
            };

            strength_list.push(strength_item);
        }

        // 按强度评分排序并更新排名
        strength_list.sort_by(|a, b| {
            b.strength_score
                .partial_cmp(&a.strength_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        for (index, item) in strength_list.iter_mut().enumerate() {
            item.strength_rank = (index + 1) as u32;
        }

        Ok(strength_list)
    }

    /// 写入每日涨停汇总到ClickHouse
    async fn write_daily_summary(&self, summary: &DailyLimitUpSummary) -> Result<()> {
        let mut insert = self.ch_client.insert("daily_limit_up_summary")?;

        insert.write(summary).await?;
        insert.end().await?;

        debug!(
            "写入每日涨停汇总: {} - 总计: {}只",
            summary.date, summary.total_count
        );

        Ok(())
    }

    /// 写入连板历史到ClickHouse
    async fn write_consecutive_history(&self, history: &[ConsecutiveBoardHistory]) -> Result<()> {
        if history.is_empty() {
            return Ok(());
        }

        let mut insert = self.ch_client.insert("consecutive_boards_history")?;

        for item in history {
            insert.write(item).await?;
        }

        insert.end().await?;

        debug!("写入连板历史: {}条记录", history.len());

        Ok(())
    }

    /// 写入板块强度到ClickHouse
    async fn write_sector_strength(&self, strength: &[SectorDailyStrength]) -> Result<()> {
        if strength.is_empty() {
            return Ok(());
        }

        let mut insert = self.ch_client.insert("sector_daily_strength")?;

        for item in strength {
            insert.write(item).await?;
        }

        insert.end().await?;

        debug!("写入板块强度: {}条记录", strength.len());

        Ok(())
    }

    /// 获取当日连板排行榜
    pub async fn get_consecutive_ranking(&self) -> Vec<ConsecutiveRecord> {
        let records = self.consecutive_records.lock().await;
        let mut ranking: Vec<_> = records.values().cloned().collect();

        // 按连板天数降序排序
        ranking.sort_by(|a, b| {
            b.consecutive_days
                .cmp(&a.consecutive_days)
                .then_with(|| b.last_limit_time.cmp(&a.last_limit_time))
        });

        ranking
    }

    /// 获取当日板块强度排行
    pub async fn get_sector_ranking(&self) -> Vec<SectorStats> {
        let stats_map = self.sector_stats.lock().await;
        let mut ranking: Vec<_> = stats_map.values().cloned().collect();

        // 按涨停股数量降序排序
        ranking.sort_by(|a, b| {
            b.limit_up_count.cmp(&a.limit_up_count).then_with(|| {
                b.total_amount
                    .partial_cmp(&a.total_amount)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        });

        ranking
    }
}
