use crate::adapters::primary::http::{
    DailyReviewResponse, IntervalDistribution, IntervalStatsResponse, MarketSentiment,
};
use crate::domain::entities::models::*;
use crate::domain::entities::theme_models::*;
use anyhow::Result;
use clickhouse::Client;
use clickhouse::Row;
use serde::{Deserialize, Serialize};

/// 龙头排行榜查询结果行(简化版)
#[derive(Row, Serialize, Deserialize)]
struct LeaderBoardRow {
    code: String,
    name: String,
    price: f64,
    change_percent: f64,
    sector: String,
    consecutive_limit_up: u16, // 修复: UInt16而不是UInt8
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
    consecutive_limit_up: u16, // 修复: UInt16
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

/// 题材热度查询结果行
#[derive(Row, Serialize, Deserialize)]
struct ThemeHotnessRow {
    theme_name: String,
    theme_type: String,
    stock_count: u64,
    limit_up_count: u64,
    limit_down_count: u64,
    limit_up_ratio: f64,
    avg_consecutive: f64,
    max_consecutive: u8,
    total_consecutive_gte_3: u64,
    total_consecutive_gte_5: u64,
    total_sealed_amount: f64,
    avg_sealed_amount: f64,
    leader_code: String,
    leader_name: String,
    leader_consecutive: u8,
    cycle_stage: String,
    cycle_days: u8,
    hotness_rank: u8,
    hotness_score: f64,
    created_at: String, // will be converted from DateTime
}

/// 区间统计计数行
#[derive(Row, Serialize, Deserialize)]
struct IntervalCountRow {
    code: String,
    limit_count: u64,
    max_consecutive: u8,
}

/// 题材详情查询结果行
#[derive(Row, Serialize, Deserialize)]
struct ThemeDetailRow {
    code: String,
    name: String,
    is_limit_up: i8,
    consecutive_days: u8,
    sealed_amount: f64,
    turnover_rate: f32,
    limit_reason: String,
    industry: String,
    concept: String,
}

/// 题材关联查询结果行
#[derive(Row, Serialize, Deserialize)]
struct ThemeRelationRow {
    related_theme: String,
    common_stocks: u64,
    common_limit_count: u64,
    correlation_strength: f32,
}

/// ClickHouse数据库客户端
#[derive(Clone)]
pub struct Database {
    client: Client,
}

impl Database {
    /// 创建新的数据库连接
    pub fn new(url: &str) -> Self {
        let client = Client::default().with_url(url);

        Self { client }
    }

    /// 获取连板排行榜(指定日期)
    pub async fn get_leader_board(&self, date: &str) -> Result<Vec<LeaderBoardItem>> {
        let sql = format!("SELECT
                code,
                name,
                toFloat64(close_price) as price,
                toFloat64(if(open_price > 0, round((close_price - open_price) / open_price * 100, 2), 0)) as change_percent,
                coalesce(industry, '未知') as sector,
                consecutive_days + 1 as consecutive_limit_up,
                toFloat64(sealed_amount) as sealed_amount
            FROM duanxianxia.limit_up_review
            WHERE trade_date = '{}' AND is_limit_up = 1
            ORDER BY consecutive_days DESC, sealed_amount DESC
            LIMIT 100", date);

        let mut cursor = self.client
            .query(&sql)
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
                history_max: 0,           // TODO: 计算历史最高连板
                recent_limit_ups: vec![], // TODO: 查询历史涨停日期
                sealed_amount: row.sealed_amount,
            });
        }

