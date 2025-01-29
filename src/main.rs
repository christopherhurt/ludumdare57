use core::{Behavior, ColorMaterial, Scene, System, Transform, Viewport2D};
use drivers::vulkan::{VulkanRenderer, VulkanWindow};
use ecs::ECS;
use renderer::{Renderer, Window};

mod core;
mod drivers;
mod ecs;
mod math;
mod renderer;

fn main() {
    let ecs = init_ecs();
    let scene = init_scene(ecs);
    let renderer = VulkanRenderer::<VulkanWindow>::get_instance();

    run_game_loop(&scene, &renderer);
}

fn init_ecs() -> ECS {
    let mut ecs = ECS::new();

    let transform_bit = ecs.register_component::<Transform>().unwrap_or_else(|_| panic!("Failed to register Transform component"));
    // TODO fix type issue
    let behavior_bit = ecs.register_component::<Behavior<_>>().unwrap_or_else(|_| panic!("Failed to register Behavior component"));
    let color_material_bit = ecs.register_component::<ColorMaterial>().unwrap_or_else(|_| panic!("Failed to register ColorMaterial component"));

    // TODO

    ecs
}

fn init_scene(ecs: ECS) -> Scene {
    let scene = Scene::new(ecs, vec![Viewport2D::default()]);

    // TODO: populate scene

    scene
}

fn run_game_loop<W: Window, R: Renderer<W>>(scene: &Scene, renderer: &R) {
    while !renderer.get_window().should_close() {
        // Behavior system
        let behavior_entities = scene.ecs.get_system_entities(System::Behavior.get_id())
            .unwrap_or_else(|_| panic!("Failed to get entities for the Behavior system"));

        behavior_entities.for_each(|e| {
            // TODO fix
            let behavior = scene.ecs.get_component::<Behavior>(*e)
                .unwrap_or_else(|_| panic!("Failed to get Behavior component for entity {}", e));

            behavior.on_update(scene, e);
        });

        // Render system
        let render_entities = scene.ecs.get_system_entities(System::Render.get_id())
            .unwrap_or_else(|_| panic!("Failed to get entities for the Render system"));

        render_entities.for_each(|e| {
            // TODO
        });
    }
}
