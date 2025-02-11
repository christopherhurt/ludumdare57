pub mod entity;
pub mod component;
pub mod system;

pub(in crate::ecs) const DEFAULT_INITIAL_ENTITY_CAPACITY: usize = 1_024;

pub(in crate::ecs) type Signature = u128;
