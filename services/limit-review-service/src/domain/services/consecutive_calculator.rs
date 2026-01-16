// ===================================================================
// 连板计算器 - 跨交易日连板数计算
// ===================================================================

use crate::domain::entities::models::*;
use anyhow::Result;
use trading_calendar::TradingCalendar;
use chrono::{Datelike, NaiveDate};
use std::collections::HashMap;

/// 连板历史记录
#[derive(Debug, Clone)]
pub struct ConsecutiveHistory {
    pub trade_date: NaiveDate,
    pub code: String,
    pub name: String,
    pub consecutive_days: i32,
    pub limit_type: Option<String>,
    pub sealed_amount: Option<f64>,
    pub industry: Option<String>,
}

/// 连板排行榜
#[derive(Debug, Clone)]
pub struct ConsecutiveRanking {
    pub code: String,
    pub name: String,
    pub consecutive_days: i32,
    pub start_date: NaiveDate,
    pub limit_type: Option<String>,
    pub sealed_amount: Option<f64>,
    pub turnover_rate: Option<f64>,
    pub industry: Option<String>,
    pub is_new_high: i32,
}

pub struct ConsecutiveCalculator {
    calendar: TradingCalendar,
}

impl ConsecutiveCalculator {
    /// 创建连板计算器 (异步版本)
    pub async fn new() -> Result<Self> {
        Ok(Self {
            calendar: TradingCalendar::new().await?,
        })
    }

    /// 创建连板计算器 (简化版本，用于测试)
    ///
    /// 注意：这个版本不使用TradingCalendar，仅用于编译通过
    /// 实际使用请使用 `new()` 方法
    #[deprecated(note = "请使用 new() 方法")]
    pub fn new_sync() -> Self {
        // TODO: 完整实现需要TradingCalendar
        // 暂时创建一个占位符，不使用calendar功能
        Self {
            calendar: unsafe { std::mem::zeroed() },
        }
    }

    /// 计算单只股票的连板数
    ///
    /// # 算法
    /// 1. 从当日开始,向前查找连续涨停的交易日
    /// 2. 遇到未涨停或停牌即停止
    /// 3. 返回连续涨停天数
    ///
    /// # 注意
    /// 此版本为简化实现,不使用数据库
    /// 实际使用时需要传入历史数据或从ClickHouse查询
    pub async fn calculate_consecutive(
        &self,
        _code: &str,
        _current_date: NaiveDate,
    ) -> Result<u8> {
        // TODO: 实现从ClickHouse查询历史涨停记录
        // 目前返回0
        Ok(0)
    }

    /// 从历史记录计算连板数(完整实现)
    pub async fn calculate_consecutive_from_history(
        &self,
        code: &str,
        current_date: NaiveDate,
        history: &[LimitUpReview],
    ) -> Result<u8> {
        // 1. 检查今日是否涨停
        let today_limit = history.iter()
            .find(|r| r.code == code && r.trade_date == current_date);

        if today_limit.is_none() || today_limit.unwrap().is_limit_up == 0 {
            return Ok(0);
        }

        // 2. 向前追溯连续涨停天数
        let mut consecutive = 1u8;
        let mut check_date = self.prev_trading_day(current_date).await?;

        loop {
            let prev_limit = history.iter()
                .find(|r| r.code == code && r.trade_date == check_date);

            match prev_limit {
                Some(record) if record.is_limit_up == 1 => {
                    consecutive += 1;
                    check_date = self.prev_trading_day(check_date).await?;

                    if consecutive >= 30 {
                        break;
                    }
                }
                _ => break,
            }
        }

        Ok(consecutive)
    }

    /// 批量计算多只股票的连板数
    pub async fn batch_calculate(
        &self,
        codes: Vec<String>,
        current_date: NaiveDate,
    ) -> Result<HashMap<String, u8>> {
        use futures::stream::{self, StreamExt};

        let results = stream::iter(codes)
            .map(|code| async {
                let consecutive = self.calculate_consecutive(&code, current_date).await?;
                Ok::<(String, u8), anyhow::Error>((code, consecutive))
            })
            .buffer_unordered(20)
            .collect::<Vec<_>>()
            .await;

        results.into_iter().collect()
    }

