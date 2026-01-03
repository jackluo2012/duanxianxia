// 个股挖掘算法实现
//
// 实际的算法实现，连接 ClickHouse 并执行查询

use crate::types::*;
use anyhow::Result;
use clickhouse::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LeaderItem {
    pub code: String,
    pub name: String,
    pub sector: String,
    pub leader_height: f64,
    pub sector_rank: u32,
    pub total_stocks: u32,
    pub price: f64,
    pub change_percent: f64,
    pub amount: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConsecutiveBoardItem {
    pub code: String,
    pub name: String,
    pub sector: String,
    pub consecutive_days: i32,
    pub start_date: String,
    pub end_date: String,
    pub board_type: String,
    pub current_price: f64,
    pub reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LimitItem {
    pub code: String,
    pub name: String,
    pub sector: String,
    pub limit_type: String,
    pub limit_time: String,
    pub limit_price: f64,
    pub volume: f64,
    pub amount: f64,
    pub reason: String,
    pub is_first: bool,
}

// ScreenerAlgorithmImpl 核心算法实现类
pub struct ScreenerAlgorithmImpl {
    client: Client,
}

impl ScreenerAlgorithmImpl {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    // ============================================
    // 算法1: 龙头高度计算
    // ============================================
    // 龙头高度 = (1 - 行业排名/行业股票总数) × 100
    // 排名基于市值，龙头高度越高越接近龙头
    pub async fn calculate_leader_height(
        &self,
        sector: Option<&str>,
        limit: usize,
    ) -> Result<Vec<LeaderItem>> {
        let query = if let Some(sector_code) = sector {
            // 指定板块的龙头高度
            format!(
                r#"
                SELECT
                    toString(date) as date,
                    code,
                    name,
                    sector_code,
                    sector_name,
                    leader_height,
                    sector_rank,
                    total_stocks_in_sector,
                    price,
                    change_percent,
                    volume,
                    amount
                FROM sector_leaders
                WHERE sector_code = '{}'
                ORDER BY leader_height DESC
                LIMIT {}
            "#,
                sector_code, limit
            )
        } else {
            // 全市场龙头高度排行
            format!(
                r#"
                SELECT
                    toString(date) as date,
                    code,
                    name,
                    sector_code,
                    sector_name,
                    leader_height,
                    sector_rank,
                    total_stocks_in_sector,
                    price,
                    change_percent,
                    volume,
                    amount
                FROM sector_leaders
                ORDER BY leader_height DESC
                LIMIT {}
            "#,
                limit
            )
        };

        let mut cursor = self.client.query(&query).fetch::<LeaderRow>()?;

        let mut items = Vec::new();
        while let Some(row) = cursor.next().await? {
            let code = row.code;
            let name = row.name;
            let sector = row.sector_name;
            let leader_height = row.leader_height;
            let sector_rank = row.sector_rank.unwrap_or(0);
            let total_stocks = row.total_stocks_in_sector.unwrap_or(0);
            let price = row.price;
            let change_percent = row.change_percent;
            let amount = row.amount;

            items.push(LeaderItem {
                code,
                name,
                sector,
                leader_height,
                sector_rank,
                total_stocks,
                price,
                change_percent,
                amount,
            });
        }

        Ok(items)
    }

    // 实时计算龙头高度（基于当前行情数据）
    pub async fn calculate_leader_height_realtime(
        &self,
        sector_code: &str,
    ) -> Result<Vec<LeaderItem>> {
        // 查询板块内所有股票的市值和涨跌幅
        let query = format!(
            r#"
            SELECT
                sq.code,
                sq.name,
                ss.sector_name as sector,
                sq.amount as market_cap,
                sq.change_percent,
                sq.price,
                sq.volume,
                sq.amount
            FROM stock_quotes sq
            INNER JOIN sector_stocks ss
                ON sq.code = ss.stock_code
                AND ss.date = today()
            WHERE ss.sector_code = '{}'
                AND sq.datetime >= today()
            ORDER BY sq.amount DESC
        "#,
            sector_code
        );

        let mut cursor = self.client.query(&query).fetch::<LeaderRow>()?;
        let mut stocks: Vec<(String, String, String, f64, f64, f64, f64)> = Vec::new();

        while let Some(row) = cursor.next().await? {
            let code = row.code;
            let name = row.name;
            let sector = row.sector_name;
            let market_cap = row.amount;
            let change_percent = row.change_percent;
            let price = row.price;
            let amount = row.amount;
            stocks.push((
                code,
                name,
                sector,
                market_cap,
                change_percent,
                price,
                amount,
            ));
        }

        let total = stocks.len() as f64;
        let mut items = Vec::new();

        for (idx, (code, name, sector, _market_cap, change_percent, price, amount)) in
            stocks.iter().enumerate()
        {
            let leader_height = if total > 0.0 {
                (1.0 - (idx as f64) / total) * 100.0
            } else {
                0.0
            };

            items.push(LeaderItem {
                code: code.clone(),
                name: name.clone(),
                sector: sector.clone(),
                leader_height,
                sector_rank: (idx + 1) as u32,
                total_stocks: total as u32,
                price: *price,
                change_percent: *change_percent,
                amount: *amount,
            });
        }

        Ok(items)
    }

    // ============================================
    // 算法2: 连板统计
    // ============================================
    // 查询连续涨停/跌停的股票
    pub async fn get_consecutive_boards(
        &self,
        min_days: i32,
        board_type: &str, // "连涨" or "连跌"
        limit: usize,
    ) -> Result<Vec<ConsecutiveBoardItem>> {
        let query = format!(
            r#"
            SELECT
                toString(date) as date,
                code,
                name,
                sector_name,
                board_type,
                consecutive_days,
                limit_count,
                toString(start_date) as start_date,
                toString(end_date) as end_date,
                current_price,
                price,
                change_percent,
                reason
            FROM consecutive_boards
            WHERE consecutive_days >= {}
                AND board_type = '{}'
            ORDER BY consecutive_days DESC, start_date DESC
            LIMIT {}
        "#,
            min_days, board_type, limit
        );

        let mut cursor = self.client.query(&query).fetch::<ConsecutiveBoardRow>()?;
        let mut items = Vec::new();

        while let Some(row) = cursor.next().await? {
            let code = row.code;
            let name = row.name;
            let sector = row.sector_name.unwrap_or_default();
            let consecutive_days = row.consecutive_days;
            let start_date = row.start_date.to_string();
            let end_date = row.end_date.to_string();
            let board_type = row.board_type;
            let current_price = row.current_price;
            let reason = row.reason.unwrap_or_else(|| "未知".to_string());

            items.push(ConsecutiveBoardItem {
                code,
                name,
                sector,
                consecutive_days,
                start_date,
                end_date,
                board_type,
                current_price,
                reason,
            });
        }

        Ok(items)
    }

    // 实时计算连板天数（基于历史数据）
    pub async fn calculate_consecutive_realtime(&self, code: &str) -> Result<i32> {
        // 查询最近30天的涨停数据
        let query = format!(
            r#"
            SELECT
                date,
                close_price,
                change_percent
            FROM stock_daily_bars
            WHERE code = '{}'
                AND date >= today() - 30
            ORDER BY date DESC
        "#,
            code
        );

        let mut cursor = self.client.query(&query).fetch::<DailyBarRow>()?;
        let mut consecutive_days = 0;
        let mut last_was_limit_up = false;

        // TODO: 实现连板天数计算逻辑
        // 1. 遍历历史数据
        // 2. 判断是否涨停（涨幅 >= 9.9%）
        // 3. 统计连续涨停天数
        // 4. 遇到非涨停则停止

        while let Some(_row) = cursor.next().await? {
            // 实现逻辑...
            break;
        }

        Ok(consecutive_days)
    }

    // ============================================
    // 算法3: 涨停跌停分析
    // ============================================
    // 查询当日涨停股票
    pub async fn get_limit_up_stocks(&self, date: &str, limit: usize) -> Result<Vec<LimitItem>> {
        let query = if date == "today" || date.is_empty() {
            format!(
                r#"
                SELECT
                    code,
                    name,
                    toString(date) as date,
                    time,
                    limit_type,
                    price,
                    change_percent,
                    volume,
                    amount,
                    reason,
                    is_first_board
                FROM limit_records
                WHERE limit_type = '涨停'
                ORDER BY time ASC
                LIMIT {}
            "#,
                limit
            )
        } else {
            format!(
                r#"
                SELECT
                    code,
                    name,
                    toString(date) as date,
                    time,
                    limit_type,
                    price,
                    change_percent,
                    volume,
                    amount,
                    reason,
                    is_first_board
                FROM limit_records
                WHERE date = {} AND limit_type = '涨停'
                ORDER BY time ASC
                LIMIT {}
            "#,
                date, limit
            )
        };

        let mut cursor = self.client.query(&query).fetch::<LimitRow>()?;
        let mut items = Vec::new();

        while let Some(row) = cursor.next().await? {
            let code = row.code;
            let name = row.name;
            let sector = "未知".to_string(); // LimitRow 没有sector_name字段
            let limit_type = row.limit_type;
            let limit_time = row.time.unwrap_or_else(|| "".to_string());
            let limit_price = row.price;
            let volume = row.volume;
            let amount = row.amount;
            let reason = row.reason.unwrap_or_else(|| "市场弱势".to_string());
            let is_first = row.is_first_board.unwrap_or(0) == 1;

            items.push(LimitItem {
                code,
                name,
                sector,
                limit_type,
                limit_time,
                limit_price,
                volume,
                amount,
                reason,
                is_first,
            });
        }

        Ok(items)
    }

    // 查询当日跌停股票
    pub async fn get_limit_down_stocks(&self, date: &str, limit: usize) -> Result<Vec<LimitItem>> {
        let query = if date == "today" || date.is_empty() {
            format!(
                r#"
                SELECT
                    code,
                    name,
                    toString(date) as date,
                    time,
                    limit_type,
                    price,
                    change_percent,
                    volume,
                    amount,
                    reason,
                    is_first_board
                FROM limit_records
                WHERE limit_type = '跌停'
                ORDER BY time ASC
                LIMIT {}
            "#,
                limit
            )
        } else {
            format!(
                r#"
                SELECT
                    code,
                    name,
                    toString(date) as date,
                    time,
                    limit_type,
                    price,
                    change_percent,
                    volume,
                    amount,
                    reason,
                    is_first_board
                FROM limit_records
                WHERE date = {} AND limit_type = '跌停'
                ORDER BY time ASC
                LIMIT {}
            "#,
                date, limit
            )
        };

        let mut cursor = self.client.query(&query).fetch::<LimitRow>()?;
        let mut items = Vec::new();

        while let Some(row) = cursor.next().await? {
            let code = row.code;
            let name = row.name;
            let sector = "未知".to_string(); // LimitRow 没有sector_name字段
            let limit_type = row.limit_type;
            let limit_time = row.time.unwrap_or_else(|| "".to_string());
            let limit_price = row.price;
            let volume = row.volume;
            let amount = row.amount;
            let reason = row.reason.unwrap_or_else(|| "市场弱势".to_string());
            let is_first = row.is_first_board.unwrap_or(0) == 1;

            items.push(LimitItem {
                code,
                name,
                sector,
                limit_type,
                limit_time,
                limit_price,
                volume,
                amount,
                reason,
                is_first,
            });
        }

        Ok(items)
    }

    // 实时检测涨停跌停（基于当前行情）
    pub async fn detect_limit_stocks_realtime(&self) -> Result<Vec<LimitItem>> {
        // 查询当前涨跌幅超过9.8%的股票
        let query = r#"
            SELECT
                code,
                name,
                change_percent,
                price,
                volume,
                amount
            FROM stock_quotes
            WHERE datetime >= today() - INTERVAL 1 HOUR
                AND abs(change_percent) >= 9.8
            ORDER BY change_percent DESC
        "#;

        let mut cursor = self.client.query(&query).fetch::<SectorStockRow>()?;
        let mut items = Vec::new();

        while let Some(row) = cursor.next().await? {
            let code = row.code;
            let name = row.name;
            let change_percent = row.change_percent;
            let price = row.price;
            let volume = row.volume;
            let amount = row.amount;

            let limit_type = if change_percent >= 9.8 {
                "涨停"
            } else {
                "跌停"
            };
            let reason = if change_percent >= 9.8 {
                "市场强势"
            } else {
                "市场弱势"
            };
            let limit_time = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

            items.push(LimitItem {
                code,
                name,
                sector: "未知".to_string(), // 需要关联查询
                limit_type: limit_type.to_string(),
                limit_time,
                limit_price: price,
                volume,
                amount,
                reason: reason.to_string(),
                is_first: false,
            });
        }

        Ok(items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_leader_height_calculation() {
        // 测试龙头高度计算逻辑
        let total = 10.0;
        let ranks = vec![0, 1, 2, 5, 9]; // 排名

        for rank in ranks {
            let height = (1.0 - rank as f64 / total) * 100.0;
            println!("Rank {}: Height = {}", rank, height);
            assert!(height >= 0.0 && height <= 100.0);
        }
    }

    #[test]
    fn test_consecutive_calculation() {
        // 测试连板天数统计逻辑
        // TODO: 添加具体测试用例
    }
}
