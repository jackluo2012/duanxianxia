//! # 领域实体模块
//!
//! 实体是具有唯一标识的领域对象，具有生命周期和状态变化。

pub mod kline_data;
pub mod limit_up_event;
pub mod stock_quote;

// 重新导出实体类型
pub use kline_data::{KlineData, KlinePeriod};
pub use limit_up_event::LimitUpEvent;
pub use stock_quote::StockQuote;
