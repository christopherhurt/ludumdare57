use cgmath::vec3;
use core::transform::Transform;
use ecs::ECS;

mod core;
mod ecs;

fn main() {
    let mut ecs = ECS::new();

    let transform_bit = ecs.register_component::<Transform>().unwrap_or_else(|_| panic!("Failed to register Transform component"));

    // TODO
    let transform = Transform {
        pos: vec3(0.0, 0.0, 0.0),
        rot: vec3(0.0, 0.0, 0.0),
        scl: vec3(0.0, 0.0, 0.0),
    };
}
