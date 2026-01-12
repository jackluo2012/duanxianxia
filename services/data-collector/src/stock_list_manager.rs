use crate::types::StockInfo;
use anyhow::Result;
use clickhouse::Client;
use clickhouse::insert::Insert;
use rustdx_complete::tcp::stock::SecurityList;
use rustdx_complete::tcp::{Tcp, Tdx};
use tracing::{debug, info, warn};

/// 股票列表管理器
pub struct StockListManager {
    ch_client: Client,
}

impl StockListManager {
    pub fn new(ch_client: Client) -> Self {
        Self { ch_client }
    }

    /// 从通达信获取全市场股票列表（深市+沪市）
    pub async fn fetch_stock_list(&self) -> Result<Vec<StockInfo>> {
        info!("正在从通达信获取全市场股票列表...");

        let mut tcp = Tcp::new()?;
        let mut all_stocks = Vec::new();

        // 获取深市股票列表 (market=0)
        info!("正在获取深市股票列表...");
        let sz_stocks = self.fetch_market_stocks(&mut tcp, 0).await?;
        info!("深市获取 {} 只股票", sz_stocks.len());
        all_stocks.extend(sz_stocks);

        // 获取沪市股票列表 (market=1)
        info!("正在获取沪市股票列表...");
        let sh_stocks = self.fetch_market_stocks(&mut tcp, 1).await?;
        info!("沪市获取 {} 只股票", sh_stocks.len());
        all_stocks.extend(sh_stocks);

        info!("全市场共获取 {} 只股票", all_stocks.len());
        Ok(all_stocks)
    }

    /// 获取单个市场的股票列表（支持分页）
    async fn fetch_market_stocks(&self, tcp: &mut Tcp, market: u16) -> Result<Vec<StockInfo>> {
        let mut all_stocks = Vec::new();
        let mut start = 0u16;
        const BATCH_SIZE: u16 = 1000; // 通达信每次最多返回1000只

        loop {
            let mut list = SecurityList::new(market, start);

            match list.recv_parsed(tcp) {
                Ok(_) => {
                    let stocks = list.result();
                    if stocks.is_empty() {
                        break;
                    }

                    // 转换为 StockInfo（仅保留A股，过滤基金、ETF、转债等）
                    let stock_infos: Vec<StockInfo> = stocks
                        .iter()
                        .filter_map(|s| {
                            if s.code.is_empty() || s.code.len() != 6 {
                                return None;
                            }

                            // 过滤非股票代码（基金、ETF、转债等）
                            let code = &s.code;
                            let is_valid_stock = match market {
                                0 => {
                                    // 深市：000xxx(主板), 001xxx(主板), 002xxx(中小板), 003xxx(主板), 300xxx(创业板)
                                    code.starts_with("000") || code.starts_with("001")
                                        || code.starts_with("002") || code.starts_with("003")
                                        || code.starts_with("300")
                                }
                                1 => {
                                    // 沪市：600xxx, 601xxx, 603xxx, 605xxx(主板), 688xxx, 689xxx(科创板)
                                    code.starts_with("600") || code.starts_with("601")
                                        || code.starts_with("603") || code.starts_with("605")
                                        || code.starts_with("688") || code.starts_with("689")
                                }
                                _ => false,
                            };

                            if !is_valid_stock {
                                return None;
                            }

                            let list_date = chrono::Utc::now().format("%Y-%m-%d").to_string();

                            Some(StockInfo {
                                code: s.code.clone(),
                                name: s.name.clone(),
                                market: market as u8,
                                list_date,
                                status: "active".to_string(),
                            })
                        })
                        .collect();

                    all_stocks.extend(stock_infos);

                    // 如果返回的股票数量少于 BATCH_SIZE，说明已经获取完毕
                    if stocks.len() < BATCH_SIZE as usize {
                        break;
                    }

                    start += BATCH_SIZE;
                }
                Err(e) => {
                    warn!(
                        "获取市场 {} 的股票列表失败 (start={}): {}",
                        market, start, e
                    );
                    break;
                }
            }
        }

        Ok(all_stocks)
    }

    /// 将股票列表持久化到 ClickHouse（分批写入）
    pub async fn update_stock_list(&self, stocks: &[StockInfo]) -> Result<()> {
        if stocks.is_empty() {
            warn!("股票列表为空，跳过更新");
            return Ok(());
        }

        info!("正在将 {} 只股票写入 ClickHouse...", stocks.len());

        // 分批写入，每批 1000 只
        let batch_size = 1000;
        let batches = stocks.chunks(batch_size);
        let mut total_written = 0usize;

        for (i, batch) in batches.enumerate() {
            let mut insert: Insert<StockInfo> = self
                .ch_client
                .insert("stock_list").await?;

            for stock in batch {
                insert.write(stock).await?;
            }

            insert.end().await?;
            total_written += batch.len();

            info!(
                "第 {}/{} 批写入成功：{} 只股票",
                i + 1,
                (stocks.len() + batch_size - 1) / batch_size,
                batch.len()
            );
        }

        info!("成功写入 {} 只股票到 ClickHouse", total_written);
        Ok(())
    }

    /// 将股票分组，每批 batch_size 只
    pub fn group_stocks(&self, stocks: &[StockInfo], batch_size: usize) -> Vec<Vec<StockInfo>> {
        let batches: Vec<Vec<StockInfo>> = stocks.chunks(batch_size).map(|s| s.to_vec()).collect();
        debug!(
            "将 {} 只股票分成 {} 批，每批最多 {} 只",
            stocks.len(),
            batches.len(),
            batch_size
        );
        batches
    }

    /// 获取并更新股票列表（完整流程）
    pub async fn fetch_and_update(&self, batch_size: usize) -> Result<Vec<Vec<StockInfo>>> {
        let stocks = self.fetch_stock_list().await?;
        self.update_stock_list(&stocks).await?;
        let batches = self.group_stocks(&stocks, batch_size);
        Ok(batches)
    }
}
