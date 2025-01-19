use std::f32;

use crate::ecs::Component;

#[derive(Clone, Debug)]
#[repr(C)]
pub struct Transform {
    pub pos: cgmath::Vector3<f32>,
    pub rot: cgmath::Vector3<f32>,
    pub scale: cgmath::Vector3<f32>,
}

impl Component for Transform {}
