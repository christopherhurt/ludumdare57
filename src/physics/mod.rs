use crate::ecs::component::Component;
use crate::math::{Vec3, VEC_3_ZERO};

// NOTE: This isn't useful without a dependent Transform component
#[derive(Clone, Debug)]
pub struct Particle {
    pub vel: Vec3,
    pub acc: Vec3,
    pub damping: f32,
    pub mass: f32,
    pub force_accum: Vec3,
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
