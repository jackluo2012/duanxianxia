//! Use Cases模块
//!
//! 用例定义了应用层可以执行的特定操作

pub mod query_history;
pub mod store_quote;

pub use query_history::QueryHistoryUseCase;
pub use store_quote::StoreQuoteUseCase;

#[cfg(test)]
mod tests;
