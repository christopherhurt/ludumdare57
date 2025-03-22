use std::collections::HashMap;

use crate::ecs::{ComponentActions, ProvisionalEntity};
use crate::ecs::component::Component;
use crate::ecs::entity::Entity;
use crate::math::{Vec3, VEC_3_ZERO};

// Particle

#[derive(Clone, Debug)]
pub struct Particle {
    pub vel: Vec3,
    pub acc: Vec3,
    pub damping: f32,
    pub mass: f32,
    pub gravity: f32,
    pub force_accum: Vec3,
    // TODO: add material which can be used to derive the coefficient of restitution - for now, it's being generated randomly for each collision
}

impl Particle {
    pub fn new(vel: Vec3, damping: f32, mass: f32, gravity: f32) -> Self {
        Self {
            vel,
            acc: VEC_3_ZERO,
            damping,
            mass,
            gravity,
            force_accum: VEC_3_ZERO,
        }
    }
}

impl Default for Particle {
    fn default() -> Self {
        Self {
            vel: VEC_3_ZERO,
            acc: VEC_3_ZERO,
            damping: 1.0,
            mass: 1.0,
            gravity: 10.0,
            force_accum: VEC_3_ZERO,
        }
    }
}

impl Component for Particle {}
impl ComponentActions for Particle {}

// ParticleCable

#[derive(Clone, Debug)]
pub struct ParticleCable {
    pub particle_a: Entity,
    pub particle_b: Entity,
    pub particle_a_prov: ProvisionalEntity,
    pub particle_b_prov: ProvisionalEntity,
    pub max_length: f32,
    pub restitution: f32,
}

impl ParticleCable {
    pub fn new(
        particle_a: Entity,
        particle_b: Entity,
        max_length: f32,
        restitution: f32,
    ) -> Self {
        Self {
            particle_a,
            particle_b,
            particle_a_prov: ProvisionalEntity(0),
            particle_b_prov: ProvisionalEntity(0),
            max_length,
            restitution,
        }
    }

    pub fn new_provisional(
        particle_a: ProvisionalEntity,
        particle_b: ProvisionalEntity,
        max_length: f32,
        restitution: f32,
    ) -> Self {
        Self {
            particle_a: Entity(0),
            particle_b: Entity(0),
            particle_a_prov: particle_a,
            particle_b_prov: particle_b,
            max_length,
            restitution,
        }
    }
}

impl Component for ParticleCable {}

impl ComponentActions for ParticleCable {
    fn update_provisional_entities(&mut self, provisional_to_entities: &HashMap<ProvisionalEntity, Entity>) {
        self.particle_a = provisional_to_entities.get(&self.particle_a_prov).unwrap_or_else(|| panic!("Failed to map provisional entity {:?}", &self.particle_a_prov)).clone();
        self.particle_b = provisional_to_entities.get(&self.particle_b_prov).unwrap_or_else(|| panic!("Failed to map provisional entity {:?}", &self.particle_b_prov)).clone();
    }
}

// ParticleRod

#[derive(Clone, Debug)]
pub struct ParticleRod {
    pub particle_a: Entity,
    pub particle_b: Entity,
    pub particle_a_prov: ProvisionalEntity,
    pub particle_b_prov: ProvisionalEntity,
    pub length: f32,
}

impl ParticleRod {
    pub fn new(
        particle_a: Entity,
        particle_b: Entity,
        length: f32,
    ) -> Self {
        Self {
            particle_a,
            particle_b,
            particle_a_prov: ProvisionalEntity(0),
            particle_b_prov: ProvisionalEntity(0),
            length,
        }
    }

    pub fn new_provisional(
        particle_a: ProvisionalEntity,
        particle_b: ProvisionalEntity,
        length: f32,
    ) -> Self {
        Self {
            particle_a: Entity(0),
            particle_b: Entity(0),
            particle_a_prov: particle_a,
            particle_b_prov: particle_b,
            length,
        }
    }
}

impl Component for ParticleRod {}

impl ComponentActions for ParticleRod {
    fn update_provisional_entities(&mut self, provisional_to_entities: &HashMap<ProvisionalEntity, Entity>) {
        self.particle_a = provisional_to_entities.get(&self.particle_a_prov).unwrap_or_else(|| panic!("Failed to map provisional entity {:?}", &self.particle_a_prov)).clone();
        self.particle_b = provisional_to_entities.get(&self.particle_b_prov).unwrap_or_else(|| panic!("Failed to map provisional entity {:?}", &self.particle_b_prov)).clone();
    }
}

// ParticleCollision

#[derive(Clone, Debug)]
pub(in crate) struct ParticleCollision {
    pub(in crate) particle_a: Entity,
    pub(in crate) particle_b: Option<Entity>, // None indicates particle_2 has infinite mass, i.e. immovable
    pub(in crate) restitution: f32,
    pub(in crate) normal: Vec3,
    pub(in crate) penetration: f32,
}

impl ParticleCollision {
    pub(in crate) fn new(
        particle_a: Entity,
        particle_b: Option<Entity>,
        restitution: f32,
        normal: Vec3,
        penetration: f32,
    ) -> Self {
        Self {
            particle_a,
            particle_b,
            restitution,
            normal,
            penetration,
        }
    }
}

impl Component for ParticleCollision {}
impl ComponentActions for ParticleCollision {}

// ParticleCollisionDetector

#[derive(Clone, Debug)]
pub struct ParticleCollisionDetector {
    pub default_restitution: f32,
}

impl ParticleCollisionDetector {
    pub fn new(default_restitution: f32) -> Self {
        Self { default_restitution }
    }
}

impl Component for ParticleCollisionDetector {}
impl ComponentActions for ParticleCollisionDetector {}
