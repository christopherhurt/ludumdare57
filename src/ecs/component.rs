use anyhow::{anyhow, Result};
use std::any::TypeId;
use std::collections::HashMap;
use std::mem::ManuallyDrop;

use crate::ecs::{ComponentActions, Signature};
use crate::ecs::entity::Entity;

pub trait Component: ComponentActions + Sized + 'static {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SystemSignature(pub(in crate::ecs) Signature);

pub struct ComponentArray {
    entity_to_index: Vec<usize>,
    index_to_entity: Vec<Entity>,
    components: Vec<u8>,
    component_size: usize,
}

const INVALID_COMPONENT_INDEX: usize = usize::MAX;
const INITIAL_BYTES_PER_CAPACITY: usize = 16;

impl ComponentArray {
    pub(in crate::ecs) fn new(initial_capacity: usize, component_size: usize) -> Self {
        Self {
            entity_to_index: vec![INVALID_COMPONENT_INDEX; initial_capacity],
            index_to_entity: Vec::with_capacity(initial_capacity),
            components: Vec::with_capacity(initial_capacity * INITIAL_BYTES_PER_CAPACITY),
            component_size,
        }
    }

    pub(in crate::ecs) fn insert_component(&mut self, entity: &Entity, component: Box<[u8]>) -> Result<()> {
        if entity.0 < self.entity_to_index.len() && self.entity_to_index[entity.0] != INVALID_COMPONENT_INDEX {
            return Err(anyhow!("Component already exists for entity {:?}", entity));
        }

        while entity.0 >= self.entity_to_index.len() {
            self.entity_to_index.resize(self.entity_to_index.len() * 2, INVALID_COMPONENT_INDEX);
        }

        self.entity_to_index[entity.0] = self.index_to_entity.len();

        self.index_to_entity.push(entity.clone());

        if self.components.len() > self.components.capacity() - component.len() {
            self.components.reserve(self.components.len());
        }

        self.components.extend_from_slice(&component.as_ref());

        Ok(())
    }

    pub(in crate::ecs) fn remove_component(&mut self, entity: &Entity) -> Result<()> {
        if entity.0 >= self.entity_to_index.len() || self.entity_to_index[entity.0] == INVALID_COMPONENT_INDEX {
            return Err(anyhow!("No such component exists for entity {:?}", entity));
        }

        let dst_index = self.entity_to_index[entity.0];
        self.entity_to_index[entity.0] = INVALID_COMPONENT_INDEX;

        let moved_entity = self.index_to_entity.pop().unwrap_or_else(|| panic!("Internal error: index_to_entity array is empty"));
        let should_move = moved_entity != *entity;

        if should_move {
            self.index_to_entity[dst_index] = moved_entity;
            self.entity_to_index[moved_entity.0] = dst_index;
        }

        // TODO: need to manually drop the component being removed

        let comp_index = dst_index * self.component_size;
        for i in (0..self.component_size).rev() {
            let moved_comp_byte = self.components.pop().unwrap_or_else(|| panic!("Internal error: components array is empty"));

            if should_move {
                self.components[comp_index + i] = moved_comp_byte;
            }
        }

        Ok(())
    }

    pub fn get_component<T: Component>(&self, entity: &Entity) -> Option<&T> {
        if entity.0 >= self.entity_to_index.len() || self.entity_to_index[entity.0] == INVALID_COMPONENT_INDEX {
            return None;
        }

        let index = self.entity_to_index[entity.0];

        unsafe {
            let comp_raw = (self.components.as_ptr() as *const T).add(index);

            Some(&*comp_raw)
        }
    }

    pub fn get_mut_component<T: Component>(&self, entity: &Entity) -> Option<&mut T> {
        if entity.0 >= self.entity_to_index.len() || self.entity_to_index[entity.0] == INVALID_COMPONENT_INDEX {
            return None;
        }

        let index = self.entity_to_index[entity.0];

        unsafe {
            // TODO: should enforce proper interior mutability here like RefCell, i.e. panic on more than one borrow with at least one mutable borrow
            let comp_raw = (self.components.as_ptr() as *mut T).add(index);

            Some(&mut *comp_raw)
        }
    }
}

impl Drop for ComponentArray {
    fn drop(&mut self) {
        // TODO: need to drop all components in the array
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

        let comp_data = component_to_boxed_slice(component);

        comp_arr.insert_component(entity, comp_data)?;

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

        self.component_types_to_arrays.insert(type_id, ComponentArray::new(self.initial_capacity, size_of::<T>()));
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

fn component_to_boxed_slice(component: Box<dyn ComponentActions>) -> Box<[u8]> {
    let ptr = component.as_ref() as *const dyn ComponentActions as *const u8;
    let comp_size = std::mem::size_of_val(component.as_ref());

    unsafe {
        let raw_slice = std::slice::from_raw_parts(ptr, comp_size);

        // TODO: right now we're not actually doing the below... will need to figure out the right way to do this
        // The Box below takes "ownership" of the component (or rather, its raw bytes), but if we don't wrap it in
        //  the ManuallyDrop here, the component will still be dropped. Instead, we'll drop the component ourselves
        //  when it's removed from the ComponentArray. This is safe because after ownership of the component is moved
        //  to the ComponentArray, it doesn't change owners for the remainder of its lifetime.
        let _ = ManuallyDrop::new(component);

        Box::from(raw_slice)
    }
}
