use crate::ecs::component::Component;
use crate::math::{Vec3, VEC_3_ZERO};

// NOTE: This isn't useful without a dependent Transform component
#[derive(Clone, Debug)]
pub struct Particle {
    pub vel: Vec3,
    pub acc: Vec3,
    pub damping: f32,
    pub mass: f32,
    pub gravity: f32,
}

impl Particle {
    pub fn new(vel: Vec3, acc: Vec3, damping: f32, mass: f32, gravity: f32) -> Self {
        Self { vel, acc, damping, mass, gravity }
    }
}

impl Default for Particle {
    fn default() -> Self {
        Self {
            vel: VEC_3_ZERO,
            acc: VEC_3_ZERO,
            damping: 1.0,
            mass: 1.0,
            gravity: 0.0,
        }
    }
}

// TODO: remove this module's dependency on the ecs module?
impl Component for Particle {}
