mod ecs;

use cgmath::vec3;

use crate::ecs::ComponentArray;
use crate::ecs::components::transform::Transform;

fn main() {
    let mut comp_arr = ComponentArray::<Transform>::new();

    comp_arr.insert(0, create_transform(0.0));
    comp_arr.insert(1, create_transform(1.0));
    comp_arr.insert(2, create_transform(2.0));
    comp_arr.insert(3, create_transform(3.0));

    comp_arr.remove(0);
    comp_arr.remove(2);

    println!("comp_arr: {:?}", comp_arr);

    println!("entity 1: {:?}", comp_arr.get(1));
    println!("entity 3: {:?}", comp_arr.get(3));
}

fn create_transform(val: f32) -> Transform {
    Transform {
        pos: vec3(val, val, val),
        rot: vec3(val, val, val),
        scale: vec3(val, val, val),
    }
}
