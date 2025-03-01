use anyhow::Result;
use render_engine::{RenderEngine, Window};
use std::collections::hash_set::Iter;
use std::collections::HashSet;

use crate::component_bindings::{Mesh, VulkanComponent};
use crate::core::{ColorMaterial, Transform, Viewport2D};
use crate::ecs::component::ComponentManager;
use crate::ecs::entity::Entity;
use crate::ecs::system::System;
use crate::ecs::{ECSBuilder, ECSCommands, ECS};
use crate::render_engine::vulkan::VulkanRenderEngine;
use crate::render_engine::{RenderEngineInitProps, WindowInitProps};

pub mod component_bindings;
pub mod core;
pub mod ecs;
pub mod math;
pub mod render_engine;

fn main() {
    pretty_env_logger::init();

    let mut ecs = init_ecs();
    create_scene(&mut ecs);

    while ecs.invoke_systems() {}
}

fn init_ecs() -> ECS {
    ECSBuilder::with_initial_entity_capacity(1_024)
        .with_component::<Viewport2D>()
        .with_component::<Transform>()
        .with_component::<Mesh>()
        .with_component::<ColorMaterial>()
        .with_component::<VulkanComponent>()
        .build()
}

fn init_render_engine() -> Result<VulkanRenderEngine> {
    let window_props = WindowInitProps {
        width: 800,
        height: 600,
        title: String::from("My Cool Game"),
    };

    let render_engine_props = RenderEngineInitProps {
        debug_enabled: true,
        window_props,
    };

    VulkanRenderEngine::new(render_engine_props)
}

fn create_scene(ecs: &mut ECS) {
    let mut render_engine = init_render_engine().unwrap_or_else(|e| panic!("{}", e));

    ecs.register_system(SHUTDOWN_ECS, HashSet::from([ecs.get_system_signature_1::<VulkanComponent>().unwrap()]), -999);

    // TODO

    ecs.register_system(SHUTDOWN_RENDER_ENGINE, HashSet::from([ecs.get_system_signature_1::<VulkanComponent>().unwrap()]), 999);
}

const SHUTDOWN_ECS: System = |entites: Iter<Entity>, components: &mut ComponentManager, commands: &mut ECSCommands| {
    entites.for_each(|e| {
        let vulkan = components.get_component::<VulkanComponent>(e).unwrap();

        if vulkan.render_engine.get_window().map_or(true, |w| w.is_closing()) {
            commands.shutdown();
        }
    });
};

const SHUTDOWN_RENDER_ENGINE: System = |entites: Iter<Entity>, components: &mut ComponentManager, commands: &mut ECSCommands| {
    if commands.is_shutting_down() {
        entites.for_each(|e| {
            let vulkan = components.get_mut_component::<VulkanComponent>(e).unwrap();

            unsafe {
                vulkan.render_engine.join_render_thread()
                    .unwrap_or_else(|e| panic!("{}", e));
            }
        });
    }
};
