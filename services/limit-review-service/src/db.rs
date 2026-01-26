use anyhow::Result;
use clickhouse::Client;
use clickhouse::sql::Identifier;
use clickhouse::Row;
use serde::{Deserialize, Serialize};
use crate::models::*;

/// 龙头排行榜查询结果行(简化版)
#[derive(Row, Serialize, Deserialize)]
struct LeaderBoardRow {
    code: String,
    name: String,
    price: f64,
    change_percent: f64,
    sector: String,
    consecutive_limit_up: u16,  // 修复: UInt16而不是UInt8
    sealed_amount: f64,
}

/// 股票详情查询结果行
#[derive(Row, Serialize, Deserialize)]
struct LatestLimitRow {
    code: String,
    name: String,
    price: f64,
    change_percent: f64,
    market_cap: f64,
    sector: String,
    consecutive_limit_up: u16,  // 修复: UInt16
    history_max: u8,
    sealed_amount: f64,
    trade_date: String,
    open_times: u8,
}

/// 历史涨停记录查询结果行
#[derive(Row, Serialize, Deserialize)]
struct HistoryRow {
    date: String,
    change_percent: f64,
    sealed_amount: f64,
    open_count: u8,
    final_sealed: f64,
}

/// 每日复盘查询结果行(简化版，不使用Option)
#[derive(Row, Serialize, Deserialize)]
struct ReviewRow {
    trade_date: String,
    code: String,
    name: String,
    is_limit_up: i8,
    limit_type: String,
    first_limit_time: String,
    last_limit_time: String,
    open_times: u8,
    limit_price: f64,
    open_price: f64,
    close_price: f64,
    high_price: f64,
    low_price: f64,
    volume: u64,
    amount: f64,
    turnover_rate: f64,
    sealed_amount: f64,
    sealed_volume: u32,
    buy1_to_buy5_vol: u32,
    consecutive_days: u8,
    last_consecutive: u8,
    is_new_high: u8,
    industry: String,
    concept: String,
    limit_reason: String,
    remark: String,
    limit_duration: u16,
    seal_period: String,
    strength_score: f64,
}

/// ClickHouse数据库客户端
#[derive(Clone)]
pub struct Database {
    client: Client,
}

impl Database {
    /// 创建新的数据库连接
    pub fn new(url: &str) -> Self {
        let client = Client::default()
            .with_url(url);

        Self { client }
    }

