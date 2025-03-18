use std::collections::HashSet;
use std::collections::hash_set::Iter;

use crate::ecs::{ECSCommands, Signature};
use crate::ecs::component::ComponentManager;
use crate::ecs::entity::Entity;

pub type System = fn(entites: Iter<Entity>, components: &ComponentManager, commands: &mut ECSCommands);

pub(in crate::ecs) struct SystemManager {
    pub(in crate::ecs) system: System,
    system_signatures: HashSet<Signature>,
    pub(in crate::ecs) precedence: i16,
    entities: HashSet<Entity>,
}

impl SystemManager {
    pub(in crate::ecs) fn new(system: System, system_signatures: HashSet<Signature>, precedence: i16, initial_capacity: usize) -> Self {
        Self {
            system,
            system_signatures,
            precedence,
            entities: HashSet::with_capacity(initial_capacity),
        }
    }

    pub(in crate::ecs) fn handle_entity_updated(&mut self, entity: &Entity, signature: Signature) {
        if self.system_signatures.iter().any(|s| signatures_match(signature, *s)) {
            self.entities.insert(entity.clone());
        } else {
            self.entities.remove(entity);
        }
    }

    pub(in crate::ecs) fn handle_entity_removed(&mut self, entity: &Entity) {
        self.entities.remove(entity);
    }

    pub(in crate::ecs) fn invoke_system(&self, components: &mut ComponentManager, commands: &mut ECSCommands) {
        (self.system)(self.entities.iter(), components, commands);
    }
}

#[inline]
const fn signatures_match(entity_signature: Signature, system_signature: Signature) -> bool {
    entity_signature & system_signature == system_signature
}
