use anyhow::{anyhow, Result};
use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;

use crate::ecs::{ComponentActions, Signature};
use crate::ecs::entity::Entity;

pub trait Component: ComponentActions + Sized + 'static {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SystemSignature(pub(in crate::ecs) Signature);

pub struct ComponentArray {
    entity_to_index: Vec<usize>,
    index_to_entity: Vec<Entity>,
    components: Vec<RefCell<Box<dyn Any>>>,
}

const INVALID_COMPONENT_INDEX: usize = usize::MAX;

impl ComponentArray {
    pub(in crate::ecs) fn new(initial_capacity: usize) -> Self {
        Self {
            entity_to_index: vec![INVALID_COMPONENT_INDEX; initial_capacity],
            index_to_entity: Vec::with_capacity(initial_capacity),
            components: Vec::with_capacity(initial_capacity),
        }
    }

    pub(in crate::ecs) fn insert_component(&mut self, entity: &Entity, component: Box<dyn ComponentActions>) -> Result<()> {
        if entity.0 < self.entity_to_index.len() && self.entity_to_index[entity.0] != INVALID_COMPONENT_INDEX {
            return Err(anyhow!("Component already exists for entity {:?}", entity));
        }

        while entity.0 >= self.entity_to_index.len() {
            self.entity_to_index.resize(self.entity_to_index.len() * 2, INVALID_COMPONENT_INDEX);
        }

        self.entity_to_index[entity.0] = self.index_to_entity.len();

        self.index_to_entity.push(entity.clone());

        self.components.push(RefCell::new(component.as_any_box()));

        Ok(())
    }

    pub(in crate::ecs) fn remove_component(&mut self, entity: &Entity) -> Result<()> {
        if entity.0 >= self.entity_to_index.len() || self.entity_to_index[entity.0] == INVALID_COMPONENT_INDEX {
            return Err(anyhow!("No such component exists for entity {:?}", entity));
        }

        let dst_index = self.entity_to_index[entity.0];
        self.entity_to_index[entity.0] = INVALID_COMPONENT_INDEX;

        let moved_entity = self.index_to_entity.pop().unwrap_or_else(|| panic!("Internal error: index_to_entity Vec is empty"));
        let moved_component = self.components.pop().unwrap_or_else(|| panic!("Internal error: components Vec is empty"));
        let should_move = moved_entity != *entity;

        if should_move {
            self.index_to_entity[dst_index] = moved_entity;
            self.components[dst_index] = moved_component;
            self.entity_to_index[moved_entity.0] = dst_index;
        }

        Ok(())
    }

    pub fn get_component<T: Component>(&self, entity: &Entity) -> Option<&T> {
        if entity.0 >= self.entity_to_index.len() || self.entity_to_index[entity.0] == INVALID_COMPONENT_INDEX {
            return None;
        }

        let index = self.entity_to_index[entity.0];

        unsafe { Some(&*(self.components[index].borrow().as_ref().downcast_ref::<T>().unwrap_or_else(|| panic!("Internal error: Failed to downcast component")) as *const T)) }
    }

    pub fn get_mut_component<T: Component>(&self, entity: &Entity) -> Option<&mut T> {
        if entity.0 >= self.entity_to_index.len() || self.entity_to_index[entity.0] == INVALID_COMPONENT_INDEX {
            return None;
        }

        let index = self.entity_to_index[entity.0];

        // TODO: ensure actual borrow checking here... there could be a way to do it with no unsafe code
        // I luv Rust...
        unsafe { Some(&mut *(self.components[index].borrow_mut().as_mut().downcast_mut::<T>().unwrap_or_else(|| panic!("Internal error: Failed to downcast component")) as *mut T)) }
    }
}

pub struct ComponentManager {
    component_registration_bit: Signature,
    component_types_to_signatures: HashMap<TypeId, Signature>,
    component_types_to_arrays: HashMap<TypeId, ComponentArray>,
    pub(in crate::ecs) initial_capacity: usize,
}

const DEFAULT_INITIAL_COMPONENT_CAPACITY: usize = 64;

impl ComponentManager {
    pub(in crate::ecs) fn new(initial_capacity: usize) -> Self {
        Self {
            component_registration_bit: 1,
            component_types_to_signatures: HashMap::with_capacity(DEFAULT_INITIAL_COMPONENT_CAPACITY),
            component_types_to_arrays: HashMap::with_capacity(DEFAULT_INITIAL_COMPONENT_CAPACITY),
            initial_capacity,
        }
    }

