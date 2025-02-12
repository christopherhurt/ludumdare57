use anyhow::Result;

use component::ComponentManager;
use entity::EntityManager;
use system::SystemManager;

pub mod entity;
pub mod component;
pub mod system;

pub(in crate::ecs) const DEFAULT_INITIAL_ENTITY_CAPACITY: usize = 1_024;

pub(in crate::ecs) type Signature = u64;

pub struct SystemSignature(Signature);

pub struct ECSCommands {
    // TODO
}

impl ECSCommands {
    pub fn create_entity(&mut self) -> Result<()> {
        // TODO: gonna have to generate new entity values here for this until they can be inserted into the real entity manager.... is this worthy of new types e.g. PlaceholderEntity??
        Ok(())
    }

    pub fn destroy_entity(&mut self) -> Result<()> {
        // TODO
        Ok(())
    }

    pub fn attach_component(&mut self) -> Result<()> {
        // TODO
        Ok(())
    }

    pub fn detach_component(&mut self) -> Result<()> {
        // TODO
        Ok(())
    }

    pub fn register_system(&mut self) -> Result<()> {
        // TODO
        Ok(())
    }

    pub fn unregister_system(&mut self) -> Result<()> {
        // TODO
        Ok(())
    }
}

pub struct ECS {
    entity_manager: EntityManager,
    component_manager: ComponentManager,
    system_managers: Vec<SystemManager>,
    commands: ECSCommands,
}

impl ECS {
    // TODO
    // - register_system
    // - unregister_system
    // - constructor: takes all components as arg? builder? also allow selection of built in systems in builder?
    // - attach component
    // - detach component
    // - create entity
    // - destroy entity
    //
    // - get system signature -> overload func for different num of components (up to 4 for now?), constant and inline?????, select the correct function with a macro rule

    // Internal (crate):
    // - execute systems
    //      - flush commands (before each system execution)
}
