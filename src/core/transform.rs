use crate::ecs::Component;

use cgmath::Vector3;

#[derive(Debug)]
pub struct Transform {
    pub pos: Vector3<f32>,
    pub rot: Vector3<f32>,
    pub scl: Vector3<f32>,
}

impl Component for Transform {}
