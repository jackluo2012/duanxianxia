//! Event Publisher Trait
//!
//! Secondary Port - Event publishing interface

use async_trait::async_trait;
use serde::Serialize;
use std::error::Error;
use std::fmt;

/// Publish Error
#[derive(Debug, Clone, PartialEq)]
pub enum PublishError {
    Connection(String),
    Serialization(String),
    Timeout(String),
}

impl fmt::Display for PublishError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PublishError::Connection(msg) => write!(f, "Connection error: {}", msg),
            PublishError::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            PublishError::Timeout(msg) => write!(f, "Timeout error: {}", msg),
        }
    }
}

impl Error for PublishError {}

/// Event Publisher Trait
///
/// This trait defines the interface for publishing events to message queues.
/// Implementations can use Redis, Kafka, RabbitMQ, etc.
#[async_trait]
pub trait EventPublisher: Send + Sync {
    /// Publish an event to a topic
    async fn publish<T>(&self, topic: &str, event: &T) -> Result<(), PublishError>
    where
        T: Serialize + Send + Sync;
}
