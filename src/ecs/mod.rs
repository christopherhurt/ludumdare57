use anyhow::{anyhow, Result};
use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet, VecDeque};

/////////////////////////////////////////////////////////////////////////////
/// Public
/////////////////////////////////////////////////////////////////////////////

pub type Entity = usize;
pub type Signature = u32;

pub trait Component: Clone + std::fmt::Debug + Sized + 'static {
    fn as_any(&self) -> &dyn Any {
        self
    }
} // TODO: need + 'static?
pub trait System {}

#[derive(Debug, Default)]
pub struct ECS {
    entity_manager: EntityManager,
    component_types_to_arrays: HashMap<TypeId, ComponentArray<dyn Component>>,
    // system_to_entities: HashMap<TypeId>,
}

impl ECS {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn create_entity(&mut self) -> Entity {
        self.entity_manager.create_entity()
    }

    pub fn destroy_entity(&mut self, entity: Entity) -> Result<()> {
        self.entity_manager.destroy_entity(entity)?;
        // TODO
        self.component_types_to_arrays.values().for_each(|b| {
            b.downcast_ref::<ComponentArray<_>>().unwrap_or_else(|| panic!("")).remove_component(entity);
        });
        Ok(())
        // TODO
    }

    pub fn attach_component<T: Component>(&mut self, entity: Entity, component: T) -> Result<()> {
        // TODO
    }

    pub fn remove_component<T: Component>(&mut self, entity: Entity) -> Result<()> {
        // TODO
    }

    pub fn get_component<T: Component>(&self, entity: Entity) -> Result<&T> {
        // TODO
    }

    pub fn get_mut_component<T: Component>(&mut self, entity: Entity) -> Result<&mut T> {
        // TODO
    }

    pub fn get_system_entities<S: System>(&self) -> Result<()> { // TODO return type?
        // TODO
    }

    pub fn register_component<T: Component>(&mut self) -> Result<Signature> {
        // TODO
    }

    pub fn register_system<S: System>(&mut self, signatures: Vec<Signature>) -> Result<()> {
        // TODO
    }
}

/////////////////////////////////////////////////////////////////////////////
/// Common
/////////////////////////////////////////////////////////////////////////////

const INITIAL_CAPACITY: usize = 1_024;

/////////////////////////////////////////////////////////////////////////////
/// EntityManager
/////////////////////////////////////////////////////////////////////////////

const DEFAULT_SIGNATURE: Signature = 0;

#[derive(Debug)]
struct EntityManager {
    entity_counter: Entity,
    usable_entities: VecDeque<Entity>,
    signatures: Vec<Signature>,
    entity_destroyed: Vec<bool>,
}

impl EntityManager {
    fn new() -> Self {
        Default::default()
    }

    fn create_entity(&mut self) -> Entity {
        let new_entity = self.usable_entities.pop_front().unwrap_or_else(|| {
            let new_entity = self.entity_counter;
            self.entity_counter += 1;
            new_entity
        });

        while self.signatures.len() <= new_entity {
            self.signatures.resize(self.signatures.len() * 2, DEFAULT_SIGNATURE);
        }
        while self.entity_destroyed.len() <= new_entity {
            self.entity_destroyed.resize(self.entity_destroyed.len() * 2, true);
        }

        self.entity_destroyed[new_entity] = false;

        new_entity
    }

    fn destroy_entity(&mut self, entity: Entity) -> Result<()> {
        if self.entity_destroyed[entity] {
            return Err(anyhow!("Tried to destroy entity {} which doesn't exist", entity));
        }

        self.signatures[entity] = DEFAULT_SIGNATURE;
        self.entity_destroyed[entity] = true;

        self.usable_entities.push_back(entity);

        Ok(())
    }

    fn set_signature(&mut self, entity: Entity, signature: Signature) -> Result<()> {
        if self.entity_destroyed[entity] {
            return Err(anyhow!("Tried to set signature for invalid entity {}", entity));
        }

        self.signatures[entity] = signature;

        Ok(())
    }

    fn has_matching_signature(&self, entity: Entity, signature: Signature) -> Result<bool> {
        if self.entity_destroyed[entity] {
            return Err(anyhow!("Tried to compare signature for invalid entity {}", entity));
        }

        Ok(self.signatures[entity] & signature == signature)
    }
}

impl Default for EntityManager {
    fn default() -> Self {
        Self {
            entity_counter: 0,
            usable_entities: VecDeque::new(),
            signatures: vec![DEFAULT_SIGNATURE; INITIAL_CAPACITY],
            entity_destroyed: vec![true; INITIAL_CAPACITY],
        }
    }
}

/////////////////////////////////////////////////////////////////////////////
/// ComponentArray
/////////////////////////////////////////////////////////////////////////////

const INVALID_INDEX: usize = usize::MAX;

#[derive(Debug)]
struct ComponentArray<T: Component> {
    entity_to_index: Vec<usize>,
    index_to_entity: Vec<Entity>,
    components: Vec<T>,
}

impl<T: Component> ComponentArray<T> {
    fn new() -> Self {
        Default::default()
    }

    fn insert_component(&mut self, entity: Entity, component: T) -> Result<()> {
        if entity < self.entity_to_index.len() && self.entity_to_index[entity] != INVALID_INDEX {
            return Err(anyhow!("Tried to attach component which already exists for entity {}", entity));
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
            return Err(anyhow!("Tried to remove component which doesn't exist for entity {}", entity));
        }

        let dst_index = self.entity_to_index[entity];

        self.entity_to_index[self.index_to_entity[self.index_to_entity.len() - 1]] = dst_index;
        self.index_to_entity[dst_index] = self.index_to_entity[self.index_to_entity.len() - 1];
        self.components[dst_index] = self.components[self.components.len() - 1].clone();

        self.entity_to_index[entity] = INVALID_INDEX;
        self.index_to_entity.pop().unwrap_or_else(|| panic!("Internal error: index_to_entity array is empty"));
        self.components.pop().unwrap_or_else(|| panic!("Internal error: components array is empty"));

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

impl<T: Component> Default for ComponentArray<T> {
    fn default() -> Self {
        Self {
            entity_to_index: vec![INVALID_INDEX; INITIAL_CAPACITY],
            index_to_entity: Vec::with_capacity(INITIAL_CAPACITY),
            components: Vec::with_capacity(INITIAL_CAPACITY),
        }
    }
}

/////////////////////////////////////////////////////////////////////////////
/// SystemManager
/////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct SystemManager {
    system_type_id: TypeId,
    signatures: Vec<Signature>,
    entities: HashSet<Entity>,
}

impl SystemManager {
    fn new(system_type_id: TypeId, signatures: Vec<Signature>) -> Self {
        Self {
            system_type_id,
            signatures,
            entities: HashSet::with_capacity(INITIAL_CAPACITY),
        }
    }

    fn entity_signature_updated(&mut self, entity: Entity, signature: Signature) {
        // TODO
        // if self.signatures.iter().any(|s| )
    }
}
