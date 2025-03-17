use anyhow::Result;
use math::{get_proj_matrix, vec2, vec3, Quat, VEC_2_ZERO, VEC_3_X_AXIS, VEC_3_Y_AXIS, VEC_3_ZERO, VEC_3_Z_AXIS};
use std::collections::hash_set::Iter;
use std::collections::HashSet;

use crate::component_bindings::{Mesh, VulkanComponent};
use crate::core::{Camera, Color, ColorMaterial, TimeDelta, Transform, Viewport2D, BLUE, ORANGE, PURPLE, RED, YELLOW};
use crate::ecs::component::{Component, ComponentManager};
use crate::ecs::entity::Entity;
use crate::ecs::system::System;
use crate::ecs::{ECSBuilder, ECSCommands, ECS};
use crate::physics::Particle;
use crate::render_engine::vulkan::VulkanRenderEngine;
use crate::render_engine::{Device, EntityRenderState, MeshId, RenderEngine, RenderState, Window, RenderEngineInitProps, Vertex, VirtualKey, WindowInitProps};

pub mod component_bindings;
pub mod core;
pub mod ecs;
pub mod math;
pub mod physics;
pub mod render_engine;

const DAMPING: f32 = 0.999;

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
        .with_component::<Particle>()
        .with_component::<MeshWrapper>()
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

    let cube_vertices = vec![
        // Front
        Vertex { pos: vec3(-0.5, -0.5, -0.5), norm: vec3(0.0, 0.0, -1.0) },
        Vertex { pos: vec3(-0.5, 0.5, -0.5), norm: vec3(0.0, 0.0, -1.0) },
        Vertex { pos: vec3(0.5, 0.5, -0.5), norm: vec3(0.0, 0.0, -1.0) },
        Vertex { pos: vec3(0.5, -0.5, -0.5), norm: vec3(0.0, 0.0, -1.0) },
        // Left
        Vertex { pos: vec3(-0.5, -0.5, 0.5), norm: vec3(-1.0, 0.0, 0.0) },
        Vertex { pos: vec3(-0.5, 0.5, 0.5), norm: vec3(-1.0, 0.0, 0.0) },
        Vertex { pos: vec3(-0.5, 0.5, -0.5), norm: vec3(-1.0, 0.0, 0.0) },
        Vertex { pos: vec3(-0.5, -0.5, -0.5), norm: vec3(-1.0, 0.0, 0.0) },
        // Back
        Vertex { pos: vec3(0.5, -0.5, 0.5), norm: vec3(0.0, 0.0, 1.0) },
        Vertex { pos: vec3(0.5, 0.5, 0.5), norm: vec3(0.0, 0.0, 1.0) },
        Vertex { pos: vec3(-0.5, 0.5, 0.5), norm: vec3(0.0, 0.0, 1.0) },
        Vertex { pos: vec3(-0.5, -0.5, 0.5), norm: vec3(0.0, 0.0, 1.0) },
        // Right
        Vertex { pos: vec3(0.5, -0.5, -0.5), norm: vec3(1.0, 0.0, 0.0) },
        Vertex { pos: vec3(0.5, 0.5, -0.5), norm: vec3(1.0, 0.0, 0.0) },
        Vertex { pos: vec3(0.5, 0.5, 0.5), norm: vec3(1.0, 0.0, 0.0) },
        Vertex { pos: vec3(0.5, -0.5, 0.5), norm: vec3(1.0, 0.0, 0.0) },
        // Top
        Vertex { pos: vec3(-0.5, 0.5, -0.5), norm: vec3(0.0, 1.0, 0.0) },
        Vertex { pos: vec3(-0.5, 0.5, 0.5), norm: vec3(0.0, 1.0, 0.0) },
        Vertex { pos: vec3(0.5, 0.5, 0.5), norm: vec3(0.0, 1.0, 0.0) },
        Vertex { pos: vec3(0.5, 0.5, -0.5), norm: vec3(0.0, 1.0, 0.0) },
        // Down
        Vertex { pos: vec3(-0.5, -0.5, 0.5), norm: vec3(0.0, -1.0, 0.0) },
        Vertex { pos: vec3(-0.5, -0.5, -0.5), norm: vec3(0.0, -1.0, 0.0) },
        Vertex { pos: vec3(0.5, -0.5, -0.5), norm: vec3(0.0, -1.0, 0.0) },
        Vertex { pos: vec3(0.5, -0.5, 0.5), norm: vec3(0.0, -1.0, 0.0) },
    ];
    let cube_indexes = vec![
        // Front
        0, 1, 2, 2, 3, 0,
        // Left
        4, 5, 6, 6, 7, 4,
        // Back
        8, 9, 10, 10, 11, 8,
        // Right
        12, 13, 14, 14, 15, 12,
        // Top
        16, 17, 18, 18, 19, 16,
        // Down
        20, 21, 22, 22, 23, 20,
    ];
    let cube_mesh_id = render_engine.get_device_mut()
        .and_then(|d| unsafe { d.create_mesh(cube_vertices, cube_indexes) })
        .unwrap_or_else(|e| panic!("{}", e));
    let cube_mesh = Mesh::new(cube_mesh_id);
    let cube_transform = Transform::new(
        vec3(-10.0, 0.0, 10.0),
        Quat::from_axis_spin(&VEC_3_Y_AXIS, 0.0).unwrap(),
        vec3(1.0, 1.0, 1.0),
    );
    let cube_color_material = ColorMaterial::new(YELLOW);
    let cube_particle = Particle::new(VEC_3_ZERO, DAMPING, 1.0);
    let cube_entity = ecs.create_entity();
    ecs.attach_provisional_component(&cube_entity, cube_mesh);
    ecs.attach_provisional_component(&cube_entity, cube_transform);
    ecs.attach_provisional_component(&cube_entity, cube_color_material);
    ecs.attach_provisional_component(&cube_entity, cube_particle);

    let cube_2_mesh = Mesh::new(cube_mesh_id);
    let cube_2_transform = Transform::new(
        vec3(0.0, 0.0, 10.0),
        Quat::from_axis_spin(&VEC_3_Y_AXIS, 0.0).unwrap(),
        vec3(3.0, 3.0, 3.0),
    );
    let cube_2_color_material = ColorMaterial::new(ORANGE);
    let cube_2_particle = Particle::new(VEC_3_ZERO, DAMPING, 5.0);
    let cube_2_entity = ecs.create_entity();
    ecs.attach_provisional_component(&cube_2_entity, cube_2_mesh);
    ecs.attach_provisional_component(&cube_2_entity, cube_2_transform);
    ecs.attach_provisional_component(&cube_2_entity, cube_2_color_material);
    ecs.attach_provisional_component(&cube_2_entity, cube_2_particle);

    let cube_3_mesh = Mesh::new(cube_mesh_id);
    let cube_3_transform = Transform::new(
        vec3(10.0, 0.0, 10.0),
        Quat::from_axis_spin(&VEC_3_Y_AXIS, 0.0).unwrap(),
        vec3(8.0, 8.0, 8.0),
    );
    let cube_3_color_material = ColorMaterial::new(RED);
    let cube_3_particle = Particle::new(VEC_3_ZERO, DAMPING, 12.0);
    let cube_3_entity = ecs.create_entity();
    ecs.attach_provisional_component(&cube_3_entity, cube_3_mesh);
    ecs.attach_provisional_component(&cube_3_entity, cube_3_transform);
    ecs.attach_provisional_component(&cube_3_entity, cube_3_color_material);
    ecs.attach_provisional_component(&cube_3_entity, cube_3_particle);

    let cube_mesh_wrapper = MeshWrapper { my_id: 0, id: cube_mesh_id };
    let cube_mesh_wrapper_entity = ecs.create_entity();
    ecs.attach_provisional_component(&cube_mesh_wrapper_entity, cube_mesh_wrapper);

    let vulkan = VulkanComponent::new(render_engine);
    let vulkan_entity = ecs.create_entity();
    ecs.attach_provisional_component(&vulkan_entity, vulkan);

    let time_delta = TimeDelta::default();
    let time_delta_entity = ecs.create_entity();
    ecs.attach_provisional_component(&time_delta_entity, time_delta);

    ecs.register_system(SHUTDOWN_ECS, HashSet::from([ecs.get_system_signature_1::<VulkanComponent>().unwrap()]), -999);
    ecs.register_system(TIME_SINCE_LAST_FRAME, HashSet::from([ecs.get_system_signature_1::<TimeDelta>().unwrap()]), -500);
    ecs.register_system(MOVE_CAMERA, HashSet::from([ecs.get_system_signature_1::<VulkanComponent>().unwrap(), ecs.get_system_signature_1::<Viewport2D>().unwrap(), ecs.get_system_signature_1::<TimeDelta>().unwrap()]), -400);
    ecs.register_system(PUSH_CUBES, HashSet::from([ecs.get_system_signature_3::<Transform, Particle, ColorMaterial>().unwrap(), ecs.get_system_signature_1::<Viewport2D>().unwrap()]), -350);
    ecs.register_system(TURN_CUBES, HashSet::from([ecs.get_system_signature_1::<VulkanComponent>().unwrap(), ecs.get_system_signature_1::<Transform>().unwrap(), ecs.get_system_signature_1::<TimeDelta>().unwrap()]), -300);
    ecs.register_system(SHOOT_PROJECTILE, HashSet::from([ecs.get_system_signature_1::<VulkanComponent>().unwrap(), ecs.get_system_signature_1::<MeshWrapper>().unwrap(), ecs.get_system_signature_1::<Viewport2D>().unwrap()]), -250);
    ecs.register_system(UPDATE_PARTICLES, HashSet::from([ecs.get_system_signature_2::<Transform, Particle>().unwrap(), ecs.get_system_signature_1::<TimeDelta>().unwrap()]), -200);
    ecs.register_system(SYNC_RENDER_STATE, HashSet::from([ecs.get_system_signature_0().unwrap()]), 2);
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
                    cam.dir = cam.dir.rotated(&VEC_3_Y_AXIS, rot_speed).unwrap().normalized().unwrap();
                    cam.up = cam.up.rotated(&VEC_3_Y_AXIS, rot_speed).unwrap().normalized().unwrap();
                }
                if window.is_key_down(VirtualKey::Right) && !window.is_key_down(VirtualKey::Left) {
                    cam.dir = cam.dir.rotated(&VEC_3_Y_AXIS, -rot_speed).unwrap().normalized().unwrap();
                    cam.up = cam.up.rotated(&VEC_3_Y_AXIS, -rot_speed).unwrap().normalized().unwrap();
                }
                if window.is_key_down(VirtualKey::Up) && !window.is_key_down(VirtualKey::Down) {
                    cam.dir = cam.dir.rotated(&cam_right_norm, rot_speed).unwrap().normalized().unwrap();
                    cam.up = cam.up.rotated(&cam_right_norm, rot_speed).unwrap().normalized().unwrap();
                }
                if window.is_key_down(VirtualKey::Down) && !window.is_key_down(VirtualKey::Up) {
                    cam.dir = cam.dir.rotated(&cam_right_norm, -rot_speed).unwrap().normalized().unwrap();
                    cam.up = cam.up.rotated(&cam_right_norm, -rot_speed).unwrap().normalized().unwrap();
                }
            }
        }
    }
};

