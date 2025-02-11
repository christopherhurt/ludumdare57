use anyhow::{anyhow, Result};
use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet, VecDeque};
use std::collections::hash_set::Iter;

/////////////////////////////////////////////////////////////////////////////
/// Public
/////////////////////////////////////////////////////////////////////////////

pub type Entity = usize;
pub type Signature = u32;
pub type SystemId = u8;

pub trait Component: 'static {}

pub struct ECS {
    entity_manager: EntityManager,
    component_types_to_arrays: HashMap<TypeId, ComponentArray<Box<dyn Any>>>,
    system_ids_to_managers: HashMap<SystemId, SystemManager>,
    component_registration_bit: Signature,
}

impl ECS {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn create_entity(&mut self) -> Entity {
        let new_entity = self.entity_manager.create_entity();
        let new_entity_signature = self.entity_manager.get_signature(new_entity).unwrap_or_else(|_| {
            panic!("Internal error: Failed to get signature for newly created entity {}", new_entity);
        });

        self.system_ids_to_managers.values_mut().for_each(|s| s.handle_entity_updated(new_entity, new_entity_signature));

        new_entity
    }

    pub fn destroy_entity(&mut self, entity: Entity) -> Result<()> {
        self.entity_manager.destroy_entity(entity)?;
        self.component_types_to_arrays.values_mut().for_each(|comp_arr| {
            comp_arr.remove_component(entity).unwrap_or_else(|_| {
                panic!("Internal error: Remove component failed for entity {} which should exist", entity);
            });
        });
        self.system_ids_to_managers.values_mut().for_each(|s: &mut SystemManager| s.handle_entity_removed(entity));

        Ok(())
    }

    pub fn attach_component<T: Component>(&mut self, entity: Entity, component: Box<T>) -> Result<()> {
        let comp_arr = self.component_types_to_arrays.get_mut(&TypeId::of::<T>()).map(|c| Ok(c))
            .unwrap_or(Err(anyhow!("Cannot attach component which isn't registered")))?;
        let mut entity_signature = self.entity_manager.get_signature(entity)?;

        comp_arr.insert_component(entity, component)?;

        entity_signature |= comp_arr.component_signature;
        self.entity_manager.set_signature(entity, entity_signature).unwrap_or_else(|_|
            panic!("Internal error: Failed to set signature for entity {} which should exist", entity));

        self.system_ids_to_managers.values_mut().for_each(|m| m.handle_entity_updated(entity, entity_signature));

        Ok(())
    }

    pub fn remove_component<T: Component>(&mut self, entity: Entity) -> Result<()> {
        let comp_arr = self.component_types_to_arrays.get_mut(&TypeId::of::<T>()).map(|c| Ok(c))
            .unwrap_or(Err(anyhow!("Cannot remove component which isn't registered")))?;
        let mut entity_signature = self.entity_manager.get_signature(entity)?;

        comp_arr.remove_component(entity)?;

        entity_signature |= comp_arr.component_signature;
        self.entity_manager.set_signature(entity, entity_signature).unwrap_or_else(|_|
            panic!("Internal error: Failed to set signature for entity {} which should exist", entity));

        self.system_ids_to_managers.values_mut().for_each(|m| m.handle_entity_updated(entity, entity_signature));

        Ok(())
    }

    pub fn get_component<T: Component>(&self, entity: Entity) -> Result<&T> {
        match self.component_types_to_arrays.get(&TypeId::of::<T>()) {
            Some(comp_arr) => {
                comp_arr.get_component(entity).map(|b| b.downcast_ref::<T>().unwrap_or_else(||
                    panic!("Internal error: component_types_to_arrays contains a mismatching key and value")))
            },
            None => Err(anyhow!("No such component has been registered")),
        }
    }

    pub fn get_mut_component<T: Component>(&mut self, entity: Entity) -> Result<&mut T> {
        match self.component_types_to_arrays.get_mut(&TypeId::of::<T>()) {
            Some(comp_arr) => {
                comp_arr.get_mut_component(entity).map(|b| b.downcast_mut::<T>().unwrap_or_else(||
                    panic!("Internal error: component_types_to_arrays contains a mismatching key and value")))
            },
            None => Err(anyhow!("No such component has been registered")),
        }
    }

    pub fn get_system_entities(&self, system_id: SystemId) -> Result<Iter<Entity>> {
        let system = self.system_ids_to_managers.get(&system_id).map(|s| Ok(s))
            .unwrap_or(Err(anyhow!("Cannot get entities for system which isn't registered")))?;

        Ok(system.entities.iter())
    }

    pub fn register_component<T: Component>(&mut self) -> Result<Signature> {
        let type_id = TypeId::of::<T>();

        if self.component_types_to_arrays.contains_key(&type_id) {
            return Err(anyhow!("Component is already registered"));
        }

        if self.component_registration_bit == 0 {
            return Err(anyhow!("Too many components are already registered"));
        }

        let component_signature = self.component_registration_bit;
        self.component_registration_bit <<= 1;
        self.component_types_to_arrays.insert(type_id, ComponentArray::new(component_signature));

        Ok(component_signature)
    }

