use anyhow::{anyhow, Result};
use std::cmp::min;
use std::collections::{HashMap, VecDeque};

use crate::ecs::Signature;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity(pub(in crate) usize);

pub(in crate::ecs) struct EntityManager {
    entity_counter: usize,
    max_capacity: usize,
    usable_entities: VecDeque<Entity>, // TODO: only use these after a certain large size it reached? to prevent collisions with any refs to entities which may have changed
    signatures: Vec<Signature>,
    entity_destroyed: Vec<bool>,
}

const DEFAULT_ENTITY_SIGNATURE: Signature = 0;

impl EntityManager {
    pub(in crate::ecs) fn new(initial_capacity: usize, max_capacity: usize) -> Self {
        Self {
            entity_counter: 0,
            max_capacity,
            usable_entities: VecDeque::new(),
            signatures: vec![DEFAULT_ENTITY_SIGNATURE; initial_capacity],
            entity_destroyed: vec![true; initial_capacity],
        }
    }

    pub(in crate::ecs) fn create_entity(&mut self) -> Result<Entity> {
        let new_entity = self.usable_entities.pop_front().unwrap_or_else(|| {
            let new_entity = Entity(self.entity_counter);
            self.entity_counter += 1;
            new_entity
        });

        if new_entity.0 >= self.max_capacity {
            return Err(anyhow!("Exceeded max entity capacity of {:?}", self.max_capacity));
        }

        while self.signatures.len() <= new_entity.0 {
            self.signatures.resize(min(self.signatures.len() * 2, self.max_capacity), DEFAULT_ENTITY_SIGNATURE);
        }
        while self.entity_destroyed.len() <= new_entity.0 {
            self.entity_destroyed.resize(min(self.entity_destroyed.len() * 2, self.max_capacity), true);
        }

        self.entity_destroyed[new_entity.0] = false;

        Ok(new_entity)
    }

    pub(in crate::ecs) fn destroy_entity(&mut self, entity: &Entity) -> Result<()> {
        if entity.0 >= self.entity_counter || self.entity_destroyed[entity.0] {
            return Err(anyhow!("Entity {:?} does not exist", entity));
        }

        self.signatures[entity.0] = DEFAULT_ENTITY_SIGNATURE;
        self.entity_destroyed[entity.0] = true;

        self.usable_entities.push_back(entity.clone());

        Ok(())
    }

    pub(in crate::ecs) fn set_signature(&mut self, entity: &Entity, signature: Signature) -> Result<()> {
        if entity.0 >= self.entity_counter || self.entity_destroyed[entity.0] {
            return Err(anyhow!("Entity {:?} does not exist", entity));
        }

        self.signatures[entity.0] = signature;

        Ok(())
    }

    pub(in crate::ecs) fn get_signature(&self, entity: &Entity) -> Result<Signature> {
        if entity.0 >= self.entity_counter || self.entity_destroyed[entity.0] {
            return Err(anyhow!("Entity {:?} does not exist", entity));
        }

        Ok(self.signatures[entity.0])
    }

    pub(in crate::ecs) fn get_all_entities_and_signatures(&self) -> HashMap<Entity, Signature> {
        let mut result = HashMap::new();

        for i in 0..self.entity_destroyed.len() {
            if !self.entity_destroyed[i] {
                let entity = Entity(i);
                let signature = self.get_signature(&entity).unwrap_or_else(|_| panic!("Internal error: no signature exists for non-destroyed entity {:?}", entity));

                result.insert(entity, signature);
            }
        }

        result
    }
}
