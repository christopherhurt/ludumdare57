use anyhow::Result;
use math::{get_proj_matrix, vec2, vec3, Quat, VEC_2_ZERO, VEC_3_Y_AXIS, VEC_3_ZERO, VEC_3_Z_AXIS};
use core::Color;
use std::collections::hash_set::Iter;
use std::collections::HashSet;

use crate::component_bindings::{Mesh, VulkanComponent};
use crate::core::{Camera, ColorMaterial, TimeDelta, Transform, Viewport2D, YELLOW};
use crate::ecs::component::ComponentManager;
use crate::ecs::entity::Entity;
use crate::ecs::system::System;
use crate::ecs::{ECSBuilder, ECSCommands, ECS};
use crate::render_engine::vulkan::VulkanRenderEngine;
use crate::render_engine::{Device, EntityRenderState, RenderEngine, RenderState, Window, RenderEngineInitProps, VirtualKey, WindowInitProps};

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
        .with_component::<TimeDelta>()
        .build()
}

fn init_render_engine() -> Result<VulkanRenderEngine> {
    let window_props = WindowInitProps {
        width: 1600,
        height: 1200,
        title: String::from("My Cool Game"),
    };

    let render_engine_props = RenderEngineInitProps {
        debug_enabled: true,
        clear_color: Color::rgb(0.0, 0.3, 0.0),
        window_props,
    };

    unsafe { VulkanRenderEngine::new(render_engine_props) }
}

fn create_scene(ecs: &mut ECS) {
    let mut render_engine = init_render_engine().unwrap_or_else(|e| panic!("{}", e));

    let cam = Camera::new(VEC_3_ZERO, VEC_3_Z_AXIS, VEC_3_Y_AXIS, 70.0);
    let viewport = Viewport2D::new(cam, VEC_2_ZERO, vec2(1.0, 1.0));
    let player_entity = ecs.create_entity();
    ecs.attach_provisional_component(&player_entity, viewport);

    let cube_positions = vec![vec3(0.0, -0.5, 1.0), vec3(0.5, 0.5, 1.0), vec3(-0.5, 0.5, 1.0)]; // TODO
    let cube_indexes = vec![0, 1, 2]; // TODO
    let cube_mesh_id = render_engine.get_device_mut()
        .and_then(|d| unsafe { d.create_mesh(cube_positions, cube_indexes) })
        .unwrap_or_else(|e| panic!("{}", e));
    let cube_mesh = Mesh::new(cube_mesh_id);
    let cube_transform = Transform::new(
        vec3(0.0, 0.0, 5.0),
        Quat::from_axis_spin(&VEC_3_Y_AXIS, 0.0).unwrap(),
        vec3(1.0, 1.0, 1.0),
    );
    let cube_color_material = ColorMaterial::new(YELLOW);
    let cube_entity = ecs.create_entity();
    ecs.attach_provisional_component(&cube_entity, cube_mesh);
    ecs.attach_provisional_component(&cube_entity, cube_transform);
    ecs.attach_provisional_component(&cube_entity, cube_color_material);

    let vulkan = VulkanComponent::new(render_engine);
    let vulkan_entity = ecs.create_entity();
    ecs.attach_provisional_component(&vulkan_entity, vulkan);

    let time_delta = TimeDelta::default();
    let time_delta_entity = ecs.create_entity();
    ecs.attach_provisional_component(&time_delta_entity, time_delta);

    ecs.register_system(SHUTDOWN_ECS, HashSet::from([ecs.get_system_signature_1::<VulkanComponent>().unwrap()]), -999);
    ecs.register_system(TIME_SINCE_LAST_FRAME, HashSet::from([ecs.get_system_signature_1::<TimeDelta>().unwrap()]), -500);
    ecs.register_system(MOVE_CAMERA, HashSet::from([ecs.get_system_signature_1::<VulkanComponent>().unwrap(), ecs.get_system_signature_1::<Viewport2D>().unwrap(), ecs.get_system_signature_1::<TimeDelta>().unwrap()]), 0);
    ecs.register_system(MOVE_CUBE, HashSet::from([ecs.get_system_signature_1::<Transform>().unwrap(), ecs.get_system_signature_1::<TimeDelta>().unwrap()]), 1);
    ecs.register_system(SYNC_RENDER_STATE, HashSet::from([ecs.get_system_signature_0().unwrap()]), 2);
    ecs.register_system(SHUTDOWN_RENDER_ENGINE, HashSet::from([ecs.get_system_signature_1::<VulkanComponent>().unwrap()]), 999);
}

const SHUTDOWN_ECS: System = |entites: Iter<Entity>, components: &mut ComponentManager, commands: &mut ECSCommands| {
    entites.for_each(|e| {
        let vulkan = components.get_component::<VulkanComponent>(e).unwrap();

        if vulkan.render_engine.get_window().map_or(true, |w| w.is_key_down(VirtualKey::Space) || w.is_closing()) {
            commands.shutdown();
        }
    });
};

