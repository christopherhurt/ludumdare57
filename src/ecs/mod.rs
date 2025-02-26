use anyhow::{anyhow, Error, Result};
use std::any::TypeId;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::default::Default;

use component::{Component, ComponentManager};
use entity::{Entity, EntityManager};
use system::{System, SystemManager};

pub mod entity;
pub mod component;
pub mod system;

pub(in crate::ecs) type Signature = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SystemSignature(Signature);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProvisionalEntity(usize);

enum EntityComponentCommandType {
    CreateEntity,
    DestroyEntity,
    AttachComponent,
    AttachProvisionalComponent,
    DetachComponent,
}

enum SystemCommandType {
    RegisterSystem,
    UnregisterSystem,
    Shutdown,
}

pub struct ECSCommands {
    entity_component_command_order: VecDeque<EntityComponentCommandType>,
    system_command_order: VecDeque<SystemCommandType>,
    provisional_entity_counter: usize,
    to_create: VecDeque<ProvisionalEntity>,
    to_destroy: VecDeque<Entity>,
    to_attach: VecDeque<(Entity, TypeId, Box<[u8]>)>,
    to_attach_provisional: VecDeque<(ProvisionalEntity, TypeId, Box<[u8]>)>,
    to_detach: VecDeque<(Entity, TypeId)>,
    to_register: VecDeque<(System, HashSet<SystemSignature>, i16)>,
    to_unregister: VecDeque<System>,
    to_shutdown: bool,
}

const INITIAL_COMMAND_CAPACITY: usize = 16;

impl ECSCommands {
    fn new() -> Self {
        Self {
            entity_component_command_order: VecDeque::with_capacity(INITIAL_COMMAND_CAPACITY),
            system_command_order: VecDeque::with_capacity(INITIAL_COMMAND_CAPACITY),
            provisional_entity_counter: 0,
            to_create: VecDeque::with_capacity(INITIAL_COMMAND_CAPACITY),
            to_destroy: VecDeque::with_capacity(INITIAL_COMMAND_CAPACITY),
            to_attach: VecDeque::with_capacity(INITIAL_COMMAND_CAPACITY),
            to_attach_provisional: VecDeque::with_capacity(INITIAL_COMMAND_CAPACITY),
            to_detach: VecDeque::with_capacity(INITIAL_COMMAND_CAPACITY),
            to_register: VecDeque::with_capacity(INITIAL_COMMAND_CAPACITY),
            to_unregister: VecDeque::with_capacity(INITIAL_COMMAND_CAPACITY),
            to_shutdown: false,
        }
    }

    pub fn create_entity(&mut self) -> ProvisionalEntity {
        let entity = ProvisionalEntity(self.provisional_entity_counter);

        self.provisional_entity_counter += 1;

        self.to_create.push_back(entity);
        self.entity_component_command_order.push_back(EntityComponentCommandType::CreateEntity);

        entity
    }

    pub fn destroy_entity(&mut self, entity: Entity) {
        self.to_destroy.push_back(entity);
        self.entity_component_command_order.push_back(EntityComponentCommandType::DestroyEntity);
    }

    pub fn attach_component<T: Component>(&mut self, entity: Entity, component: T) {
        let raw_comp_data = component_to_boxed_slice(component);

        self.to_attach.push_back((entity, TypeId::of::<T>(), raw_comp_data));
        self.entity_component_command_order.push_back(EntityComponentCommandType::AttachComponent);
    }

    pub fn attach_provisional_component<T: Component>(&mut self, provisional_entity: ProvisionalEntity, component: T) {
        let raw_comp_data = component_to_boxed_slice(component);

        self.to_attach_provisional.push_back((provisional_entity, TypeId::of::<T>(), raw_comp_data));
        self.entity_component_command_order.push_back(EntityComponentCommandType::AttachProvisionalComponent);
    }

    pub fn detach_component<T: Component>(&mut self, entity: Entity) {
        self.to_detach.push_back((entity, TypeId::of::<T>()));
        self.entity_component_command_order.push_back(EntityComponentCommandType::DetachComponent);
    }

