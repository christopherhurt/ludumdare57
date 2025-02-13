use anyhow::Result;
use std::collections::{HashSet, VecDeque};
use std::default::Default;

use component::{Component, ComponentManager};
use entity::{Entity, EntityManager};
use system::{System, SystemManager};

pub mod entity;
pub mod component;
pub mod system;

pub(in crate::ecs) type Signature = u64;

pub struct SystemSignature(Signature);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProvisionalEntity(usize);

enum CommandType {
    CreateEntity,
    DestroyEntity,
    AttachComponent,
    AttachProvisionalComponent,
    DetachComponent,
    RegisterSystem,
    UnregisterSystem,
}

pub struct ECSCommands {
    command_order: VecDeque<CommandType>,
    provisional_entity_counter: usize,
    to_create: VecDeque<ProvisionalEntity>,
    to_destroy: VecDeque<Entity>,
    to_attach: VecDeque<(Entity, dyn Component)>, // TODO: u8?
    to_attach_provisional: VecDeque<(ProvisionalEntity, dyn Component)>, // TODO: u8?
    to_detach: VecDeque<(Entity, dyn Component)>, // TODO: u8?
    to_register: VecDeque<(System, HashSet<SystemSignature>, i16)>,
    to_unregister: VecDeque<System>,
}

const INITIAL_COMMAND_CAPACITY: usize = 16;

impl ECSCommands {
    fn new() -> Self {
        Self {
            command_order: VecDeque::with_capacity(INITIAL_COMMAND_CAPACITY),
            provisional_entity_counter: 0,
            to_create: VecDeque::with_capacity(INITIAL_COMMAND_CAPACITY),
            to_destroy: VecDeque::with_capacity(INITIAL_COMMAND_CAPACITY),
            to_attach: VecDeque::with_capacity(INITIAL_COMMAND_CAPACITY),
            to_attach_provisional: VecDeque::with_capacity(INITIAL_COMMAND_CAPACITY),
            to_detach: VecDeque::with_capacity(INITIAL_COMMAND_CAPACITY),
            to_register: VecDeque::with_capacity(INITIAL_COMMAND_CAPACITY),
            to_unregister: VecDeque::with_capacity(INITIAL_COMMAND_CAPACITY),
        }
    }

    pub fn create_entity(&mut self) -> Result<ProvisionalEntity> {
        let entity = ProvisionalEntity(self.provisional_entity_counter);

        self.provisional_entity_counter += 1;

        self.to_create.push_back(entity);

        self.command_order.push_back(CommandType::CreateEntity);

        Ok(entity)
    }

    pub fn destroy_entity(&mut self, entity: Entity) -> Result<()> {
        self.to_destroy.push_back(entity);

        self.command_order.push_back(CommandType::DestroyEntity);

        Ok(())
    }

    pub fn attach_component<T: Component>(&mut self, entity: Entity, component: T) -> Result<()> {
        // TODO
        Ok(())
    }

    // TODO: overload for placeholder entity

    pub fn detach_component<T: Component>(&mut self, entity: Entity) -> Result<()> {
        // TODO
        Ok(())
    }

    pub fn register_system(&mut self, system: System, signatures: HashSet<SystemSignature>, precedence: i16) -> Result<()> {
        self.to_register.push_back((system, signatures, precedence));

        self.command_order.push_back(CommandType::RegisterSystem);

        Ok(())
    }

    pub fn unregister_system(&mut self, system: System) -> Result<()> {
        self.to_unregister.push_back(system);

        self.command_order.push_back(CommandType::UnregisterSystem);

        Ok(())
    }
}

pub struct ECS {
    entity_manager: EntityManager,
    component_manager: ComponentManager,
    system_managers: Vec<SystemManager>,
    commands: ECSCommands,
}

const INTIIAL_SYSTEM_CAPACITY: usize = 256;

impl ECS {
    fn new(component_manager: ComponentManager, initial_entity_capacity: usize, max_entity_capacity: usize) -> Self {
        Self {
            entity_manager: EntityManager::new(initial_entity_capacity, max_entity_capacity),
            component_manager,
            system_managers: Vec::with_capacity(INTIIAL_SYSTEM_CAPACITY),
            commands: ECSCommands::new(),
        }
    }

    pub fn create_entity(&mut self) -> Result<ProvisionalEntity> {
        self.commands.create_entity()
    }

    pub fn destroy_entity(&mut self, entity: Entity) -> Result<()> {
        self.commands.destroy_entity(entity)
    }

    pub fn attach_component<T: Component>(&mut self, entity: Entity, component: T) -> Result<()> {
        // TODO
        Ok(())
    }

    // TODO: overload for placeholder entity

    pub fn detach_component<T: Component>(&mut self, entity: Entity) -> Result<()> {
        self.commands.detach_component(entity)
    }

    pub fn register_system(&mut self, system: System, signatures: HashSet<SystemSignature>, precedence: i16) -> Result<()> {
        self.commands.register_system(system, signatures, precedence)
    }

    pub fn unregister_system(&mut self, system: System) -> Result<()> {
        self.commands.unregister_system(system)
    }

    // TODO
    // - get system signature (pub) -> overload func for different num of components (up to 4 for now?), constant and inline?????, select the correct function with a macro rule

    pub(in crate) fn execute_systems(&mut self) {
        self.flush_all_commands();

        // TODO: flush system commands...before and after?
        self.flush_entity_component_commands(); // TODO: do between each

        self.flush_all_commands();
    }

    fn flush_entity_component_commands(&mut self) {
        // TODO

        self.commands.provisional_entity_counter = 0;
    }

    fn flush_all_commands(&mut self) {
        self.flush_entity_component_commands();

        // TODO: flush systems commands
    }
}

pub struct ECSBuilder {
    component_manager: ComponentManager,
    initial_entity_capacity: usize,
    max_entity_capacity: usize,
}

const DEFAULT_INITIAL_ENTITY_CAPACITY: usize = 1_024;
const DEFAULT_MAX_ENTITY_CAPACITY: usize = usize::MAX;

impl ECSBuilder {
    pub fn with_initial_entity_capacity(initial_entity_capacity: usize) -> Self {
        Self {
            component_manager: ComponentManager::new(initial_entity_capacity),
            initial_entity_capacity,
            max_entity_capacity: DEFAULT_MAX_ENTITY_CAPACITY,
        }
    }

    pub fn with_component<T: Component>(&mut self) -> &mut Self {
        self.component_manager.register_component::<T>(); // TODO: panic on error

        self
    }

    pub fn with_max_entity_capacity(&mut self, max_entity_capacity: usize) -> &mut Self {
        self.max_entity_capacity = max_entity_capacity;

        self
    }

    pub fn build(self) -> ECS {
        ECS::new(self.component_manager, self.initial_entity_capacity, self.max_entity_capacity)
    }
}

impl Default for ECSBuilder {
    fn default() -> Self {
        Self {
            component_manager: ComponentManager::new(DEFAULT_INITIAL_ENTITY_CAPACITY),
            initial_entity_capacity: DEFAULT_INITIAL_ENTITY_CAPACITY,
            max_entity_capacity: DEFAULT_MAX_ENTITY_CAPACITY,
        }
    }
}