const SHOOT_PROJECTILE: System = |entites: Iter<Entity>, components: &mut ComponentManager, commands: &mut ECSCommands| {
    let vulkan = entites.clone().find_map(|e| components.get_component::<VulkanComponent>(e)).unwrap();
    let mesh_id = entites.clone().find_map(|e|
        components.get_component::<MeshWrapper>(e).filter(|m| m.my_id == 0)
    ).unwrap().id;
    let cam = &entites.clone().find_map(|e| components.get_component::<Viewport2D>(e)).unwrap().cam;

    if vulkan.render_engine.is_key_down(VirtualKey::Space) {
        let cam_dir_norm = cam.dir.normalized().unwrap();

        let mesh = Mesh::new(mesh_id);
        let color_material = ColorMaterial::new(PURPLE);
        let transform = Transform::new(cam.pos + cam_dir_norm * 5.0, Quat::from_axis_spin(&VEC_3_X_AXIS, 0.0).unwrap(), vec3(1.0, 1.0, 1.0));
        let particle = Particle::new(cam_dir_norm * 35.0, DAMPING, 5.0);

        let proj_entity = commands.create_entity();
        commands.attach_provisional_component(&proj_entity, mesh);
        commands.attach_provisional_component(&proj_entity, color_material);
        commands.attach_provisional_component(&proj_entity, transform);
        commands.attach_provisional_component(&proj_entity, particle);
    } else if vulkan.render_engine.is_key_down(VirtualKey::Enter) {
        let cam_dir_norm = cam.dir.normalized().unwrap();

        let mesh = Mesh::new(mesh_id);
        let color_material = ColorMaterial::new(BLUE);
        let transform = Transform::new(cam.pos + cam_dir_norm * 5.0, Quat::from_axis_spin(&VEC_3_X_AXIS, 0.0).unwrap(), vec3(3.0, 3.0, 3.0));
        let particle = Particle::new(cam_dir_norm * 5.0, 0.9, 1.0);

        let proj_entity = commands.create_entity();
        commands.attach_provisional_component(&proj_entity, mesh);
        commands.attach_provisional_component(&proj_entity, color_material);
        commands.attach_provisional_component(&proj_entity, transform);
        commands.attach_provisional_component(&proj_entity, particle);
    }
};

