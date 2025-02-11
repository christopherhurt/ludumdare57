use anyhow::{anyhow, Result};

use crate::ecs::Signature;
use crate::ecs::entity::Entity;

pub trait Component {}

pub struct ComponentArray<T: Component> {
    component_signature: Signature,
    entity_to_index: Vec<usize>,
    index_to_entity: Vec<Entity>,
    components: Vec<T>,
}

const INVALID_COMPONENT_INDEX: usize = usize::MAX;

impl<T: Component> ComponentArray<T> {
    pub(in crate::ecs) fn new(component_signature: Signature, initial_capacity: usize) -> Self {
        Self {
            component_signature,
            entity_to_index: vec![INVALID_COMPONENT_INDEX; initial_capacity],
            index_to_entity: Vec::with_capacity(initial_capacity),
            components: Vec::with_capacity(initial_capacity),
        }
    }

    pub(in crate::ecs) fn insert_component(&mut self, entity: Entity, component: T) -> Result<()> {
        if entity.0 < self.entity_to_index.len() && self.entity_to_index[entity.0] != INVALID_COMPONENT_INDEX {
            return Err(anyhow!("Component already exists for entity {:?}", entity));
        }

        while entity.0 >= self.entity_to_index.len() {
            self.entity_to_index.resize(self.entity_to_index.len() * 2, INVALID_COMPONENT_INDEX);
        }

        self.entity_to_index[entity.0] = self.index_to_entity.len();

        self.index_to_entity.push(entity);
        self.components.push(component);

        Ok(())
    }

    pub(in crate::ecs) fn remove_component(&mut self, entity: Entity) -> Result<()> {
        if entity.0 >= self.entity_to_index.len() || self.entity_to_index[entity.0] == INVALID_COMPONENT_INDEX {
            return Err(anyhow!("No such component exists for entity {:?}", entity));
        }

        let dst_index = self.entity_to_index[entity.0];
        self.entity_to_index[entity.0] = INVALID_COMPONENT_INDEX;

        let moved_entity = self.index_to_entity.pop().unwrap_or_else(|| panic!("Internal error: index_to_entity array is empty"));
        self.index_to_entity[dst_index] = moved_entity;
        self.entity_to_index[moved_entity.0] = dst_index;

        let moved_component = self.components.pop().unwrap_or_else(|| panic!("Internal error: components array is empty"));
        self.components[dst_index] = moved_component;

        Ok(())
    }

    pub fn get_component(&self, entity: Entity) -> Result<&T> {
        if entity.0 >= self.entity_to_index.len() || self.entity_to_index[entity.0] == INVALID_COMPONENT_INDEX {
            return Err(anyhow!("No such component exists for entity {:?}", entity));
        }

        Ok(&self.components[self.entity_to_index[entity.0]])
    }

    pub fn get_mut_component(&mut self, entity: Entity) -> Result<&mut T> {
        if entity.0 >= self.entity_to_index.len() || self.entity_to_index[entity.0] == INVALID_COMPONENT_INDEX {
            return Err(anyhow!("No such component exists for entity {:?}", entity));
        }

        Ok(&mut self.components[self.entity_to_index[entity.0]])
    }
}