    /// 获取连板排行榜(指定日期)
    pub async fn get_leader_board(&self, date: &str) -> Result<Vec<LeaderBoardItem>> {
        let mut cursor = self.client
            .query("SELECT
                code,
                name,
                toFloat64(close_price) as price,
                toFloat64(if(open_price > 0, round((close_price - open_price) / open_price * 100, 2), 0)) as change_percent,
                coalesce(industry, '未知') as sector,
                consecutive_days + 1 as consecutive_limit_up,
                toFloat64(sealed_amount) as sealed_amount
            FROM duanxianxia.limit_up_review
            WHERE trade_date = ? AND is_limit_up = 1
            ORDER BY consecutive_days DESC, sealed_amount DESC
            LIMIT 100")
            .fetch::<LeaderBoardRow>()?;

        let mut items = Vec::new();
        while let Some(row) = cursor.next().await? {
            items.push(LeaderBoardItem {
                code: row.code,
                name: row.name,
                price: row.price,
                change_percent: row.change_percent,
                market_cap: 0.0, // TODO: 从其他表获取市值
                sector: row.sector,
                consecutive_limit_up: row.consecutive_limit_up as i32,
                history_max: 0, // TODO: 计算历史最高连板
                recent_limit_ups: vec![], // TODO: 查询历史涨停日期
                sealed_amount: row.sealed_amount,
            });
        }

        Ok(items)
    }

    /// 获取股票详情(包含历史涨停记录)
    pub async fn get_leader_detail(&self, code: &str, limit_days: u32) -> Result<Option<LeaderDetail>> {
        // 查询最新涨停记录
        let mut cursor = self.client
            .query("SELECT
                code,
                name,
                toFloat64(close_price) as price,
                toFloat64(if(open_price > 0, round((close_price - open_price) / open_price * 100, 2), 0)) as change_percent,
                toFloat64(0.0) as market_cap,
                coalesce(industry, '未知') as sector,
                consecutive_days + 1 as consecutive_limit_up,
                0 as history_max,
                toFloat64(sealed_amount) as sealed_amount,
                toString(trade_date) as trade_date,
                open_times
            FROM duanxianxia.limit_up_review
            WHERE code = '{}' AND is_limit_up = 1
            ORDER BY trade_date DESC
            LIMIT 1")
            .fetch::<LatestLimitRow>()?;

        let latest = if let Some(row) = cursor.next().await? {
            row
        } else {
            return Ok(None);
        };

        // 查询历史涨停记录
        let mut history_cursor = self.client
            .query("SELECT
                toString(trade_date) as date,
                toFloat64(if(open_price > 0, round((close_price - open_price) / open_price * 100, 2), 0)) as change_percent,
                toFloat64(sealed_amount) as sealed_amount,
                open_times as open_count,
                toFloat64(sealed_amount) as final_sealed
            FROM duanxianxia.limit_up_review
            WHERE code = '{}' AND is_limit_up = 1
            ORDER BY trade_date DESC
            LIMIT {}")
            .fetch::<HistoryRow>()?;

        let mut limit_up_history = Vec::new();
        let mut recent_limit_ups = Vec::new();
        let mut total_sealed_amount = 0.0;

        while let Some(row) = history_cursor.next().await? {
            recent_limit_ups.push(row.date.clone());
            total_sealed_amount += row.sealed_amount;

            limit_up_history.push(LimitUpHistoryRecord {
                date: row.date,
                change_percent: row.change_percent,
                sealed_amount: row.sealed_amount,
                open_count: row.open_count as i32,
                final_sealed: row.final_sealed,
            });
        }

        // 获取首次涨停日期
        let first_limit_date = if recent_limit_ups.len() > 0 {
            recent_limit_ups.last().cloned().unwrap_or_default()
        } else {
            latest.trade_date.clone()
        };

        // 获取最新涨停日期
        let latest_limit_date = recent_limit_ups.first().cloned().unwrap_or_default();

        Ok(Some(LeaderDetail {
            code: latest.code,
            name: latest.name,
            price: latest.price,
            change_percent: latest.change_percent,
            market_cap: latest.market_cap,
            sector: latest.sector,
            consecutive_limit_up: latest.consecutive_limit_up as i32,
            history_max: latest.history_max as i32,
            first_limit_up_date: first_limit_date,
            latest_limit_up_date: latest_limit_date,
            total_sealed_amount,
            recent_limit_ups,
            sealed_amount: latest.sealed_amount,
            limit_up_history,
        }))
    }

    /// 获取指定日期的涨停复盘数据
    pub async fn get_daily_review(&self, date: &str) -> Result<Vec<LimitUpReview>> {
        let sql = format!("SELECT
                toString(trade_date) as trade_date,
                code,
                name,
                is_limit_up,
                ifNull(toString(limit_type), '') as limit_type,
                ifNull(toString(first_limit_time), '') as first_limit_time,
                ifNull(toString(last_limit_time), '') as last_limit_time,
                open_times,
                toFloat64(ifNull(limit_price, 0)) as limit_price,
                toFloat64(ifNull(open_price, 0)) as open_price,
                toFloat64(ifNull(close_price, 0)) as close_price,
                toFloat64(ifNull(high_price), 0)) as high_price,
                toFloat64(ifNull(low_price, 0)) as low_price,
                volume,
                toFloat64(amount) as amount,
                toFloat64(turnover_rate) as turnover_rate,
                toFloat64(sealed_amount) as sealed_amount,
                sealed_volume,
                buy1_to_buy5_vol,
                consecutive_days,
                last_consecutive,
                is_new_high,
                ifNull(industry, '') as industry,
                ifNull(concept, '') as concept,
                ifNull(limit_reason, '') as limit_reason,
                ifNull(remark, '') as remark,
                ifNull(limit_duration, 0) as limit_duration,
                ifNull(seal_period, '') as seal_period,
                toFloat64(ifNull(strength_score, 0)) as strength_score
            FROM duanxianxia.limit_up_review
            WHERE trade_date = '{}'", date);

        let mut cursor = self.client
            .query(&sql)
            .fetch::<ReviewRow>()?;

        let mut reviews = Vec::new();
        while let Some(row) = cursor.next().await? {
            // 转换字符串到Option类型
            let limit_type = if row.limit_type.is_empty() { None } else { Some(row.limit_type) };
            let first_limit_time = if row.first_limit_time.is_empty() { None } else { Some(row.first_limit_time) };
            let last_limit_time = if row.last_limit_time.is_empty() { None } else { Some(row.last_limit_time) };
            let industry = if row.industry.is_empty() { None } else { Some(row.industry) };
            let concept = if row.concept.is_empty() { None } else { Some(row.concept) };
            let limit_reason = if row.limit_reason.is_empty() { None } else { Some(row.limit_reason) };
            let remark = if row.remark.is_empty() { None } else { Some(row.remark) };
            let seal_period = if row.seal_period.is_empty() { None } else { Some(row.seal_period) };

            // 转换DateTime字符串
            let first_limit_dt = first_limit_time.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc));
            let last_limit_dt = last_limit_time.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc));

            // 转换价格字段到Option类型
            let limit_price = if row.limit_price == 0.0 { None } else { Some(row.limit_price) };
            let open_price = if row.open_price == 0.0 { None } else { Some(row.open_price) };
            let close_price = if row.close_price == 0.0 { None } else { Some(row.close_price) };
            let high_price = if row.high_price == 0.0 { None } else { Some(row.high_price) };
            let low_price = if row.low_price == 0.0 { None } else { Some(row.low_price) };

            reviews.push(LimitUpReview {
                trade_date: chrono::NaiveDate::parse_from_str(&row.trade_date, "%Y-%m-%d")?,
                code: row.code,
                name: row.name,
                is_limit_up: row.is_limit_up as i32,
                limit_type,
                first_limit_time: first_limit_dt,
                last_limit_time: last_limit_dt,
                open_times: row.open_times as i32,
                consecutive_days: row.consecutive_days as i32,
                sealed_amount: Some(row.sealed_amount),
                last_consecutive: row.last_consecutive as i32,
                is_new_high: row.is_new_high as i32,
                industry,
                concept,
                limit_reason,
                remark,
                limit_duration: if row.limit_duration == 0 { None } else { Some(row.limit_duration as i32) },
                seal_period,
                strength_score: if row.strength_score == 0.0 { None } else { Some(row.strength_score) },
                volume: Some(row.volume as f64),
                amount: Some(row.amount),
                turnover_rate: Some(row.turnover_rate),
                sealed_volume: Some(row.sealed_volume as i64),
                buy1_to_buy5_vol: Some(row.buy1_to_buy5_vol as i64),
            });
        }

        Ok(reviews)
    }
}