    pub fn register_system(&mut self, system_id: SystemId, signatures: HashSet<Signature>) -> Result<()> {
        if self.system_ids_to_managers.contains_key(&system_id) {
            return Err(anyhow!("System is already registered"));
        }

        let mut system_manager = SystemManager::new(signatures);
        self.entity_manager.entity_destroyed.iter().enumerate().for_each(|(entity, is_destroyed)| if !is_destroyed {
            let entity_signature = self.entity_manager.get_signature(entity).unwrap_or_else(|_|
                panic!("Internal error: Could not get signature for valid entity {}", entity));
            system_manager.handle_entity_updated(entity, entity_signature);
        });

        self.system_ids_to_managers.insert(system_id, system_manager);

        Ok(())
    }
}

impl Default for ECS {
    fn default() -> Self {
        Self {
            entity_manager: EntityManager::default(),
            component_types_to_arrays: HashMap::default(),
            system_ids_to_managers: HashMap::default(),
            component_registration_bit: 1,
        }
    }
}

/////////////////////////////////////////////////////////////////////////////
/// ComponentArray
/////////////////////////////////////////////////////////////////////////////

const INVALID_INDEX: usize = usize::MAX;

struct ComponentArray<T> {
    component_signature: Signature,
    entity_to_index: Vec<usize>,
    index_to_entity: Vec<Entity>,
    components: Vec<T>,
}

impl<T> ComponentArray<T> {
    fn new(component_signature: Signature) -> Self {
        Self {
            component_signature,
            entity_to_index: vec![INVALID_INDEX; INITIAL_CAPACITY],
            index_to_entity: Vec::with_capacity(INITIAL_CAPACITY),
            components: Vec::with_capacity(INITIAL_CAPACITY),
        }
    }

    fn insert_component(&mut self, entity: Entity, component: T) -> Result<()> {
        if entity < self.entity_to_index.len() && self.entity_to_index[entity] != INVALID_INDEX {
            return Err(anyhow!("Cannot attach component which is already attached to entity {}", entity));
        }

        while entity >= self.entity_to_index.len() {
            self.entity_to_index.resize(self.entity_to_index.len() * 2, INVALID_INDEX);
        }

        self.entity_to_index[entity] = self.index_to_entity.len();

        self.index_to_entity.push(entity);
        self.components.push(component);

        Ok(())
    }

    fn remove_component(&mut self, entity: Entity) -> Result<()> {
        if entity >= self.entity_to_index.len() || self.entity_to_index[entity] == INVALID_INDEX {
            return Err(anyhow!("Tried to remove component which isn't attached for entity {}", entity));
        }

        let dst_index = self.entity_to_index[entity];
        self.entity_to_index[entity] = INVALID_INDEX;

        let moved_entity = self.index_to_entity.pop().unwrap_or_else(|| panic!("Internal error: index_to_entity array is empty"));
        self.index_to_entity[dst_index] = moved_entity;
        self.entity_to_index[moved_entity] = dst_index;

        let moved_component = self.components.pop().unwrap_or_else(|| panic!("Internal error: components array is empty"));
        self.components[dst_index] = moved_component;

        Ok(())
    }

    fn get_component(&self, entity: Entity) -> Result<&T> {
        if entity >= self.entity_to_index.len() || self.entity_to_index[entity] == INVALID_INDEX {
            return Err(anyhow!("Tried to get component for invalid entity {}", entity));
        }

        Ok(&self.components[self.entity_to_index[entity]])
    }

    fn get_mut_component(&mut self, entity: Entity) -> Result<&mut T> {
        if entity >= self.entity_to_index.len() || self.entity_to_index[entity] == INVALID_INDEX {
            return Err(anyhow!("Tried to get mutable component for invalid entity {}", entity));
        }

        Ok(&mut self.components[self.entity_to_index[entity]])
    }
}

/////////////////////////////////////////////////////////////////////////////
/// SystemManager
/////////////////////////////////////////////////////////////////////////////

struct SystemManager {
    signatures: HashSet<Signature>,
    entities: HashSet<Entity>,
}

impl SystemManager {
    fn new(signatures: HashSet<Signature>) -> Self {
        Self {
            signatures,
            entities: HashSet::with_capacity(INITIAL_CAPACITY),
        }
    }

    fn handle_entity_updated(&mut self, entity: Entity, signature: Signature) {
        if self.signatures.iter().any(|s| signatures_match(signature, *s)) {
            self.entities.insert(entity);
        } else {
            self.entities.remove(&entity);
        }
    }

    fn handle_entity_removed(&mut self, entity: Entity) {
        self.entities.remove(&entity);
    }
}

fn signatures_match(entity_signature: Signature, system_signature: Signature) -> bool {
    entity_signature & system_signature == system_signature
}