    pub fn register_system(&mut self, system: System, signatures: HashSet<SystemSignature>, precedence: i16) {
        self.to_register.push_back((system, signatures, precedence));
        self.system_command_order.push_back(SystemCommandType::RegisterSystem);
    }

    pub fn unregister_system(&mut self, system: System) {
        self.to_unregister.push_back(system);
        self.system_command_order.push_back(SystemCommandType::UnregisterSystem);
    }

    pub fn shutdown(&mut self) {
        self.to_shutdown = true;
        self.system_command_order.push_back(SystemCommandType::Shutdown);
    }
}

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
    system_managers: Vec<RefCell<SystemManager>>,
    system_hashes: HashSet<System>,
    commands: ECSCommands,
    initial_entity_capacity: usize,
    is_shutdown: bool,
}

const INTIIAL_SYSTEM_CAPACITY: usize = 256;

impl ECS {
    fn new(component_manager: ComponentManager, initial_entity_capacity: usize, max_entity_capacity: usize) -> Self {
        Self {
            entity_manager: EntityManager::new(initial_entity_capacity, max_entity_capacity),
            component_manager,
            system_managers: Vec::with_capacity(INTIIAL_SYSTEM_CAPACITY),
            system_hashes: HashSet::with_capacity(INTIIAL_SYSTEM_CAPACITY),
            commands: ECSCommands::new(),
            initial_entity_capacity,
            is_shutdown: false,
        }
    }

    pub fn create_entity(&mut self) -> ProvisionalEntity {
        self.commands.create_entity()
    }

    pub fn destroy_entity(&mut self, entity: Entity) {
        self.commands.destroy_entity(entity);
    }

    pub fn attach_component<T: Component>(&mut self, entity: Entity, component: T) {
        self.commands.attach_component(entity, component);
    }

    pub fn attach_provisional_component<T: Component>(&mut self, provisional_entity: ProvisionalEntity, component: T) {
        self.commands.attach_provisional_component(provisional_entity, component);
    }

    pub fn detach_component<T: Component>(&mut self, entity: Entity) {
        self.commands.detach_component::<T>(entity);
    }

    pub fn register_system(&mut self, system: System, signatures: HashSet<SystemSignature>, precedence: i16) {
        self.commands.register_system(system, signatures, precedence);
    }

    pub fn unregister_system(&mut self, system: System) {
        self.commands.unregister_system(system);
    }

    pub fn shutdown(&mut self) {
        self.commands.shutdown();
    }

    pub fn get_system_signature_0(&self) -> Result<SystemSignature> {
        Ok(SystemSignature(0))
    }

    pub fn get_system_signature_1<A: Component>(&self) -> Result<SystemSignature> {
        let sig = self.component_manager.get_signature(TypeId::of::<A>())?;

        Ok(SystemSignature(sig))
    }

    pub fn get_system_signature_2<A: Component, B: Component>(&self) -> Result<SystemSignature> {
        let sig_a = self.component_manager.get_signature(TypeId::of::<A>())?;
        let sig_b = self.component_manager.get_signature(TypeId::of::<B>())?;

        Ok(SystemSignature(sig_a | sig_b))
    }

    pub fn get_system_signature_3<A: Component, B: Component, C: Component>(&self) -> Result<SystemSignature> {
        let sig_a = self.component_manager.get_signature(TypeId::of::<A>())?;
        let sig_b = self.component_manager.get_signature(TypeId::of::<B>())?;
        let sig_c = self.component_manager.get_signature(TypeId::of::<C>())?;

        Ok(SystemSignature(sig_a | sig_b | sig_c))
    }

