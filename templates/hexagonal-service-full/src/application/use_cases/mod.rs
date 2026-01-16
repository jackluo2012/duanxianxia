//! 用例模块
//!
//! 用例定义了应用层可以执行的特定操作。

pub mod create_entity;
pub mod get_entity;
pub mod update_entity;

pub use create_entity::CreateEntityUseCase;
pub use get_entity::GetEntityUseCase;
pub use update_entity::UpdateEntityUseCase;