const TIME_SINCE_LAST_FRAME: System = |entites: Iter<Entity>, components: &mut ComponentManager, _: &mut ECSCommands| {
    entites.for_each(|e| {
        let time_delta = components.get_mut_component::<TimeDelta>(e).unwrap();

        if time_delta.is_started {
            let now = std::time::SystemTime::now();
            time_delta.since_last_frame = now.duration_since(time_delta.timestamp).unwrap();
            time_delta.timestamp = now;
        } else {
            time_delta.is_started = true;
            time_delta.timestamp = std::time::SystemTime::now();
        }
    });
};

const MOVE_CAMERA: System = |entites: Iter<Entity>, components: &mut ComponentManager, _: &mut ECSCommands| {
    let vulkan = entites.clone().find_map(|e| components.get_component::<VulkanComponent>(e)).unwrap();
    let time_delta = entites.clone().find_map(|e| components.get_component::<TimeDelta>(e)).unwrap();

    if let Ok(window) = vulkan.render_engine.get_window() {
        for e in entites {
            if let Some(viewport) = components.get_mut_component::<Viewport2D>(e) {
                let cam = &mut viewport.cam;

                let mut move_dir = VEC_3_ZERO;
                let cam_right_norm = cam.dir.cross(&cam.up).normalized().unwrap();

                if window.is_key_down(VirtualKey::W) {
                    move_dir += cam.dir.normalized().unwrap();
                }
                if window.is_key_down(VirtualKey::S) {
                    move_dir -= cam.dir.normalized().unwrap();
                }
                if window.is_key_down(VirtualKey::D) {
                    move_dir += cam_right_norm;
                }
                if window.is_key_down(VirtualKey::A) {
                    move_dir -= cam_right_norm;
                }
                if window.is_key_down(VirtualKey::Q) {
                    move_dir += cam.up.normalized().unwrap();
                }
                if window.is_key_down(VirtualKey::E) {
                    move_dir -= cam.up.normalized().unwrap();
                }

                let move_speed = 10.0 * time_delta.since_last_frame.as_secs_f32();
                if let Ok(dir) = move_dir.normalized() {
                    cam.pos += dir * move_speed;
                }

                let rot_speed = 150.0 * time_delta.since_last_frame.as_secs_f32();
                if window.is_key_down(VirtualKey::Left) && !window.is_key_down(VirtualKey::Right) {
                    cam.dir = cam.dir.rotated(&cam.up, rot_speed).normalized().unwrap();
                }
                if window.is_key_down(VirtualKey::Right) && !window.is_key_down(VirtualKey::Left) {
                    cam.dir = cam.dir.rotated(&cam.up, -rot_speed).normalized().unwrap();
                }
                if window.is_key_down(VirtualKey::Up) && !window.is_key_down(VirtualKey::Down) {
                    cam.dir = cam.dir.rotated(&cam_right_norm, rot_speed).normalized().unwrap();
                    cam.up = cam.up.rotated(&cam_right_norm, rot_speed).normalized().unwrap();
                }
                if window.is_key_down(VirtualKey::Down) && !window.is_key_down(VirtualKey::Up) {
                    cam.dir = cam.dir.rotated(&cam_right_norm, -rot_speed).normalized().unwrap();
                    cam.up = cam.up.rotated(&cam_right_norm, -rot_speed).normalized().unwrap();
                }
            }
        }
    }
};

const MOVE_CUBE: System = |entites: Iter<Entity>, components: &mut ComponentManager, _: &mut ECSCommands| {
    let time_delta = entites.clone().find_map(|e| components.get_component::<TimeDelta>(e)).unwrap();
    let time_delta_sec = time_delta.since_last_frame.as_secs_f32();

    for e in entites {
        if let Some(transform) = components.get_mut_component::<Transform>(e) {
            let spin = Quat::from_axis_spin(&VEC_3_Y_AXIS, 5.0 * time_delta_sec).unwrap();
            transform.rot *= spin;
        }
    }
};

const SYNC_RENDER_STATE: System = |entites: Iter<Entity>, components: &mut ComponentManager, _: &mut ECSCommands| {
    let vulkan = entites.clone().find_map(|e| components.get_mut_component::<VulkanComponent>(e)).unwrap();
    let viewport = entites.clone().find_map(|e| components.get_component::<Viewport2D>(e)).unwrap();

    let entity_states = entites.clone().filter(|e|
        components.get_component::<Transform>(e).is_some()
        && components.get_component::<Mesh>(e).is_some()
        && components.get_component::<ColorMaterial>(e).is_some())
    .map(|e| EntityRenderState {
        world: components.get_component::<Transform>(e).unwrap().to_world_mat(),
        mesh_id: components.get_component::<Mesh>(e).unwrap().id,
        color: components.get_component::<ColorMaterial>(e).unwrap().color,
    }).collect();

    let aspect_ratio = vulkan.render_engine.get_window().and_then(|w| {
        Ok((w.get_width() as f32) / (w.get_height() as f32))
    }).unwrap_or(1.0);
    let proj = get_proj_matrix(0.01, 1000.0, viewport.cam.fov_deg, aspect_ratio).unwrap();

    let render_state = RenderState {
        view: viewport.cam.to_view_mat().unwrap(),
        proj,
        entity_states,
    };

    vulkan.render_engine.sync_state(render_state).unwrap_or_default();
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