        Ok(items)
    }

    /// 获取股票详情(包含历史涨停记录)
    pub async fn get_leader_detail(
        &self,
        code: &str,
        limit_days: u32,
    ) -> Result<Option<LeaderDetail>> {
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
            LIMIT ?")
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
                toFloat64(ifNull(high_price, 0)) as high_price,
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

        let mut cursor = self
            .client
            .query(&sql)
            .fetch::<ReviewRow>()?;

        let mut reviews = Vec::new();
        while let Some(row) = cursor.next().await? {
            // 转换字符串到Option类型
            let limit_type = if row.limit_type.is_empty() {
                None
            } else {
                Some(row.limit_type)
            };
            let first_limit_time = if row.first_limit_time.is_empty() {
                None
            } else {
                Some(row.first_limit_time)
            };
            let last_limit_time = if row.last_limit_time.is_empty() {
                None
            } else {
                Some(row.last_limit_time)
            };
            let industry = if row.industry.is_empty() {
                None
            } else {
                Some(row.industry)
            };
            let concept = if row.concept.is_empty() {
                None
            } else {
                Some(row.concept)
            };
            let limit_reason = if row.limit_reason.is_empty() {
                None
            } else {
                Some(row.limit_reason)
            };
            let remark = if row.remark.is_empty() {
                None
            } else {
                Some(row.remark)
            };
            let seal_period = if row.seal_period.is_empty() {
                None
            } else {
                Some(row.seal_period)
            };

            // 转换DateTime字符串
            let first_limit_dt = first_limit_time
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc));
            let last_limit_dt = last_limit_time
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc));

            // 转换价格字段到Option类型
            let limit_price = if row.limit_price == 0.0 {
                None
            } else {
                Some(row.limit_price)
            };
            let open_price = if row.open_price == 0.0 {
                None
            } else {
                Some(row.open_price)
            };
            let close_price = if row.close_price == 0.0 {
                None
            } else {
                Some(row.close_price)
            };
            let high_price = if row.high_price == 0.0 {
                None
            } else {
                Some(row.high_price)
            };
            let low_price = if row.low_price == 0.0 {
                None
            } else {
                Some(row.low_price)
            };

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

                // 新增字段 - 暂时设为None或默认值
                limit_direction: None,
                max_consecutive: 0,
                interval_stats: None,
                strength_score: if row.strength_score == 0.0 {
                    None
                } else {
                    Some(row.strength_score as f32)
                },
                manual_reason: None,
                reason_source: None,

                last_consecutive: row.last_consecutive as i32,
                is_new_high: row.is_new_high as i32,
                industry,
                concept,
                limit_reason,
                remark,
                limit_duration: if row.limit_duration == 0 {
                    None
                } else {
                    Some(row.limit_duration as i32)
                },
                seal_period,
                volume: Some(row.volume as f64),
                amount: Some(row.amount),
                turnover_rate: Some(row.turnover_rate),
                sealed_volume: Some(row.sealed_volume as i64),
                buy1_to_buy5_vol: Some(row.buy1_to_buy5_vol as i64),
            });
        }

        Ok(reviews)
    }

    /// 获取带区间统计的完整每日复盘数据
    pub async fn get_daily_review_with_interval(&self, date: &str) -> Result<DailyReviewResponse> {
        // 1. 获取所有复盘数据
        let all_reviews = self.get_daily_review(date).await?;

        // 2. 分离涨停和跌停股票
        let limit_up_stocks: Vec<LimitUpReview> = all_reviews
            .iter()
            .filter(|r| r.is_limit_up == 1)
            .cloned()
            .collect();

        let limit_down_stocks: Vec<LimitUpReview> = all_reviews
            .iter()
            .filter(|r| r.is_limit_up != 1)
            .cloned()
            .collect();

        // 3. 计算市场情绪
        let total_limit_up = limit_up_stocks.len() as i32;
        let total_limit_down = limit_down_stocks.len() as i32;
        let max_consecutive = limit_up_stocks
            .iter()
            .map(|s| s.consecutive_days)
            .max()
            .unwrap_or(0);

        // 简单的情绪指数计算 (涨停数 - 跌停数)
        let sentiment_index = if (total_limit_up + total_limit_down) > 0 {
            ((total_limit_up - total_limit_down) as f64
                / (total_limit_up + total_limit_down) as f64)
                * 100.0
        } else {
            0.0
        };

        let market_sentiment = MarketSentiment {
            date: date.to_string(),
            total_limit_up,
            total_limit_down,
            max_consecutive,
            sentiment_index,
        };

        // 4. 计算区间统计
        let interval_stats = self.calculate_interval_stats(&limit_up_stocks).await?;

        Ok(DailyReviewResponse {
            market_sentiment,
            limit_up_stocks,
            limit_down_stocks,
            interval_stats,
        })
    }

    /// 计算区间统计分布
    ///
    /// 查询最近5/10/20个交易日内每只股票的涨停次数和连板情况
    /// 返回不同区间内涨停次数的股票分布统计
    async fn calculate_interval_stats(
        &self,
        stocks: &[LimitUpReview],
    ) -> Result<IntervalStatsResponse> {
        if stocks.is_empty() {
            // 返回空分布
            return Ok(IntervalStatsResponse {
                days_5: IntervalDistribution::empty(),
                days_10: IntervalDistribution::empty(),
                days_20: IntervalDistribution::empty(),
            });
        }

        // 获取当前日期（使用第一只股票的交易日期）
        let current_date = stocks[0].trade_date.format("%Y-%m-%d").to_string();

        // 计算5/10/20天的区间分布
        let days_5 = self.calculate_interval_distribution(&current_date, 5).await?;
        let days_10 = self.calculate_interval_distribution(&current_date, 10).await?;
        let days_20 = self.calculate_interval_distribution(&current_date, 20).await?;

        Ok(IntervalStatsResponse {
            days_5,
            days_10,
            days_20,
        })
    }

    /// 计算指定交易天数窗口内的涨停分布
    ///
    /// 查询最近N个交易日内每只股票的涨停次数，并返回分布统计
    async fn calculate_interval_distribution(
        &self,
        end_date: &str,
        trading_days: u32,
    ) -> Result<IntervalDistribution> {
        tracing::debug!("计算{}天区间涨停分布: {}", trading_days, end_date);

        // 查询区间内每只股票的涨停次数
        let sql = format!(
            "SELECT
                code,
                countIf(is_limit_up = 1) as limit_count,
                max(consecutive_days) as max_consecutive
            FROM duanxianxia.limit_up_review
            WHERE trade_date <= '{}' AND trade_date >= toDate(toDate('{}') - INTERVAL {} DAY)
            GROUP BY code
            HAVING limit_count > 0",
            end_date, end_date, trading_days * 2 // 粗略估算日历天数
        );

        let mut cursor = self.client.query(&sql).fetch::<IntervalCountRow>()?;

        let mut distribution = IntervalDistribution::empty();

        while let Some(row) = cursor.next().await? {
            // 根据区间内涨停次数分类统计
            match row.limit_count {
                c if c >= 8 => distribution.count_8 += 1,
                c if c >= 7 => distribution.count_7 += 1,
                c if c >= 6 => distribution.count_6 += 1,
                c if c >= 5 => distribution.count_5 += 1,
                c if c >= 4 => distribution.count_4 += 1,
                c if c >= 3 => distribution.count_3 += 1,
                c if c >= 2 => distribution.count_2 += 1,
                c if c >= 1 => distribution.count_1 += 1,
                _ => {}
            }
        }

        Ok(distribution)
    }

    /// 获取题材热度榜
    ///
    /// 基于limit_up_review表实时计算题材热度，包括：
    /// - 涨停股票数量统计
    /// - 连板高度统计
    /// - 封单金额统计
    /// - 龙头股票识别
    pub async fn get_theme_hotness(&self, date: &str, limit: usize) -> Result<Vec<ThemeHotness>> {
        tracing::info!("📊 计算题材热度: {}", date);

        // 从concept字段提取题材并统计
        let sql = format!(
            "SELECT
                multiIf(
                    concept = '', '未分类',
                    position(concept, ',') > 0, splitByString(',', concept)[1],
                    concept
                ) as theme_name,
                'concept' as theme_type,
                count() as stock_count,
                countIf(is_limit_up = 1) as limit_up_count,
                countIf(is_limit_up = -1) as limit_down_count,
                round(countIf(is_limit_up = 1) / count() * 100, 2) as limit_up_ratio,
                round(avg(consecutive_days), 2) as avg_consecutive,
                max(consecutive_days) as max_consecutive,
                countIf(consecutive_days >= 3) as total_consecutive_gte_3,
                countIf(consecutive_days >= 5) as total_consecutive_gte_5,
                coalesce(sum(if(is_limit_up = 1, sealed_amount, 0)), 0) as total_sealed_amount,
                round(coalesce(avg(if(is_limit_up = 1, sealed_amount, null)), 0), 2) as avg_sealed_amount,
                argMax(code, sealed_amount) as leader_code,
                argMax(name, sealed_amount) as leader_name,
                max(consecutive_days) as leader_consecutive,
                'init' as cycle_stage,
                0 as cycle_days,
                0 as hotness_rank,
                0.0 as hotness_score,
                toString(now()) as created_at
            FROM duanxianxia.limit_up_review
            WHERE trade_date = '{}' AND concept != ''
            GROUP BY theme_name, theme_type
            ORDER BY limit_up_count DESC, max_consecutive DESC, total_sealed_amount DESC
            LIMIT {}",
            date, limit
        );

        let mut cursor = self.client.query(&sql).fetch::<ThemeHotnessRow>()?;
        let mut themes = Vec::new();
        while let Some(row) = cursor.next().await? {
            let theme = ThemeHotness {
                trade_date: chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
                    .map_err(|e| anyhow::anyhow!("Invalid date format: {}", e))?,
                theme_name: row.theme_name,
                theme_type: if row.theme_type == "industry" {
                    crate::domain::entities::theme_models::ThemeType::Industry
                } else {
                    crate::domain::entities::theme_models::ThemeType::Concept
                },
                stock_count: row.stock_count as u16,
                limit_up_count: row.limit_up_count as u16,
                limit_down_count: row.limit_down_count as u16,
                limit_up_ratio: row.limit_up_ratio as f32,
                avg_consecutive: row.avg_consecutive as f32,
                max_consecutive: row.max_consecutive as u16,
                total_consecutive_gte_3: row.total_consecutive_gte_3 as u16,
                total_consecutive_gte_5: row.total_consecutive_gte_5 as u16,
                total_sealed_amount: row.total_sealed_amount,
                avg_sealed_amount: row.avg_sealed_amount,
                leader_code: row.leader_code,
                leader_name: row.leader_name,
                leader_consecutive: row.leader_consecutive as u16,
                cycle_stage: crate::domain::entities::theme_models::CycleStage::Init,
                cycle_days: row.cycle_days,
                hotness_rank: row.hotness_rank as u16 + 1,
                hotness_score: row.hotness_score,
                created_at: chrono::Utc::now(),
            };
            themes.push(theme);
        }

        tracing::info!("✅ 计算出{}个题材", themes.len());
        Ok(themes)
    }

    /// 获取题材详情
    ///
    /// 返回指定题材在指定日期的详细信息，包括：
    /// - 题材基本统计
    /// - 涨停股票列表（按连板数排序）
    /// - 龙头、中军、跟风股票分层
    pub async fn get_theme_detail(
        &self,
        date: &str,
        theme_name: &str,
    ) -> Result<serde_json::Value> {
        tracing::info!("📊 查询题材详情: {} - {}", date, theme_name);

        // 查询该题材下的所有涨停股票
        let pattern = format!("%{}%", theme_name);
        let sql = format!("SELECT
            code,
            name,
            is_limit_up,
            consecutive_days,
            sealed_amount,
            turnover_rate,
            limit_reason,
            industry,
            concept
        FROM duanxianxia.limit_up_review
        WHERE trade_date = '{}' AND (concept LIKE '{}' OR industry = '{}')
        ORDER BY consecutive_days DESC, sealed_amount DESC", date, pattern, theme_name);

        let mut cursor = self.client.query(&sql).fetch::<ThemeDetailRow>()?;

        let mut stocks = Vec::new();
        let mut total_sealed = 0.0;
        let mut max_consecutive = 0u8;
        let mut limit_up_count = 0usize;

        while let Some(row) = cursor.next().await? {
            total_sealed += row.sealed_amount;
            max_consecutive = max_consecutive.max(row.consecutive_days);
            if row.is_limit_up == 1 {
                limit_up_count += 1;
            }

            stocks.push(serde_json::json!({
                "code": row.code,
                "name": row.name,
                "consecutive_days": row.consecutive_days,
                "sealed_amount": row.sealed_amount,
                "turnover_rate": row.turnover_rate,
                "limit_reason": row.limit_reason,
            }));
        }

        // 分层：龙头(连板>=5)、中军(3<=连板<5)、跟风(连板<3)
        let leaders: Vec<_> = stocks.iter()
            .filter(|s| s["consecutive_days"].as_u64().unwrap_or(0) >= 5)
            .cloned()
            .collect();

        let mid: Vec<_> = stocks.iter()
            .filter(|s| {
                let cons = s["consecutive_days"].as_u64().unwrap_or(0);
                cons >= 3 && cons < 5
            })
            .cloned()
            .collect();

        let followers: Vec<_> = stocks.iter()
            .filter(|s| s["consecutive_days"].as_u64().unwrap_or(0) < 3)
            .cloned()
            .collect();

        Ok(serde_json::json!({
            "theme_name": theme_name,
            "date": date,
            "stats": {
                "total_stocks": limit_up_count,
                "max_consecutive": max_consecutive,
                "total_sealed_amount": total_sealed,
            },
            "stocks": {
                "leaders": leaders,
                "mid": mid,
                "followers": followers,
            }
        }))
    }

    /// 获取题材关联关系
    ///
    /// 基于共同涨停股票挖掘题材之间的关联关系
    /// 返回与指定题材相关的其他题材及关联强度
    pub async fn get_theme_relations(
        &self,
        date: &str,
        theme_name: &str,
    ) -> Result<Vec<ThemeRelation>> {
        tracing::info!("📊 查询题材关联: {} - {}", date, theme_name);

        // 查询与当前题材有共同涨停股票的其他题材
        let pattern = format!("%{}%", theme_name);
        let sql = format!("SELECT
            arrayJoin(splitByString(',', concept)) as related_theme,
            count() as common_stocks,
            countIf(is_limit_up = 1) as common_limit_count,
            round(countIf(is_limit_up = 1) / count() * 100, 2) as correlation_strength
        FROM duanxianxia.limit_up_review
        WHERE trade_date = '{}' AND concept LIKE '{}' AND concept != ''
        GROUP BY related_theme
        HAVING related_theme != '{}' AND common_stocks >= 2
        ORDER BY common_limit_count DESC, correlation_strength DESC
        LIMIT 10", date, pattern, theme_name);

        let mut cursor = self.client.query(&sql).fetch::<ThemeRelationRow>()?;

        let mut relations = Vec::new();
        while let Some(row) = cursor.next().await? {
            let trade_date = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
                .map_err(|e| anyhow::anyhow!("Invalid date format: {}", e))?;

            relations.push(ThemeRelation {
                trade_date,
                parent_theme: theme_name.to_string(),
                child_theme: row.related_theme,
                relation_type: crate::domain::entities::theme_models::RelationType::Related,
                correlation_strength: row.correlation_strength,
                common_stocks: row.common_stocks as u16,
                common_limit_count: row.common_limit_count as u16,
                created_at: chrono::Utc::now(),
            });
        }

        tracing::info!("✅ 找到{}个关联题材", relations.len());
        Ok(relations)
    }
}
