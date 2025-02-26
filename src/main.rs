use anyhow::Result;
use std::collections::hash_set::Iter;
use std::collections::HashSet;

use crate::core::{ColorMaterial, Mesh, Transform, Viewport2D};
use crate::ecs::component::ComponentManager;
use crate::ecs::entity::Entity;
use crate::ecs::system::System;
use crate::ecs::{ECSBuilder, ECSCommands, ECS};
use crate::render_engine::vulkan::VulkanRenderEngine;
use crate::render_engine::{RenderEngineInitProps, WindowInitProps};

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
        .with_component::<VulkanRenderEngine>()
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

    // TODO

    ecs.register_system(shutdown_render_engine, HashSet::from([ecs.get_system_signature_1::<VulkanRenderEngine>().unwrap()]), 999);
}

const shutdown_render_engine: System = |entites: &Iter<Entity>, components: &mut ComponentManager, commands: &mut ECSCommands| {
    // TODO: need to both check and call ECS shutdown? as well as render engine shutdown...
};
