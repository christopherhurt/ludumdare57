use anyhow::Result;
use math::{get_proj_matrix, vec2, vec3, Quat, VEC_2_ZERO, VEC_3_X_AXIS, VEC_3_Y_AXIS, VEC_3_ZERO, VEC_3_Z_AXIS};
use std::collections::hash_set::Iter;
use std::collections::HashSet;
use std::time::Duration;

use crate::component_bindings::{Mesh, VulkanComponent};
use crate::core::{Camera, Color, ColorMaterial, TimeDelta, Timer, Transform, Viewport2D, BLUE, ORANGE, PURPLE, RED, YELLOW, BLACK, WHITE, MAGENTA, GREEN};
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
        .with_component::<Timer>()
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

    let cube_4_mesh = Mesh::new(cube_mesh_id);
    let cube_4_transform = Transform::new(
        vec3(-10.0, 10.0, -10.0),
        Quat::from_axis_spin(&VEC_3_Y_AXIS, 0.0).unwrap(),
        vec3(6.0, 6.0, 6.0),
    );
    let cube_4_color_material = ColorMaterial::new(BLACK);
    let cube_4_particle = Particle::new(VEC_3_ZERO, DAMPING, 10.0);
    let cube_4_entity = ecs.create_entity();
    ecs.attach_provisional_component(&cube_4_entity, cube_4_mesh);
    ecs.attach_provisional_component(&cube_4_entity, cube_4_transform);
    ecs.attach_provisional_component(&cube_4_entity, cube_4_color_material);
    ecs.attach_provisional_component(&cube_4_entity, cube_4_particle);

    let cube_5_mesh = Mesh::new(cube_mesh_id);
    let cube_5_transform = Transform::new(
        vec3(0.0, 10.0, -10.0),
        Quat::from_axis_spin(&VEC_3_Y_AXIS, 0.0).unwrap(),
        vec3(2.0, 2.0, 2.0),
    );
    let cube_5_color_material = ColorMaterial::new(WHITE);
    let cube_5_particle = Particle::new(VEC_3_ZERO, DAMPING, 3.0);
    let cube_5_entity = ecs.create_entity();
    ecs.attach_provisional_component(&cube_5_entity, cube_5_mesh);
    ecs.attach_provisional_component(&cube_5_entity, cube_5_transform);
    ecs.attach_provisional_component(&cube_5_entity, cube_5_color_material);
    ecs.attach_provisional_component(&cube_5_entity, cube_5_particle);

    let cube_6_mesh = Mesh::new(cube_mesh_id);
    let cube_6_transform = Transform::new(
        vec3(0.0, 0.0, 0.0),
        Quat::from_axis_spin(&VEC_3_Y_AXIS, 0.0).unwrap(),
        vec3(1.0, 1.0, 1.0),
    );
    let cube_6_color_material = ColorMaterial::new(GREEN);
    let cube_6_particle = Particle::new(VEC_3_ZERO, DAMPING, 6.0);
    let cube_6_entity = ecs.create_entity();
    ecs.attach_provisional_component(&cube_6_entity, cube_6_mesh);
    ecs.attach_provisional_component(&cube_6_entity, cube_6_transform);
    ecs.attach_provisional_component(&cube_6_entity, cube_6_color_material);
    ecs.attach_provisional_component(&cube_6_entity, cube_6_particle);

    let cube_7_mesh = Mesh::new(cube_mesh_id);
    let cube_7_transform = Transform::new(
        vec3(10.0, 40.0, -10.0),
        Quat::from_axis_spin(&VEC_3_Y_AXIS, 0.0).unwrap(),
        vec3(3.0, 3.0, 3.0),
    );
    let cube_7_color_material = ColorMaterial::new(MAGENTA);
    let cube_7_particle = Particle::new(VEC_3_ZERO, DAMPING, 5.0);
    let cube_7_entity = ecs.create_entity();
    ecs.attach_provisional_component(&cube_7_entity, cube_7_mesh);
    ecs.attach_provisional_component(&cube_7_entity, cube_7_transform);
    ecs.attach_provisional_component(&cube_7_entity, cube_7_color_material);
    ecs.attach_provisional_component(&cube_7_entity, cube_7_particle);

    let cube_8_mesh = Mesh::new(cube_mesh_id);
    let cube_8_transform = Transform::new(
        vec3(10.0, -40.0, -25.0),
        Quat::from_axis_spin(&VEC_3_Y_AXIS, 0.0).unwrap(),
        vec3(8.0, 8.0, 8.0),
    );
    let cube_8_color_material = ColorMaterial::new(MAGENTA);
    let cube_8_particle = Particle::new(VEC_3_ZERO, DAMPING, 18.0);
    let cube_8_entity = ecs.create_entity();
    ecs.attach_provisional_component(&cube_8_entity, cube_8_mesh);
    ecs.attach_provisional_component(&cube_8_entity, cube_8_transform);
    ecs.attach_provisional_component(&cube_8_entity, cube_8_color_material);
    ecs.attach_provisional_component(&cube_8_entity, cube_8_particle);

    let cube_mesh_wrapper = MeshWrapper { my_id: 0, id: cube_mesh_id };
    let cube_mesh_wrapper_entity = ecs.create_entity();
    ecs.attach_provisional_component(&cube_mesh_wrapper_entity, cube_mesh_wrapper.clone());

    let firework_spawner_entity = ecs.create_entity();
    ecs.attach_provisional_component(&firework_spawner_entity, cube_mesh_wrapper);
    ecs.attach_provisional_component(&firework_spawner_entity, Timer::for_initial_duration(Duration::from_secs(3)));
    ecs.attach_provisional_component(&firework_spawner_entity, Transform::new(vec3(20.0, 0.0, 0.0), Quat::from_axis_spin(&VEC_3_X_AXIS, 0.0).unwrap(), VEC_3_ZERO));

    let vulkan = VulkanComponent::new(render_engine);
    let vulkan_entity = ecs.create_entity();
    ecs.attach_provisional_component(&vulkan_entity, vulkan);

    let time_delta = TimeDelta::default();
    let time_delta_entity = ecs.create_entity();
    ecs.attach_provisional_component(&time_delta_entity, time_delta);

    ecs.register_system(SHUTDOWN_ECS, HashSet::from([ecs.get_system_signature_1::<VulkanComponent>().unwrap()]), -999);
    ecs.register_system(TIME_SINCE_LAST_FRAME, HashSet::from([ecs.get_system_signature_1::<TimeDelta>().unwrap()]), -500);
    ecs.register_system(MOVE_CAMERA, HashSet::from([ecs.get_system_signature_1::<VulkanComponent>().unwrap(), ecs.get_system_signature_1::<Viewport2D>().unwrap(), ecs.get_system_signature_1::<TimeDelta>().unwrap()]), -400);
    ecs.register_system(CHECK_OUT_OF_BOUNDS, HashSet::from([ecs.get_system_signature_3::<Transform, Particle, ColorMaterial>().unwrap()]), -375);
    ecs.register_system(PUSH_CUBES, HashSet::from([ecs.get_system_signature_3::<Transform, Particle, ColorMaterial>().unwrap(), ecs.get_system_signature_1::<Viewport2D>().unwrap()]), -350);
    ecs.register_system(APPLY_GRAVITY, HashSet::from([ecs.get_system_signature_3::<Transform, Particle, ColorMaterial>().unwrap()]), -350);
    ecs.register_system(APPLY_DRAG, HashSet::from([ecs.get_system_signature_3::<Transform, Particle, ColorMaterial>().unwrap()]), -350);
    ecs.register_system(APPLY_CEILING_SPRING, HashSet::from([ecs.get_system_signature_3::<Transform, Particle, ColorMaterial>().unwrap()]), -350);
    ecs.register_system(APPLY_BUNGEE_SPRING, HashSet::from([ecs.get_system_signature_3::<Transform, Particle, ColorMaterial>().unwrap(), ecs.get_system_signature_1::<Viewport2D>().unwrap()]), -350);
    ecs.register_system(APPLY_BUOYANCY, HashSet::from([ecs.get_system_signature_3::<Transform, Particle, ColorMaterial>().unwrap()]), -350);
    ecs.register_system(TURN_CUBES, HashSet::from([ecs.get_system_signature_1::<VulkanComponent>().unwrap(), ecs.get_system_signature_1::<Transform>().unwrap(), ecs.get_system_signature_1::<TimeDelta>().unwrap()]), -300);
    ecs.register_system(SHOOT_FIREWORKS, HashSet::from([ecs.get_system_signature_3::<Timer, MeshWrapper, Transform>().unwrap()]), -299);
    ecs.register_system(SHOOT_PROJECTILE, HashSet::from([ecs.get_system_signature_1::<VulkanComponent>().unwrap(), ecs.get_system_signature_1::<MeshWrapper>().unwrap(), ecs.get_system_signature_1::<Viewport2D>().unwrap()]), -250);
    ecs.register_system(UPDATE_PARTICLES, HashSet::from([ecs.get_system_signature_2::<Transform, Particle>().unwrap(), ecs.get_system_signature_1::<TimeDelta>().unwrap()]), -200);
    ecs.register_system(SYNC_RENDER_STATE, HashSet::from([ecs.get_system_signature_0().unwrap()]), 2);
    ecs.register_system(UPDATE_TIMERS, HashSet::from([ecs.get_system_signature_1::<Timer>().unwrap(), ecs.get_system_signature_1::<TimeDelta>().unwrap()]), 3);
    ecs.register_system(SHUTDOWN_RENDER_ENGINE, HashSet::from([ecs.get_system_signature_1::<VulkanComponent>().unwrap()]), 999);
}