    pub(in crate::ecs) fn attach_component(&mut self, entity: &Entity, type_id: TypeId, component: Box<dyn ComponentActions>) -> Result<()> {
        let comp_arr = self.component_types_to_arrays.get_mut(&type_id).map(|c| Ok(c))
            .unwrap_or(Err(anyhow!("No such component has been registered")))?;

        comp_arr.insert_component(entity, component)?;

        Ok(())
    }

    pub(in crate::ecs) fn detach_component(&mut self, entity: &Entity, type_id: TypeId) -> Result<()> {
        let comp_arr = self.component_types_to_arrays.get_mut(&type_id).map(|c| Ok(c))
            .unwrap_or(Err(anyhow!("No such component has been registered")))?;

        comp_arr.remove_component(entity)?;

        Ok(())
    }

    pub(in crate::ecs) fn get_signature(&self, type_id: TypeId) -> Result<Signature> {
        let signature = self.component_types_to_signatures.get(&type_id).map(|s| Ok(s))
            .unwrap_or(Err(anyhow!("No such component has been registered")))?;

        Ok(*signature)
    }

    pub(in crate::ecs) fn register_component<T: Component>(&mut self) -> Result<()> {
        let type_id = TypeId::of::<T>();

        if self.component_types_to_arrays.contains_key(&type_id) {
            return Err(anyhow!("The component is already registered"));
        }

        if self.component_registration_bit == 0 {
            return Err(anyhow!("The maximum number of components has already been registered"));
        }

        let component_signature = self.component_registration_bit;
        self.component_registration_bit <<= 1;

        self.component_types_to_arrays.insert(type_id, ComponentArray::new(self.initial_capacity));
        self.component_types_to_signatures.insert(type_id, component_signature);

        Ok(())
    }

    pub fn get_component<T: Component>(&self, entity: &Entity) -> Option<&T> {
        match self.component_types_to_arrays.get(&TypeId::of::<T>()) {
            Some(comp_arr) => comp_arr.get_component::<T>(entity),
            None => None,
        }
    }

    pub fn get_mut_component<T: Component>(&self, entity: &Entity) -> Option<&mut T> {
        match self.component_types_to_arrays.get(&TypeId::of::<T>()) {
            Some(comp_arr) => comp_arr.get_mut_component(entity),
            None => None,
        }
    }

    pub(in crate::ecs) fn handle_entity_removed(&mut self, entity: &Entity) {
        self.component_types_to_arrays.values_mut().for_each(|comp_arr| {
            comp_arr.remove_component(entity).unwrap_or_default();
        });
    }

    pub fn get_system_signature_0(&self) -> Result<SystemSignature> {
        Ok(SystemSignature(0))
    }

    pub fn get_system_signature_1<A: Component>(&self) -> Result<SystemSignature> {
        let sig = self.get_signature(TypeId::of::<A>())?;

        Ok(SystemSignature(sig))
    }

    pub fn get_system_signature_2<A: Component, B: Component>(&self) -> Result<SystemSignature> {
        let sig_a = self.get_signature(TypeId::of::<A>())?;
        let sig_b = self.get_signature(TypeId::of::<B>())?;

        Ok(SystemSignature(sig_a | sig_b))
    }

    pub fn get_system_signature_3<A: Component, B: Component, C: Component>(&self) -> Result<SystemSignature> {
        let sig_a = self.get_signature(TypeId::of::<A>())?;
        let sig_b = self.get_signature(TypeId::of::<B>())?;
        let sig_c = self.get_signature(TypeId::of::<C>())?;

        Ok(SystemSignature(sig_a | sig_b | sig_c))
    }

    pub fn get_system_signature_4<A: Component, B: Component, C: Component, D: Component>(&self) -> Result<SystemSignature> {
        let sig_a = self.get_signature(TypeId::of::<A>())?;
        let sig_b = self.get_signature(TypeId::of::<B>())?;
        let sig_c = self.get_signature(TypeId::of::<C>())?;
        let sig_d = self.get_signature(TypeId::of::<D>())?;

        Ok(SystemSignature(sig_a | sig_b | sig_c | sig_d))
    }
}
