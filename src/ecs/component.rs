use anyhow::{anyhow, Result};
use std::any::TypeId;
use std::collections::HashMap;

use crate::ecs::Signature;
use crate::ecs::entity::Entity;

pub trait Component: Sized + 'static {}

pub struct ComponentArray {
    component_signature: Signature,
    entity_to_index: Vec<usize>,
    index_to_entity: Vec<Entity>,
    components: Vec<u8>,
}

const INVALID_COMPONENT_INDEX: usize = usize::MAX;

impl ComponentArray {
    pub(in crate::ecs) fn new(component_signature: Signature, initial_capacity: usize) -> Self {
        Self {
            component_signature,
            entity_to_index: vec![INVALID_COMPONENT_INDEX; initial_capacity],
            index_to_entity: Vec::with_capacity(initial_capacity),
            components: Vec::with_capacity(initial_capacity * size_of::<T>()),
        }
    }

    pub(in crate::ecs) fn insert_component<T: Component>(&mut self, entity: Entity, component: T) -> Result<()> {
        if entity.0 < self.entity_to_index.len() && self.entity_to_index[entity.0] != INVALID_COMPONENT_INDEX {
            return Err(anyhow!("Component already exists for entity {:?}", entity));
        }

        while entity.0 >= self.entity_to_index.len() {
            self.entity_to_index.resize(self.entity_to_index.len() * 2, INVALID_COMPONENT_INDEX);
        }

        self.entity_to_index[entity.0] = self.index_to_entity.len();

        self.index_to_entity.push(entity);

        if self.components.len() > self.components.capacity() - size_of::<T>() {
            self.components.reserve(self.components.len());
        }

        unsafe {
            let comp_raw = self.components.as_mut_ptr().add(self.components.len()) as *mut T;
            *comp_raw = component;
        }

        Ok(())
    }

    pub(in crate::ecs) fn remove_component<T: Component>(&mut self, entity: Entity) -> Result<()> {
        if entity.0 >= self.entity_to_index.len() || self.entity_to_index[entity.0] == INVALID_COMPONENT_INDEX {
            return Err(anyhow!("No such component exists for entity {:?}", entity));
        }

        let dst_index = self.entity_to_index[entity.0];
        self.entity_to_index[entity.0] = INVALID_COMPONENT_INDEX;

        let moved_entity = self.index_to_entity.pop().unwrap_or_else(|| panic!("Internal error: index_to_entity array is empty"));
        let should_move = moved_entity != entity;

        if should_move {
            self.index_to_entity[dst_index] = moved_entity;
            self.entity_to_index[moved_entity.0] = dst_index;
        }

        let comp_index = dst_index * size_of::<T>();
        for i in (0..size_of::<T>()).rev() {
            let moved_comp_byte = self.components.pop().unwrap_or_else(|| panic!("Internal error: components array is empty"));

            if should_move {
                self.components[comp_index + i] = moved_comp_byte;
            }
        }

        Ok(())
    }

    pub fn get_component<T: Component>(&self, entity: Entity) -> Result<&T> {
        if entity.0 >= self.entity_to_index.len() || self.entity_to_index[entity.0] == INVALID_COMPONENT_INDEX {
            return Err(anyhow!("No such component exists for entity {:?}", entity));
        }

        let index = self.entity_to_index[entity.0];

        unsafe {
            let comp_raw = (self.components.as_ptr() as *const T).add(index);

            Ok(&*comp_raw)
        }
    }

    pub fn get_mut_component<T: Component>(&mut self, entity: Entity) -> Result<&mut T> {
        if entity.0 >= self.entity_to_index.len() || self.entity_to_index[entity.0] == INVALID_COMPONENT_INDEX {
            return Err(anyhow!("No such component exists for entity {:?}", entity));
        }

        let index = self.entity_to_index[entity.0];

        unsafe {
            let comp_raw = (self.components.as_mut_ptr() as *mut T).add(index);

            Ok(&mut *comp_raw)
        }
    }
}

pub struct ComponentManager {
    component_types_to_arrays: HashMap<TypeId, ComponentArray>,
}

impl ComponentManager {
    pub(in crate::ecs) fn attach_component<T: Component>(&mut self, entity: Entity, component: T) -> Result<()> {
        let comp_arr = self.component_types_to_arrays.get_mut(&TypeId::of::<T>()).map(|c| Ok(c))
            .unwrap_or(Err(anyhow!("No such component has been registered")))?;

        comp_arr.insert_component(entity, component)?;

        Ok(())
    }

    pub(in crate::ecs) fn detach_component<T: Component>(&mut self, entity: Entity) -> Result<()> {
        let comp_arr = self.component_types_to_arrays.get_mut(&TypeId::of::<T>()).map(|c| Ok(c))
            .unwrap_or(Err(anyhow!("No such component has been registered")))?;

        comp_arr.remove_component::<T>(entity)?;

        Ok(())
    }

    pub fn get_component<T: Component>(&self, entity: Entity) -> Result<&T> {
        match self.component_types_to_arrays.get(&TypeId::of::<T>()) {
            Some(comp_arr) => comp_arr.get_component::<T>(entity),
            None => Err(anyhow!("No such component has been registered")),
        }
    }

    pub fn get_mut_component<T: Component>(&mut self, entity: Entity) -> Result<&mut T> {
        match self.component_types_to_arrays.get_mut(&TypeId::of::<T>()) {
            Some(comp_arr) => comp_arr.get_mut_component(entity),
            None => Err(anyhow!("No such component has been registered")),
        }
    }
}