    pub fn get_system_signature_4<A: Component, B: Component, C: Component, D: Component>(&self) -> Result<SystemSignature> {
        let sig_a = self.component_manager.get_signature(TypeId::of::<A>())?;
        let sig_b = self.component_manager.get_signature(TypeId::of::<B>())?;
        let sig_c = self.component_manager.get_signature(TypeId::of::<C>())?;
        let sig_d = self.component_manager.get_signature(TypeId::of::<D>())?;

        Ok(SystemSignature(sig_a | sig_b | sig_c | sig_d))
    }

    pub(in crate) fn invoke_systems(&mut self) -> bool {
        // TODO: There's definitely a better way to handle errors here than always panicking... we could maybe add user configurable error handling and then crash the
        //  application, print a warning, etc. depending on the error. Also, considering using more specific Error types here rather than anyhow. Also, do we want
        //  to continue flushing remaining commands even when one fails?

        if self.is_shutdown {
            return false;
        }

        flush_all_commands(&mut self.commands, &mut self.entity_manager, &mut self.component_manager, &mut self.system_managers, &mut self.system_hashes, self.initial_entity_capacity, &mut self.is_shutdown)
            .unwrap_or_else(|e| panic!("{}", e));

        if self.is_shutdown {
            return false;
        }

        self.system_managers.iter().for_each(|manager| {
            manager.borrow_mut().invoke_system(&mut self.component_manager, &mut self.commands);
            flush_entity_component_commands(&mut self.commands, &mut self.entity_manager, &mut self.component_manager, &self.system_managers);
        });

        flush_all_commands(&mut self.commands, &mut self.entity_manager, &mut self.component_manager, &mut self.system_managers, &mut self.system_hashes, self.initial_entity_capacity, &mut self.is_shutdown)
            .unwrap_or_else(|e| panic!("{}", e));

        true
    }

    pub(in crate) fn is_shutdown(&self) -> bool {
        self.is_shutdown
    }
}

fn flush_entity_component_commands(
    commands: &mut ECSCommands,
    entity_manager: &mut EntityManager,
    component_manager: &mut ComponentManager,
    system_managers: &Vec<RefCell<SystemManager>>,
) -> Result<()> {
    let mut provisional_entity_map: HashMap<ProvisionalEntity, Entity> = HashMap::with_capacity(commands.to_create.len());

    while let Some(command_type) = commands.entity_component_command_order.pop_front() {
        match command_type {
            EntityComponentCommandType::CreateEntity => {
                let provisional_entity = commands.to_create.pop_front().unwrap_or_else(|| panic!("Internal error: expected a provisional entity to create"));

                let entity = entity_manager.create_entity()?;

                system_managers.iter().map(|manager| {
                    let updated_signature = entity_manager.get_signature(entity)?;
                    manager.borrow_mut().handle_entity_updated(entity, updated_signature);

                    Ok::<_, Error>(())
                }).find(|r| r.is_err()).unwrap_or(Ok(()))?;

                provisional_entity_map.insert(provisional_entity, entity);
            },
            EntityComponentCommandType::DestroyEntity => {
                let entity = commands.to_destroy.pop_front().unwrap_or_else(|| panic!("Internal error: expected an entity to destroy"));

                component_manager.handle_entity_removed(entity);
                system_managers.iter().for_each(|manager| manager.borrow_mut().handle_entity_removed(entity));

                entity_manager.destroy_entity(entity)?;
            },
            EntityComponentCommandType::AttachComponent => {
                let (entity, type_id, comp_data) = commands.to_attach.pop_front().unwrap_or_else(|| panic!("Internal error: expected a component to attach"));

                component_manager.attach_component(entity, type_id, comp_data)?;

                let component_signature = component_manager.get_signature(type_id)?;
                apply_entity_signature_update(entity, component_signature, entity_manager, system_managers)?;
            },
            EntityComponentCommandType::AttachProvisionalComponent => {
                let (provisional_entity, type_id, comp_data) = commands.to_attach_provisional.pop_front().unwrap_or_else(|| panic!("Internal error: expected a component to attach"));

                let entity = *provisional_entity_map.get(&provisional_entity).unwrap_or_else(|| panic!("Internal error: provisional entity {:?} was not created before attaching a component to it", provisional_entity));

                component_manager.attach_component(entity, type_id, comp_data)?;

                let component_signature = component_manager.get_signature(type_id)?;
                apply_entity_signature_update(entity, component_signature, entity_manager, system_managers)?;
            },
            EntityComponentCommandType::DetachComponent => {
                let (entity, type_id) = commands.to_detach.pop_front().unwrap_or_else(|| panic!("Internal error: expected a component to detach"));

                component_manager.detach_component(entity, type_id)?;

                let component_signature = component_manager.get_signature(type_id)?;
                apply_entity_signature_update(entity, component_signature, entity_manager, system_managers)?;
            },
        }
    }

    commands.provisional_entity_counter = 0;

    #[cfg(debug_assertions)] {
        if !commands.to_create.is_empty() {
            panic!("Internal error: to_create was not drained")
        }
        if !commands.to_destroy.is_empty() {
            panic!("Internal error: to_destroy was not drained")
        }
        if !commands.to_attach.is_empty() {
            panic!("Internal error: to_attach was not drained")
        }
        if !commands.to_attach_provisional.is_empty() {
            panic!("Internal error: to_attach_provisional was not drained")
        }
        if !commands.to_detach.is_empty() {
            panic!("Internal error: to_detach was not drained")
        }
    }

    Ok(())
}