const UPDATE_PARTICLES: System = |entites: Iter<Entity>, components: &mut ComponentManager, _: &mut ECSCommands| {
    let time_delta = entites.clone().find_map(|e| components.get_component::<TimeDelta>(e)).unwrap();
    let delta = time_delta.since_last_frame.as_secs_f32();

    for e in entites {
        let transform = components.get_mut_component::<Transform>(e);
        let particle = components.get_mut_component::<Particle>(e);

        if transform.is_some() && particle.is_some() {
            let transform = transform.unwrap();
            let particle = particle.unwrap();

            transform.pos += particle.vel * delta;
            particle.acc = particle.force_accum / particle.mass;
            particle.vel += particle.acc * delta;
            // Raising to the delta power makes the damping more realistic when frame times are inconsistent, especially when damping
            //  is not terribly close to 0. However, this operation is expensive, so we wouldn't want to do it when we're applying this
            //  to a huge number of particles, for example.
            particle.vel *= DAMPING.powf(delta);
        }
    }
};

const PUSH_CUBES: System = |entites: Iter<Entity>, components: &mut ComponentManager, commands: &mut ECSCommands| {
    let cam = &entites.clone().find_map(|e| components.get_component::<Viewport2D>(e)).unwrap().cam;

    const PUSH_DIST: f32 = 8.0;
    const FORCE_FACTOR: f32 = 1.0;

    for e in entites {
        let transform = components.get_component::<Transform>(e);
        let particle = components.get_mut_component::<Particle>(e);
        let material = components.get_mut_component::<ColorMaterial>(e);

        if transform.is_some() && particle.is_some() && material.is_some() {
            let transform = transform.unwrap();
            let particle = particle.unwrap();
            let material = material.unwrap();

            if material.color == PURPLE {
                particle.force_accum.y = particle.mass * -5.0;
            } else if material.color == BLUE {
                particle.force_accum.y = particle.mass * 0.6;
            } else {
                let diff = transform.pos - cam.pos;
                if diff.len() <= PUSH_DIST {
                    particle.force_accum = diff / PUSH_DIST * FORCE_FACTOR;
                } else {
                    particle.force_accum = VEC_3_ZERO;
                }
            }

            let game_bounds = 50.0;

            if transform.pos.len() > game_bounds {
                commands.destroy_entity(e);
            }
        }
    }
};