// Built-in
const SHUTDOWN_ECS: System = |entites: Iter<Entity>, components: &ComponentManager, commands: &mut ECSCommands| {
    entites.for_each(|e| {
        let vulkan = components.get_component::<VulkanComponent>(e).unwrap();

        if vulkan.render_engine.get_window().map_or(true, |w| w.is_closing()) {
            commands.shutdown();
        }
    });
};

// Built-in
const TIME_SINCE_LAST_FRAME: System = |entites: Iter<Entity>, components: &ComponentManager, _: &mut ECSCommands| {
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

const MOVE_CAMERA: System = |entites: Iter<Entity>, components: &ComponentManager, _: &mut ECSCommands| {
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

const SHOOT_PROJECTILE: System = |entites: Iter<Entity>, components: &ComponentManager, commands: &mut ECSCommands| {
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

// Built-in
const UPDATE_PARTICLES: System = |entites: Iter<Entity>, components: &ComponentManager, _: &mut ECSCommands| {
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

            particle.force_accum = VEC_3_ZERO;
        }
    }
};

const CHECK_OUT_OF_BOUNDS: System = |entites: Iter<Entity>, components: &ComponentManager, commands: &mut ECSCommands| {
    for (e, transform, _, _) in get_cubes(entites, components) {
        let game_bounds = 100.0;

        if transform.pos.len() > game_bounds {
            commands.destroy_entity(e);
        }
    }
};

const PUSH_CUBES: System = |entites: Iter<Entity>, components: &ComponentManager, _: &mut ECSCommands| {
    let cam = &entites.clone().find_map(|e| components.get_component::<Viewport2D>(e)).unwrap().cam;

    const PUSH_DIST: f32 = 8.0;
    const FORCE_FACTOR: f32 = 1.0;

    for (_, transform, particle, material) in get_cubes(entites, components) {
        if material.color != PURPLE && material.color != BLUE {
            let diff = transform.pos - cam.pos;
            if diff.len() <= PUSH_DIST {
                particle.force_accum += diff / PUSH_DIST * FORCE_FACTOR;
            }
        }
    }
};

// TODO: formalize force generator functions and move and generalize them to the physics module as I see fit
const APPLY_GRAVITY: System = |entites: Iter<Entity>, components: &ComponentManager, _: &mut ECSCommands| {
    for (_, _, particle, material) in get_cubes(entites, components) {
        if material.color == PURPLE {
            particle.force_accum.y += particle.mass * -5.0;
        } else if material.color == BLUE {
            particle.force_accum.y += particle.mass * 0.6;
        } else if material.color == BLACK || material.color == WHITE || material.color == MAGENTA || material.color == GREEN {
            particle.force_accum.y += particle.mass * -25.0;
        }
    }
};

const APPLY_DRAG: System = |entites: Iter<Entity>, components: &ComponentManager, _: &mut ECSCommands| {
    const K1: f32 = 0.05;
    const K2: f32 = 0.05;

    for (_, _, particle, material) in get_cubes(entites, components) {
        if let Ok(speed) = particle.vel.normalized() {
            if material.color == PURPLE {
                particle.force_accum += -particle.vel * (K1 * speed + K2 * speed * speed);
            } else if material.color == GREEN {
                particle.force_accum += -particle.vel * (20.0 * speed + 20.0 * speed * speed);
            } else if material.color == BLACK || material.color == WHITE {
                particle.force_accum += -particle.vel * (1.0 * speed + 1.0 * speed * speed);
            } else if material.color == MAGENTA {
                particle.force_accum += -particle.vel * (10.0 * speed + 10.0 * speed * speed);
            }
        }
    }
};

const APPLY_CEILING_SPRING: System = |entites: Iter<Entity>, components: &ComponentManager, _: &mut ECSCommands| {
    const CEIL_HEIGHT: f32 = 10.0;
    const REST_LENGTH: f32 = 3.0;
    const K: f32 = 10.0;

    for (_, transform, particle, material) in get_cubes(entites, components) {
        if material.color == WHITE || material.color == BLACK {
            let pos = transform.pos;
            let d = pos - vec3(pos.x, CEIL_HEIGHT, pos.z);
            let delta_length = d.len() - REST_LENGTH;

            if let Ok(d_norm) = d.normalized() {
                particle.force_accum += -K * delta_length * d_norm;
            }
        }
    }
};

const APPLY_BUNGEE_SPRING: System = |entites: Iter<Entity>, components: &ComponentManager, _: &mut ECSCommands| {
    let cam_pos = &entites.clone().find_map(|e| components.get_component::<Viewport2D>(e)).unwrap().cam.pos;

    const K: f32 = 50.0;
    const REST_LENGTH: f32 = 5.0;

    for (_, transform, particle, material) in get_cubes(entites, components) {
        if material.color == GREEN {
            let d = transform.pos - *cam_pos;
            let delta_length = d.len() - REST_LENGTH;

            if delta_length > 0.0 {
                if let Ok(d_norm) = d.normalized() {
                    particle.force_accum += -K * delta_length * d_norm;
                }
            }
        }
    }
};

const APPLY_BUOYANCY: System = |entites: Iter<Entity>, components: &ComponentManager, _: &mut ECSCommands| {
    const DENSITY: f32 = 10.0;
    const WATER_HEIGHT: f32 = 0.0;

    for (_, transform, particle, material) in get_cubes(entites, components) {
        if material.color == MAGENTA {
            let submersion_depth = -transform.scl.y / 2.0;
            let volume = transform.scl.x * transform.scl.y * transform.scl.z;

            let d = ((transform.pos.y - WATER_HEIGHT - submersion_depth) / (2.0 * submersion_depth)).max(0.0).min(1.0);

            particle.force_accum += vec3(0.0, d * DENSITY * volume, 0.0);
        }
    }
};

const TURN_CUBES: System = |entites: Iter<Entity>, components: &ComponentManager, _: &mut ECSCommands| {
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

const SHOOT_FIREWORKS: System = |entites: Iter<Entity>, components: &ComponentManager, commands: &mut ECSCommands| {
    let pos = &entites.clone().find_map(|e| components.get_component::<Transform>(e)).unwrap().pos;
    let mesh_id = &entites.clone().find_map(|e| components.get_component::<MeshWrapper>(e).filter(|m| m.my_id == 0)).unwrap().id;

    for e in entites {
        if let Some(timer) = components.get_mut_component::<Timer>(e) {
            if timer.remaining_duration.is_none() {
                let mesh = Mesh::new(mesh_id.clone());
                let color_material = ColorMaterial::new(PURPLE);
                let transform = Transform::new(pos.clone(), Quat::from_axis_spin(&VEC_3_Y_AXIS, 45.0).unwrap(), vec3(1.0, 1.0, 1.0));
                let particle = Particle::new(VEC_3_Y_AXIS * 10.0, 0.9999999, 0.1);

                let proj_entity = commands.create_entity();
                commands.attach_provisional_component(&proj_entity, mesh);
                commands.attach_provisional_component(&proj_entity, color_material);
                commands.attach_provisional_component(&proj_entity, transform);
                commands.attach_provisional_component(&proj_entity, particle);

                timer.reset();
            }
        }
    }
};

// Built-in
const SYNC_RENDER_STATE: System = |entites: Iter<Entity>, components: &ComponentManager, _: &mut ECSCommands| {
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

// Built-in
const UPDATE_TIMERS: System = |entites: Iter<Entity>, components: &ComponentManager, _: &mut ECSCommands| {
    let time_delta = entites.clone().find_map(|e| components.get_component::<TimeDelta>(e)).unwrap();

    for e in entites {
        if let Some(timer) = components.get_mut_component::<Timer>(e) {
            timer.update(&time_delta.since_last_frame);
        }
    }
};

// Built-in
const SHUTDOWN_RENDER_ENGINE: System = |entites: Iter<Entity>, components: &ComponentManager, commands: &mut ECSCommands| {
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

fn get_cubes<'a>(entities: Iter<'a, Entity>, components: &'a ComponentManager) -> impl Iterator<Item = (&'a Entity, &'a Transform, &'a mut Particle, &'a ColorMaterial)> {
    entities.map(|e| {
        let transform = components.get_component::<Transform>(e);
        let particle = components.get_mut_component::<Particle>(e);
        let material = components.get_component::<ColorMaterial>(e);

        if transform.is_some() && particle.is_some() && material.is_some() {
            Some((e, transform.unwrap(), particle.unwrap(), material.unwrap()))
        } else {
            None
        }
    })
    .filter(|e| e.is_some())
    .map(|e| e.unwrap())
}

#[derive(Debug, Clone)]
struct MeshWrapper {
    my_id: u16,
    id: MeshId,
}

impl Component for MeshWrapper {}
