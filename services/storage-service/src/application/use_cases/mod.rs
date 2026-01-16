//! Use Cases模块
//!
//! 用例定义了应用层可以执行的特定操作

pub mod store_quote;
pub mod query_history;

pub use store_quote::StoreQuoteUseCase;
pub use query_history::QueryHistoryUseCase;

#[cfg(test)]
mod tests;
