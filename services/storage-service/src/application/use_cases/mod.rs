//! Use Cases模块
//!
//! 用例定义了应用层可以执行的特定操作

pub mod query_history;
pub mod query_realtime;
pub mod store_quote;

pub use query_history::QueryHistoryUseCase;
pub use query_realtime::QueryRealtimeUseCase;
pub use store_quote::StoreQuoteUseCase;

#[cfg(test)]
mod tests;
