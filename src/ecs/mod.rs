use std::collections::VecDeque;

pub mod components;

const MAX_ENTITY_COUNT: usize = 5;

pub trait Component: Clone {}

#[derive(Debug)]
pub struct ComponentArray<T: Component> {
    components: Vec<Option<T>>,
    entity_to_index: Vec<usize>,
    index_to_entity: Vec<usize>,
    size: usize,
}

impl<T: Component> ComponentArray<T> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn insert(&mut self, entity: usize, component: T) {
        assert!(entity < MAX_ENTITY_COUNT, "Tried to insert invalid entity {}", entity);

        let index = self.size;

        self.index_to_entity[index] = entity;
        self.entity_to_index[entity] = index;
        self.components[index] = Some(component);

        self.size += 1;
    }

    pub fn remove(&mut self, entity: usize) {
        assert!(entity < MAX_ENTITY_COUNT && self.entity_to_index[entity] < MAX_ENTITY_COUNT, "Tried to remove invalid entity {}", entity);

        let index = self.entity_to_index[entity];
        let moved_entity = self.index_to_entity[self.size - 1];

        self.index_to_entity[index] = moved_entity;
        self.entity_to_index[moved_entity] = index;

        self.index_to_entity[self.size - 1] = usize::MAX;
        self.entity_to_index[entity] = usize::MAX;

        self.components[index] = self.components[self.size - 1].clone();
        self.components[self.size - 1] = None;

        self.size -= 1;
    }

    pub fn get(&self, entity: usize) -> &T {
        assert!(entity < MAX_ENTITY_COUNT && self.entity_to_index[entity] < MAX_ENTITY_COUNT, "Tried to get invalid entity {}", entity);

        self.components[self.entity_to_index[entity]].as_ref().unwrap()
    }
}

impl<T: Component> Default for ComponentArray<T> {
    fn default() -> Self {
        Self {
            components: std::iter::repeat_with(|| None).take(MAX_ENTITY_COUNT).collect::<Vec<_>>(),
            entity_to_index: vec![usize::MAX; MAX_ENTITY_COUNT],
            index_to_entity: vec![usize::MAX; MAX_ENTITY_COUNT],
            size: 0,
        }
    }
}

pub type EntitySignature = u16;

#[derive(Debug)]
pub struct EntityManager {
    entity_counter: usize,
    usable_entities: VecDeque<usize>,
    signatures: Vec<u16>,
}

impl EntityManager {
    pub fn new() -> Self {
        Self {
            entity_counter: 0,
            usable_entities: VecDeque::new(),
            signatures: vec![0; MAX_ENTITY_COUNT],
        }
    }

    pub fn create_entity(&mut self) -> usize {
        assert!(self.entity_counter < MAX_ENTITY_COUNT, "Exceeded max entity count of {}", MAX_ENTITY_COUNT);

        self.usable_entities.pop_front().unwrap_or_else(|| {
            let new_entity = self.entity_counter;
            self.entity_counter += 1;
            new_entity
        })
    }

    pub fn destroy_entity(&mut self, entity: usize) {
        assert!(entity < self.entity_counter, "Tried to destroy invalid entity {}", entity);

        self.signatures[entity] = 0;
        self.usable_entities.push_back(entity);
    }

    pub fn set_signature(&mut self, entity: usize, signature: EntitySignature) {
        assert!(entity < MAX_ENTITY_COUNT, "Tried to set signature for invalid entity {}", entity);

        self.signatures[entity] = signature;
    }

    pub fn has_matching_signature(&self, entity: usize, signature: EntitySignature) -> bool {
        assert!(entity < MAX_ENTITY_COUNT, "Tried to get signature for invalid entity {}", entity);

        self.signatures[entity] & signature == signature
    }
}
