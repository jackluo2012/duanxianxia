use crate::domain::entities::models::{AuctionData, BacktestError, BacktestPeriod, DayData};
use chrono::NaiveDate;
use clickhouse::Client;
use clickhouse::Row;
use serde::Deserialize;

pub struct ClickHouseDataSource {
    client: Client,
}

impl ClickHouseDataSource {
    pub fn new(url: &str) -> Self {
        let client = Client::default().with_url(url);

        Self { client }
    }

    /// 加载回测期间的数据
    pub async fn load_backtest_data(
        &self,
        period: &BacktestPeriod,
    ) -> Result<Vec<DayData>, BacktestError> {
        let start_ts = period
            .start_date
            .and_hms_opt(9, 0, 0)
            .unwrap()
            .and_utc()
            .timestamp();
        let end_ts = period
            .end_date
            .and_hms_opt(15, 0, 0)
            .unwrap()
            .and_utc()
            .timestamp();

        // 查询竞价数据
        let auction_query = format!(
            "SELECT \
                toUInt64(toUnixTimestamp(timestamp)) as timestamp, \
                code, \
                name, \
                price, \
                change_percent, \
                buy_seal_amount, \
                sell_seal_amount, \
                strength_score, \
                open_price \
            FROM duanxianxia.auction_data \
            WHERE timestamp >= {} AND timestamp <= {} \
            ORDER BY timestamp, code",
            start_ts, end_ts
        );

        let auction_records: Vec<AuctionRecord> = self
            .client
            .query(&auction_query)
            .fetch_all()
            .await
            .map_err(|e| BacktestError::InternalError(e.to_string()))?;

        // 按日期分组
        let mut day_data_map: std::collections::HashMap<NaiveDate, Vec<AuctionData>> =
            std::collections::HashMap::new();

        for record in auction_records {
            let date = chrono::DateTime::from_timestamp(record.timestamp, 0)
                .unwrap()
                .naive_utc()
                .date();

            let auction_data = AuctionData {
                timestamp: record.timestamp,
                code: record.code,
                name: record.name,
                price: record.price,
                change_percent: record.change_percent,
                buy_seal_amount: record.buy_seal_amount,
                sell_seal_amount: record.sell_seal_amount,
                strength_score: record.strength_score,
                open_price: record.open_price,
            };

            day_data_map.entry(date).or_default().push(auction_data);
        }

        // 转换为 DayData 列表
        let mut result: Vec<DayData> = day_data_map
            .into_iter()
            .map(|(date, auction_data)| DayData {
                date,
                auction_data,
                stock_prices: std::collections::HashMap::new(),
            })
            .collect();

        result.sort_by_key(|d| d.date);
        Ok(result)
    }
}

#[derive(Row, Debug, Clone, Deserialize)]
struct AuctionRecord {
    timestamp: i64,
    code: String,
    name: String,
    price: f64,
    change_percent: f64,
    buy_seal_amount: f64,
    sell_seal_amount: f64,
    strength_score: i32,
    open_price: f64,
}