    /// 更新连板追踪表
    ///
    /// # 逻辑
    /// 1. 查询当日所有涨停股票
    /// 2. 计算每只股票连板数
    /// 3. 更新consecutive_tracker表
    ///
    /// # 注意
    /// 此版本为简化实现,实际需要ClickHouse客户端
    pub async fn update_tracker(&self, _date: NaiveDate) -> Result<usize> {
        // TODO: 实现从ClickHouse查询并更新tracker表
        // 目前返回0
        Ok(0)
    }

    /// 判断是否创60日新高
    ///
    /// # 查询逻辑
    /// 查询前60个交易日的最高价,与当日最高价比较
    pub async fn is_new_high(
        &self,
        _code: &str,
        _date: NaiveDate,
        _high: f64,
    ) -> Result<bool> {
        // TODO: 实现从ClickHouse查询60日最高价
        // 目前返回false
        Ok(false)
    }

    /// 从历史记录判断是否新高
    pub fn is_new_high_from_history(
        &self,
        high: f64,
        history_60d_high: Option<f64>,
    ) -> bool {
        match history_60d_high {
            Some(max) => high >= max - 0.01,
            None => false,
        }
    }

    /// 获取股票的连板历史
    pub async fn get_consecutive_history(
        &self,
        _code: &str,
        _days: u16,
    ) -> Result<Vec<ConsecutiveHistory>> {
        // TODO: 实现从ClickHouse查询
        Ok(vec![])
    }

    /// 获取连板排行榜
    pub async fn get_consecutive_ranking(
        &self,
        _date: NaiveDate,
        _min_consecutive: u8,
        _limit: usize,
    ) -> Result<Vec<ConsecutiveRanking>> {
        // TODO: 实现从ClickHouse查询
        Ok(vec![])
    }

    /// 获取前一交易日
    ///
    /// # 集成TradingCalendar
    /// 使用真实的交易日历判断节假日
    pub async fn prev_trading_day(&self, date: NaiveDate) -> Result<NaiveDate> {
        use chrono::Duration;
        let mut prev = date - Duration::days(1);

        // 向前查找直到找到交易日
        let max_iterations = 10; // 最多查找10天
        let mut iterations = 0;

        while iterations < max_iterations {
            let naive_date = NaiveDate::from_ymd_opt(prev.year(), prev.month(), prev.day())
                .ok_or_else(|| anyhow::anyhow!("Invalid date"))?;

            // 检查是否为交易日
            if self.calendar.is_trading_day(naive_date).await {
                return Ok(prev);
            }

            prev = prev - Duration::days(1);
            iterations += 1;
        }

        Err(anyhow::anyhow!("无法找到前一交易日"))
    }

    /// 计算市场情绪指数
    ///
    /// # 综合评分公式
    /// 情绪指数 = (
    ///   涨停总数权重 × 0.3 +
    ///   连板高度权重 × 0.3 +
    ///   封单金额权重 × 0.2 +
    ///   一字板占比权重 × 0.2
    /// )
    pub async fn calculate_market_sentiment(&self, _date: NaiveDate) -> Result<MarketSentiment> {
        // TODO: 实现从ClickHouse查询统计数据
        // 目前返回空情绪指数
        Ok(MarketSentiment {
            date: _date,
            total_limit_up: 0,
            total_limit_down: 0,
            limit_up_ratio: 0.0,
            max_consecutive: 0,
            consecutive_gte_3: 0,
            consecutive_gte_5: 0,
            straight_count: 0,
            t_shape_count: 0,
            natural_count: 0,
            broken_count: 0,
            total_sealed_amount: 0.0,
            avg_sealed_amount: 0.0,
            sentiment_index: 0.0,
            sentiment_level: "未知".to_string(),
        })
    }
}
