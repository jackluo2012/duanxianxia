//! # 领域服务模块
//!
//! 领域服务封装不属于特定实体或值对象的业务逻辑。

pub mod kline_aggregator;
pub mod limit_up_detector;
pub mod quote_collector;

// 重新导出服务和trait
pub use kline_aggregator::{AggregationError, DefaultKlineAggregator, KlineAggregator};
pub use limit_up_detector::{DefaultLimitUpDetector, DetectionError, LimitUpDetector};
pub use quote_collector::{
    CollectionError, DefaultQuoteCollector, QuoteCollector as DomainQuoteCollector,
};
