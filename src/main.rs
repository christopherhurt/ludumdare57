use core::{Node, Scene, Viewport2D};
use drivers::vulkan_render_engine::VulkanRenderEngine;
use render_engine::{RenderEngine, RenderEngineInitProperties, Window, WindowInitProperties};

pub mod core;
pub mod drivers;
pub mod math;
pub mod render_engine;

fn main() {
    pretty_env_logger::init();

    let render_engine_properties = RenderEngineInitProperties {
        debug_enabled: true,
        window_properties: WindowInitProperties {
            width: 800,
            height: 600,
            title: "My Cool Game".to_string(),
        },
    };

    let mut render_engine = VulkanRenderEngine::new(render_engine_properties)
        .unwrap_or_else(|_| panic!("Failed to init VulkanRenderEngine"));
    let mut scene = init_scene();

    run_game_loop(&mut render_engine, &mut scene);
}

fn init_scene() -> Scene {
    let mut scene = Scene::new(vec![Viewport2D::default()]);

    // TODO
    scene.add_node(Node::default());

    scene
}

fn run_game_loop(render_engine: &mut VulkanRenderEngine, scene: &mut Scene) {
    while !render_engine.get_window().is_closing() {
        // TODO: game logic

        render_engine.sync_data(scene).unwrap_or_else(|_| panic!("Failed to sync data with render engine"));
    }

    render_engine.join_render_thread().unwrap();
}