fn apply_entity_signature_update(
    entity: Entity,
    component_signature: Signature,
    entity_manager: &mut EntityManager,
    system_managers: &Vec<RefCell<SystemManager>>,
) -> Result<()> {
    let mut entity_signature = entity_manager.get_signature(entity)?;
    entity_signature |= component_signature;

    entity_manager.set_signature(entity, entity_signature)?;
    system_managers.iter().for_each(|manager| manager.borrow_mut().handle_entity_updated(entity, entity_signature));

    Ok(())
}

fn flush_all_commands(
    commands: &mut ECSCommands,
    entity_manager: &mut EntityManager,
    component_manager: &mut ComponentManager,
    system_managers: &mut Vec<RefCell<SystemManager>>,
    system_hashes: &mut HashSet<System>,
    initial_entity_capacity: usize,
    is_shutdown: &mut bool,
) -> Result<()> {
    flush_entity_component_commands(commands, entity_manager, component_manager, system_managers);

    while let Some(command_type) = commands.system_command_order.pop_front() {
        match command_type {
            SystemCommandType::RegisterSystem => {
                let (system, system_signatures, precedence) = commands.to_register.pop_front().unwrap_or_else(|| panic!("Internal error: expected a system to register"));

                if system_hashes.contains(&system) {
                    return Err(anyhow!("System is already registered"));
                }

                let raw_signatures = system_signatures.iter().map(|sig| sig.0).collect();

                let system_manager = SystemManager::new(system, raw_signatures, precedence, initial_entity_capacity);

                system_managers.push(RefCell::new(system_manager));
                system_managers.sort_by_key(|c| c.borrow().precedence);

                system_hashes.insert(system);
            },
            SystemCommandType::UnregisterSystem => {
                let system = commands.to_unregister.pop_front().unwrap_or_else(|| panic!("Internal error: expected a system to unregister"));

                if !system_hashes.remove(&system) {
                    return Err(anyhow!("System is not registered"));
                }

                system_managers.retain(|m| m.borrow().system != system);
            },
            SystemCommandType::Shutdown => {
                *is_shutdown = true;
            },
        }
    }

    #[cfg(debug_assertions)] {
        if !commands.to_register.is_empty() {
            panic!("Internal error: to_register was not drained")
        }
        if !commands.to_unregister.is_empty() {
            panic!("Internal error: to_unregister was not drained")
        }
    }

    Ok(())
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

    pub fn with_component<T: Component>(mut self) -> Self {
        self.component_manager.register_component::<T>().unwrap_or_else(|e| panic!("{}", e));

        self
    }

    pub fn with_max_entity_capacity(mut self, max_entity_capacity: usize) -> Self {
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
