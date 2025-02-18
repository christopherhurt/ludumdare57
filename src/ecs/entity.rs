use anyhow::{anyhow, Result};
use std::cmp::min;
use std::collections::VecDeque;

use crate::ecs::{DEFAULT_ENTITY_SIGNATURE, Signature};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity(pub(in crate::ecs) usize);

pub(in crate::ecs) struct EntityManager {
    entity_counter: usize,
    max_capacity: usize,
    usable_entities: VecDeque<Entity>,
    signatures: Vec<Signature>,
    entity_destroyed: Vec<bool>,
}

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

    pub(in crate::ecs) fn destroy_entity(&mut self, entity: Entity) -> Result<()> {
        if entity.0 >= self.entity_counter || self.entity_destroyed[entity.0] {
            return Err(anyhow!("Entity {:?} does not exist", entity));
        }

        self.signatures[entity.0] = DEFAULT_ENTITY_SIGNATURE;
        self.entity_destroyed[entity.0] = true;

        self.usable_entities.push_back(entity);

        Ok(())
    }

    pub(in crate::ecs) fn set_signature(&mut self, entity: Entity, signature: Signature) -> Result<()> {
        if entity.0 >= self.entity_counter || self.entity_destroyed[entity.0] {
            return Err(anyhow!("Entity {:?} does not exist", entity));
        }

        self.signatures[entity.0] = signature;

        Ok(())
    }

    pub(in crate::ecs) fn get_signature(&self, entity: Entity) -> Result<Signature> {
        if entity.0 >= self.entity_counter || self.entity_destroyed[entity.0] {
            return Err(anyhow!("Entity {:?} does not exist", entity));
        }

        Ok(self.signatures[entity.0])
    }
}
