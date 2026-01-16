//! {{service_name}} - Domain Layer
//!
//! 领域层包含核心业务逻辑,独立于任何技术实现。

pub mod entities;
pub mod value_objects;
pub mod services;
pub mod ports;

pub use entities::*;
pub use value_objects::*;
pub use services::*;
pub use ports::*;
