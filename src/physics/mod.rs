use crate::ecs::component::Component;
use crate::ecs::entity::Entity;
use crate::ecs::ProvisionalEntity;
use crate::math::{Vec3, VEC_3_ZERO};

// Particle

#[derive(Clone, Debug)]
pub struct Particle {
    pub vel: Vec3,
    pub acc: Vec3,
    pub damping: f32,
    // TODO: consider including gravity value here instead of as its own system?
    // TODO: make mass an Option<f32> value where None indicates the particle has infinite mass, i.e. immovable
    pub mass: f32,
    pub force_accum: Vec3,
    // TODO: add material which can be used to derive the coefficient of restitution - for now, it's being generated randomly for each collision
}

impl Particle {
    pub fn new(vel: Vec3, damping: f32, mass: f32) -> Self {
        Self {
            vel,
            acc: VEC_3_ZERO,
            damping,
            mass,
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
            force_accum: VEC_3_ZERO,
        }
    }
}

// TODO: remove this module's dependency on the ecs module?
impl Component for Particle {}

// ParticleCable

// TODO: how to handle this paradigm for ProvisionalEntity??
#[derive(Clone, Debug)]
pub struct ParticleCable {
    pub particle_a: Entity,
    pub particle_b: Entity,
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
            max_length,
            restitution,
        }
    }
}

impl Component for ParticleCable {}

// ParticleRod

#[derive(Clone, Debug)]
pub struct ParticleRod {
    pub particle_a: Entity,
    pub particle_b: Entity,
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
            length,
        }
    }
}

impl Component for ParticleRod {}

// ParticleCollision

#[derive(Clone, Debug)]
pub(in crate) struct ParticleCollision {
    // TODO: Using copies of Entity and not references leaves open the possibility that particle_a or particle_b are destroyed before
    //  this collision is resolved. To prevent this, we should 1) lift the restriction that particle_a and particle_b must exist at
    //  collision resolution time, or otherwise panic, or 2) control the ordering of built-in and user defined systems such that
    //  motion collision resolution, i.e. applied impulses, always happens before user-defined collision resolutions. Though, also,
    //  the user should not be dumb and destroy the particle_a or particle_b entity without destroying the ParticleCollision entity...
    //  or 3) maybe some other approach...
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

// ParticleCollisionResolver

#[derive(Clone, Debug)]
pub struct ParticleCollisionResolver {
    // TODO: Make this Option<f32> where a value of None indicates to continue the resolver until all velocities and interpenetrations are resolved
    pub num_iterations_factor: f32,
}

impl ParticleCollisionResolver {
    pub fn new(num_iterations_factor: f32) -> Self {
        Self { num_iterations_factor }
    }
}

impl Component for ParticleCollisionResolver {}
