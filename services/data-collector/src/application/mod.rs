//! Application Layer Module
//!
//! Contains application services that orchestrate domain logic and adapters

pub mod quote_collection_service;
pub mod orchestrator;

pub use quote_collection_service::ApplicationQuoteCollectionService;
pub use orchestrator::QuoteCollectionOrchestrator;
