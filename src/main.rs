use core::{Event, Node, Scene, Viewport2D};
use drivers::vulkan_render_engine::VulkanRenderEngine;
use render_engine::{RenderEngine, RenderEngineProperties, Window, WindowProperties};

mod core;
mod drivers;
mod math;
mod render_engine;

fn main() {
    let render_engine_properties = RenderEngineProperties {
        debug_enabled: true,
        window_properties: WindowProperties {
            width: 800,
            height: 600,
            title: "My Cool Game".to_string(),
        },
    };

    let mut render_engine = VulkanRenderEngine::new(&render_engine_properties)
        .unwrap_or_else(|_| panic!("Failed to init VulkanRenderEngine"));
    let mut scene = init_scene();

    run_game_loop(&mut render_engine, &mut scene);
}

fn init_scene() -> Scene<'static> {
    let node_0 = Node::default();

    Scene::new(vec![node_0], vec![Viewport2D::default()])
}

fn run_game_loop(render_engine: &mut VulkanRenderEngine, scene: &mut Scene) {
    while !render_engine.get_window().is_closing() {
        scene.fire_event(&Event::Update);

        render_engine.sync_data(scene);
    }
}
