use anyhow::Result;
use std::any::TypeId;
use std::collections::{HashSet, VecDeque};
use std::default::Default;

use component::{Component, ComponentManager};
use entity::{Entity, EntityManager};
use system::{System, SystemManager};

pub mod entity;
pub mod component;
pub mod system;

pub(in crate::ecs) type Signature = u64;

pub(in crate::ecs) const DEFAULT_ENTITY_SIGNATURE: Signature = 0;

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
    to_attach: VecDeque<(Entity, TypeId, Box<[u8]>)>,
    to_attach_provisional: VecDeque<(ProvisionalEntity, TypeId, Box<[u8]>)>,
    to_detach: VecDeque<(Entity, TypeId)>,
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

    pub fn create_entity(&mut self) -> ProvisionalEntity {
        let entity = ProvisionalEntity(self.provisional_entity_counter);

        self.provisional_entity_counter += 1;

        self.to_create.push_back(entity);
        self.command_order.push_back(CommandType::CreateEntity);

        entity
    }

    pub fn destroy_entity(&mut self, entity: Entity) {
        self.to_destroy.push_back(entity);
        self.command_order.push_back(CommandType::DestroyEntity);
    }

    pub fn attach_component<T: Component>(&mut self, entity: Entity, component: T) {
        let raw_comp_data = component_to_boxed_slice(component);

        self.to_attach.push_back((entity, TypeId::of::<T>(), raw_comp_data));
        self.command_order.push_back(CommandType::AttachComponent);
    }

    pub fn attach_provisional_component<T: Component>(&mut self, provisional_entity: ProvisionalEntity, component: T) {
        let raw_comp_data = component_to_boxed_slice(component);

        self.to_attach_provisional.push_back((provisional_entity, TypeId::of::<T>(), raw_comp_data));
        self.command_order.push_back(CommandType::AttachProvisionalComponent);
    }

    pub fn detach_component<T: Component>(&mut self, entity: Entity) {
        self.to_detach.push_back((entity, TypeId::of::<T>()));
        self.command_order.push_back(CommandType::DetachComponent);
    }

    pub fn register_system(&mut self, system: System, signatures: HashSet<SystemSignature>, precedence: i16) {
        self.to_register.push_back((system, signatures, precedence));
        self.command_order.push_back(CommandType::RegisterSystem);
    }

    pub fn unregister_system(&mut self, system: System) {
        self.to_unregister.push_back(system);
        self.command_order.push_back(CommandType::UnregisterSystem);
    }
}

#[inline]
fn component_to_boxed_slice<T: Component>(component: T) -> Box<[u8]> {
    let comp_size = size_of::<T>();

    unsafe {
        let ptr = &component as *const T as *const u8;
        let raw_slice = std::slice::from_raw_parts(ptr, comp_size);
        Box::from(raw_slice)
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

    pub fn create_entity(&mut self) -> ProvisionalEntity {
        self.commands.create_entity()
    }

    pub fn destroy_entity(&mut self, entity: Entity) {
        self.commands.destroy_entity(entity)
    }

    pub fn attach_component<T: Component>(&mut self, entity: Entity, component: T) {
        self.commands.attach_component(entity, component)
    }

    pub fn attach_provisional_component<T: Component>(&mut self, provisional_entity: ProvisionalEntity, component: T) {
        self.commands.attach_provisional_component(provisional_entity, component)
    }

    pub fn detach_component<T: Component>(&mut self, entity: Entity) {
        self.commands.detach_component::<T>(entity)
    }

    pub fn register_system(&mut self, system: System, signatures: HashSet<SystemSignature>, precedence: i16) {
        self.commands.register_system(system, signatures, precedence)
    }

    pub fn unregister_system(&mut self, system: System) {
        self.commands.unregister_system(system)
    }

    pub fn get_system_signature_0(&self) -> Result<SystemSignature> {
        Ok(SystemSignature(DEFAULT_ENTITY_SIGNATURE))
    }

    pub fn get_system_signature_1<A: Component>(&self) -> Result<SystemSignature> {
        let sig = self.component_manager.get_signature::<A>()?;

        Ok(SystemSignature(sig))
    }

    pub fn get_system_signature_2<A: Component, B: Component>(&self) -> Result<SystemSignature> {
        let sig_a = self.component_manager.get_signature::<A>()?;
        let sig_b = self.component_manager.get_signature::<B>()?;

        Ok(SystemSignature(sig_a | sig_b))
    }

    pub fn get_system_signature_3<A: Component, B: Component, C: Component>(&self) -> Result<SystemSignature> {
        let sig_a = self.component_manager.get_signature::<A>()?;
        let sig_b = self.component_manager.get_signature::<B>()?;
        let sig_c = self.component_manager.get_signature::<C>()?;

        Ok(SystemSignature(sig_a | sig_b | sig_c))
    }

    pub fn get_system_signature_4<A: Component, B: Component, C: Component, D: Component>(&self) -> Result<SystemSignature> {
        let sig_a = self.component_manager.get_signature::<A>()?;
        let sig_b = self.component_manager.get_signature::<B>()?;
        let sig_c = self.component_manager.get_signature::<C>()?;
        let sig_d = self.component_manager.get_signature::<D>()?;

        Ok(SystemSignature(sig_a | sig_b | sig_c | sig_d))
    }

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
        self.component_manager.register_component::<T>().unwrap_or_else(|e| panic!("{}", e));

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
