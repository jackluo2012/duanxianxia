//! 值对象模块
//!
//! 值对象是不可变的,通过其属性值来标识的对象。

pub mod entity_id;

// 导出所有值对象
pub use entity_id::EntityId;