const TURN_CUBES: System = |entites: Iter<Entity>, components: &mut ComponentManager, _: &mut ECSCommands| {
    let vulkan = entites.clone().find_map(|e| components.get_component::<VulkanComponent>(e)).unwrap();
    let time_delta = entites.clone().find_map(|e| components.get_component::<TimeDelta>(e)).unwrap();

    if let Ok(window) = vulkan.render_engine.get_window() {
        for e in entites {
            if let Some(transform) = components.get_mut_component::<Transform>(e) {
                let rot_speed = 90.0 * time_delta.since_last_frame.as_secs_f32();

                if window.is_key_down(VirtualKey::J) && !window.is_key_down(VirtualKey::L) {
                    let spin = Quat::from_axis_spin(&VEC_3_Y_AXIS, -rot_speed).unwrap();
                    transform.rot = (transform.rot * spin).normalized().unwrap();
                }
                if window.is_key_down(VirtualKey::L) && !window.is_key_down(VirtualKey::J) {
                    let spin = Quat::from_axis_spin(&VEC_3_Y_AXIS, rot_speed).unwrap();
                    transform.rot = (transform.rot * spin).normalized().unwrap();
                }
                if window.is_key_down(VirtualKey::I) && !window.is_key_down(VirtualKey::K) {
                    let spin = Quat::from_axis_spin(&VEC_3_X_AXIS, -rot_speed).unwrap();
                    transform.rot = (transform.rot * spin).normalized().unwrap();
                }
                if window.is_key_down(VirtualKey::K) && !window.is_key_down(VirtualKey::I) {
                    let spin = Quat::from_axis_spin(&VEC_3_X_AXIS, rot_speed).unwrap();
                    transform.rot = (transform.rot * spin).normalized().unwrap();
                }
                if window.is_key_down(VirtualKey::U) && !window.is_key_down(VirtualKey::O) {
                    let spin = Quat::from_axis_spin(&VEC_3_Z_AXIS, -rot_speed).unwrap();
                    transform.rot = (transform.rot * spin).normalized().unwrap();
                }
                if window.is_key_down(VirtualKey::O) && !window.is_key_down(VirtualKey::U) {
                    let spin = Quat::from_axis_spin(&VEC_3_Z_AXIS, rot_speed).unwrap();
                    transform.rot = (transform.rot * spin).normalized().unwrap();
                }
            }
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
        world: components.get_component::<Transform>(e).unwrap().to_world_mat().unwrap(),
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

#[derive(Debug, Clone)]
struct MeshWrapper {
    my_id: u16,
    id: MeshId,
}

impl Component for MeshWrapper {}
