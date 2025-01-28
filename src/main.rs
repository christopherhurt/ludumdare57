use cgmath::vec3;
use core::{ColorMaterial, Transform};
use ecs::ECS;

mod core;
mod ecs;

fn main() {
    // TODO

    let transform = Transform {
        pos: vec3(0.0, 0.0, 0.0),
        rot: vec3(0.0, 0.0, 0.0),
        scl: vec3(0.0, 0.0, 0.0),
    };
}

fn init_ecs() -> ECS {
    let mut ecs = ECS::new();

    let transform_bit = ecs.register_component::<Transform>().unwrap_or_else(|_| panic!("Failed to register Transform component"));
    let color_material_bit = ecs.register_component::<ColorMaterial>().unwrap_or_else(|_| panic!("Failed to register ColorMaterial component"));

    // TODO

    ecs
}
