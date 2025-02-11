use std::collections::HashSet;

use crate::ecs::Signature;
use crate::ecs::entity::Entity;

type System = fn(); // TODO: component manager, entity iterator, entity commands

pub(in crate::ecs) struct SystemManager {
    system: System,
    system_signatures: HashSet<Signature>,
    entities: HashSet<Entity>,
}

impl SystemManager {
    pub(in crate::ecs) fn new(system: System, system_signatures: HashSet<Signature>, initial_capacity: usize) -> Self {
        Self {
            system,
            system_signatures,
            entities: HashSet::with_capacity(initial_capacity),
        }
    }

    pub(in crate::ecs) fn handle_entity_updated(&mut self, entity: Entity, signature: Signature) {
        if self.system_signatures.iter().any(|s| signatures_match(signature, *s)) {
            self.entities.insert(entity);
        } else {
            self.entities.remove(&entity);
        }
    }

    pub(in crate::ecs) fn handle_entity_removed(&mut self, entity: Entity) {
        self.entities.remove(&entity);
    }
}

#[inline]
const fn signatures_match(entity_signature: Signature, system_signature: Signature) -> bool {
    entity_signature & system_signature == system_signature
}
