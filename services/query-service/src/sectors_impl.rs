// 概念板块算法实现
//
// 实际的板块查询和统计算法

use crate::types::*;
use anyhow::Result;
use clickhouse::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Sector {
    pub code: String,
    pub name: String,
    pub stock_count: i32,
    pub avg_change_percent: f64,
    pub total_amount: f64,
    pub limit_up_count: i32,
    pub limit_down_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectorStock {
    pub code: String,
    pub name: String,
    pub price: f64,
    pub change_percent: f64,
    pub volume: f64,
    pub amount: f64,
    pub market_cap: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SectorPerformance {
    pub sector_code: String,
    pub sector_name: String,
    pub avg_change_percent: f64,
    pub median_change_percent: f64,
    pub total_volume: f64,
    pub total_amount: f64,
    pub stock_count: i32,
    pub limit_up_count: i32,
    pub limit_down_count: i32,
    pub rise_count: i32,
    pub fall_count: i32,
    pub flat_count: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SectorFlow {
    pub sector_code: String,
    pub sector_name: String,
    pub inflow: f64,        // 资金流入
    pub outflow: f64,       // 资金流出
    pub net_inflow: f64,    // 净流入
    pub main_inflow: f64,   // 主力流入
    pub retail_inflow: f64, // 散户流入
}

// SectorAlgorithmImpl 板块算法实现类
pub struct SectorAlgorithmImpl {
    client: Client,
}

impl SectorAlgorithmImpl {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    // ============================================
    // 功能1: 获取所有板块列表
    // ============================================
    pub async fn get_sectors(&self, date: &str) -> Result<Vec<Sector>> {
        let query = if date == "today" || date.is_empty() {
            format!(
                r#"
                SELECT
                    sector_code as code,
                    sector_name as name,
                    stock_count,
                    avg_change_percent,
                    total_amount,
                    limit_up_count,
                    limit_down_count
                FROM sector_performance
                WHERE date = today()
                ORDER BY total_amount DESC
            "#
            )
        } else {
            format!(
                r#"
                SELECT
                    sector_code as code,
                    sector_name as name,
                    stock_count,
                    avg_change_percent,
                    total_amount,
                    limit_up_count,
                    limit_down_count
                FROM sector_performance
                WHERE date = {}
                ORDER BY total_amount DESC
            "#,
                date
            )
        };

        let mut cursor = self.client.query(&query).fetch::<SectorRow>()?;
        let mut sectors = Vec::new();

        while let Some(row) = cursor.next().await? {
            let code = row.code;
            let name = row.name;
            let stock_count = row.stock_count;
            let avg_change_percent = row.avg_change_percent;
            let total_amount = row.total_amount;
            let limit_up_count = row.limit_up_count;
            let limit_down_count = row.limit_down_count;

            sectors.push(Sector {
                code,
                name,
                stock_count,
                avg_change_percent,
                total_amount,
                limit_up_count,
                limit_down_count,
            });
        }

        Ok(sectors)
    }

    // ============================================
    // 功能2: 获取板块内股票列表
    // ============================================
    pub async fn get_sector_stocks(
        &self,
        sector_code: &str,
        date: &str,
    ) -> Result<Vec<SectorStock>> {
        let query = if date == "today" || date.is_empty() {
            format!(
                r#"
                SELECT
                    sq.code,
                    sq.name,
                    sq.price,
                    sq.change_percent,
                    sq.volume,
                    sq.amount,
                    sq.amount as market_cap
                FROM stock_quotes sq
                INNER JOIN sector_stocks ss
                    ON sq.code = ss.stock_code
                    AND ss.date = today()
                WHERE ss.sector_code = '{}'
                    AND sq.datetime >= today()
                ORDER BY sq.amount DESC
            "#,
                sector_code
            )
        } else {
            format!(
                r#"
                SELECT
                    sq.code,
                    sq.name,
                    sq.price,
                    sq.change_percent,
                    sq.volume,
                    sq.amount,
                    sq.amount as market_cap
                FROM stock_quotes sq
                INNER JOIN sector_stocks ss
                    ON sq.code = ss.stock_code
                    AND ss.date = {}
                WHERE ss.sector_code = '{}'
                ORDER BY sq.amount DESC
            "#,
                date, sector_code
            )
        };

        let mut cursor = self.client.query(&query).fetch::<SectorStockRow>()?;
        let mut stocks = Vec::new();

        while let Some(row) = cursor.next().await? {
            let code = row.code;
            let name = row.name;
            let price = row.price;
            let change_percent = row.change_percent;
            let volume = row.volume;
            let amount = row.amount;
            let market_cap = row.amount; // amount 作为 market_cap

            stocks.push(SectorStock {
                code,
                name,
                price,
                change_percent,
                volume,
                amount,
                market_cap,
            });
        }

        Ok(stocks)
    }

    // 实时查询板块内股票（从 sector_stocks 表获取关联）
    pub async fn get_sector_stocks_realtime(&self, sector_code: &str) -> Result<Vec<SectorStock>> {
        // 先获取板块内的股票代码列表
        let codes_query = format!(
            r#"
            SELECT stock_code
            FROM sector_stocks
            WHERE sector_code = '{}' AND date = today()
        "#,
            sector_code
        );

        let mut codes_cursor = self
            .client
            .query(&codes_query)
            .fetch::<SectorStockCodeRow>()?;
        let mut stock_codes = Vec::new();

        while let Some(row) = codes_cursor.next().await? {
            let code = row.stock_code;
            stock_codes.push(code);
        }

        if stock_codes.is_empty() {
            return Ok(Vec::new());
        }

        // 批量查询这些股票的实时行情
        let codes_str = stock_codes
            .iter()
            .map(|c| format!("'{}'", c))
            .collect::<Vec<_>>()
            .join(",");

        let quotes_query = format!(
            r#"
            SELECT
                code,
                name,
                price,
                change_percent,
                volume,
                amount,
                amount as market_cap
            FROM stock_quotes
            WHERE code IN ({})
                AND datetime >= today() - INTERVAL 1 HOUR
            ORDER BY amount DESC
        "#,
            codes_str
        );

        let mut quotes_cursor = self.client.query(&quotes_query).fetch::<SectorStockRow>()?;
        let mut stocks = Vec::new();

        while let Some(row) = quotes_cursor.next().await? {
            let code = row.code;
            let name = row.name;
            let price = row.price;
            let change_percent = row.change_percent;
            let volume = row.volume;
            let amount = row.amount;
            let market_cap = row.amount; // amount 作为 market_cap

            stocks.push(SectorStock {
                code,
                name,
                price,
                change_percent,
                volume,
                amount,
                market_cap,
            });
        }

        Ok(stocks)
    }

    // ============================================
    // 功能3: 获取板块表现排行
    // ============================================
    pub async fn get_sector_performance(
        &self,
        date: &str,
        limit: usize,
    ) -> Result<Vec<SectorPerformance>> {
        let query = if date == "today" || date.is_empty() {
            format!(
                r#"
                SELECT
                    sector_code,
                    sector_name,
                    avg_change_percent,
                    median_change_percent,
                    total_volume,
                    total_amount,
                    stock_count,
                    limit_up_count,
                    limit_down_count,
                    rise_count,
                    fall_count,
                    flat_count
                FROM sector_performance
                WHERE date = today()
                ORDER BY avg_change_percent DESC
                LIMIT {}
            "#,
                limit
            )
        } else {
            format!(
                r#"
                SELECT
                    sector_code,
                    sector_name,
                    avg_change_percent,
                    median_change_percent,
                    total_volume,
                    total_amount,
                    stock_count,
                    limit_up_count,
                    limit_down_count,
                    rise_count,
                    fall_count,
                    flat_count
                FROM sector_performance
                WHERE date = {}
                ORDER BY avg_change_percent DESC
                LIMIT {}
            "#,
                date, limit
            )
        };

        let mut cursor = self.client.query(&query).fetch::<SectorPerformanceRow>()?;
        let mut performances = Vec::new();

        while let Some(row) = cursor.next().await? {
            let sector_code = row.sector_code;
            let sector_name = row.sector_name;
            let avg_change_percent = row.avg_change_percent;
            let median_change_percent = row.median_change_percent;
            let total_volume = row.total_volume;
            let total_amount = row.total_amount;
            let stock_count = row.stock_count;
            let limit_up_count = row.limit_up_count;
            let limit_down_count = row.limit_down_count;
            let rise_count = row.rise_count;
            let fall_count = row.fall_count;
            let flat_count = row.flat_count;

            performances.push(SectorPerformance {
                sector_code,
                sector_name,
                avg_change_percent,
                median_change_percent,
                total_volume,
                total_amount,
                stock_count,
                limit_up_count,
                limit_down_count,
                rise_count,
                fall_count,
                flat_count,
            });
        }

        Ok(performances)
    }

    // 实时计算板块表现（基于当前行情）
    pub async fn calculate_sector_performance_realtime(
        &self,
        sector_code: &str,
    ) -> Result<SectorPerformance> {
        // 查询板块内所有股票的实时行情
        let stocks = self.get_sector_stocks_realtime(sector_code).await?;

        if stocks.is_empty() {
            return Ok(SectorPerformance {
                sector_code: sector_code.to_string(),
                sector_name: "未知".to_string(),
                avg_change_percent: 0.0,
                median_change_percent: 0.0,
                total_volume: 0.0,
                total_amount: 0.0,
                stock_count: 0,
                limit_up_count: 0,
                limit_down_count: 0,
                rise_count: 0,
                fall_count: 0,
                flat_count: 0,
            });
        }

        let stock_count = stocks.len() as i32;
        let total_amount: f64 = stocks.iter().map(|s| s.amount).sum();
        let total_volume: f64 = stocks.iter().map(|s| s.volume).sum();

        // 计算平均涨跌幅
        let avg_change_percent: f64 =
            stocks.iter().map(|s| s.change_percent).sum::<f64>() / stock_count as f64;

        // 计算中位数涨跌幅
        let mut changes: Vec<f64> = stocks.iter().map(|s| s.change_percent).collect();
        changes.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median_change_percent = if changes.len() % 2 == 0 {
            (changes[changes.len() / 2 - 1] + changes[changes.len() / 2]) / 2.0
        } else {
            changes[changes.len() / 2]
        };

        // 统计涨跌停和平盘数量
        let limit_up_count = stocks.iter().filter(|s| s.change_percent >= 9.8).count() as i32;
        let limit_down_count = stocks.iter().filter(|s| s.change_percent <= -9.8).count() as i32;
        let rise_count = stocks
            .iter()
            .filter(|s| s.change_percent > 0.0 && s.change_percent < 9.8)
            .count() as i32;
        let fall_count = stocks
            .iter()
            .filter(|s| s.change_percent < 0.0 && s.change_percent > -9.8)
            .count() as i32;
        let flat_count = stocks
            .iter()
            .filter(|s| (s.change_percent - 0.0).abs() < 0.01)
            .count() as i32;

        // 查询板块名称
        let name_query = format!(
            r#"
            SELECT DISTINCT sector_name
            FROM sector_stocks
            WHERE sector_code = '{}' AND date = today()
            LIMIT 1
        "#,
            sector_code
        );

        let mut name_cursor = self.client.query(&name_query).fetch::<SectorNameRow>()?;
        let sector_name = if let Some(row) = name_cursor.next().await? {
            row.sector_name
        } else {
            "未知".to_string()
        };

        Ok(SectorPerformance {
            sector_code: sector_code.to_string(),
            sector_name,
            avg_change_percent,
            median_change_percent,
            total_volume,
            total_amount,
            stock_count,
            limit_up_count,
            limit_down_count,
            rise_count,
            fall_count,
            flat_count,
        })
    }

    // ============================================
    // 功能4: 获取板块资金流向
    // ============================================
    pub async fn get_sector_flow(&self, sector_code: &str, date: &str) -> Result<SectorFlow> {
        // 资金流向 = 主力资金 + 散户资金
        // 简化算法：净流入 = 涨幅股票成交额 - 跌幅股票成交额
        let stocks = self.get_sector_stocks(sector_code, date).await?;

        let mut inflow = 0.0; // 流入（上涨股票成交额）
        let mut outflow = 0.0; // 流出（下跌股票成交额）

        for stock in &stocks {
            if stock.change_percent > 0.0 {
                inflow += stock.amount;
            } else if stock.change_percent < 0.0 {
                outflow += stock.amount;
            }
        }

        let net_inflow = inflow - outflow;

        // 简化：主力资金 = 大额成交（前20%的股票）
        let mut stocks_by_amount = stocks.clone();
        stocks_by_amount.sort_by(|a, b| b.amount.partial_cmp(&a.amount).unwrap());

        let top_20_percent = (stocks_by_amount.len() as f64 * 0.2).ceil() as usize;
        let main_inflow: f64 = stocks_by_amount
            .iter()
            .take(top_20_percent)
            .filter(|s| s.change_percent > 0.0)
            .map(|s| s.amount)
            .sum();

        let retail_inflow = inflow - main_inflow;

        // 查询板块名称
        let name_query = format!(
            r#"
            SELECT DISTINCT sector_name
            FROM sector_stocks
            WHERE sector_code = '{}' AND date = today()
            LIMIT 1
        "#,
            sector_code
        );

        let mut name_cursor = self.client.query(&name_query).fetch::<SectorNameRow>()?;
        let sector_name = if let Some(row) = name_cursor.next().await? {
            row.sector_name
        } else {
            "未知".to_string()
        };

        Ok(SectorFlow {
            sector_code: sector_code.to_string(),
            sector_name,
            inflow,
            outflow,
            net_inflow,
            main_inflow,
            retail_inflow,
        })
    }

    // 实时计算板块资金流向
    pub async fn calculate_sector_flow_realtime(&self, sector_code: &str) -> Result<SectorFlow> {
        self.get_sector_flow(sector_code, "today").await
    }

    // ============================================
    // 功能5: 批量计算所有板块表现
    // ============================================
    pub async fn calculate_all_sectors_performance(&self, date: &str) -> Result<usize> {
        // 获取所有板块代码
        let sectors_query = format!(
            r#"
            SELECT DISTINCT sector_code as stock_code
            FROM sector_stocks
            WHERE date = {}
        "#,
            if date == "today" || date.is_empty() {
                "today()"
            } else {
                date
            }
        );

        let mut sectors_cursor = self
            .client
            .query(&sectors_query)
            .fetch::<SectorStockCodeRow>()?;
        let mut sector_codes = Vec::new();

        while let Some(row) = sectors_cursor.next().await? {
            // 需要修改查询以匹配 SectorStockCodeRow（stock_code 字段）
            // 或者直接使用字符串
            let code = row.stock_code;
            sector_codes.push(code);
        }

        // 计算每个板块的表现
        let mut calculated_count = 0;

        for sector_code in &sector_codes {
            match self
                .calculate_sector_performance_realtime(sector_code)
                .await
            {
                Ok(_) => calculated_count += 1,
                Err(e) => {
                    eprintln!(
                        "Failed to calculate performance for sector {}: {}",
                        sector_code, e
                    );
                }
            }
        }

        Ok(calculated_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sector_performance_calculation() {
        // 测试板块表现计算逻辑
        let changes = vec![5.0, 3.0, 2.0, 1.0, 0.0, -1.0, -2.0, -3.0];
        let avg = changes.iter().map(|x| *x).sum::<f64>() / changes.len() as f64;

        assert_eq!(avg, 0.625); // (5+3+2+1+0-1-2-3)/8

        let mut sorted = changes.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median = (sorted[3] + sorted[4]) / 2.0;
        assert_eq!(median, 0.5); // (0+1)/2
    }

    #[test]
    fn test_flow_calculation() {
        // 测试资金流向计算
        let inflow = 1000000.0;
        let outflow = 800000.0;
        let net_inflow = inflow - outflow;

        assert_eq!(net_inflow, 200000.0);
    }
}
