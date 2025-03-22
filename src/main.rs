use anyhow::Result;
use math::{get_proj_matrix, vec2, vec3, Quat, VEC_2_ZERO, VEC_3_X_AXIS, VEC_3_Y_AXIS, VEC_3_ZERO, VEC_3_Z_AXIS};
use rand::Rng;
use std::cmp::Ordering;
use std::collections::hash_set::Iter;
use std::collections::HashSet;
use std::time::Duration;

use crate::component_bindings::{Mesh, VulkanComponent};
use crate::core::{Camera, Color, ColorMaterial, TimeDelta, Timer, Transform, Viewport2D, BLUE, ORANGE, PURPLE, RED, YELLOW, BLACK, WHITE, MAGENTA, GREEN, GRAY, BROWN, CYAN};
use crate::ecs::component::{Component, ComponentManager};
use crate::ecs::entity::Entity;
use crate::ecs::system::System;
use crate::ecs::{ComponentActions, ECSBuilder, ECSCommands, ECS};
use crate::physics::{Particle, ParticleCable, ParticleRod, ParticleCollision, ParticleCollisionDetector};
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
        .with_component::<ParticleCable>()
        .with_component::<ParticleRod>()
        .with_component::<ParticleCollision>()
        .with_component::<ParticleCollisionDetector>()
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

    let cube_9_mesh = Mesh::new(cube_mesh_id);
    let cube_9_transform = Transform::new(
        vec3(-10.0, 10.0, 30.0),
        Quat::from_axis_spin(&VEC_3_Y_AXIS, 0.0).unwrap(),
        vec3(6.0, 6.0, 6.0),
    );
    let cube_9_color_material = ColorMaterial::new(GRAY);
    let cube_9_particle = Particle::new(VEC_3_ZERO, 1.0, 10.0);
    let cube_9_entity = ecs.create_entity();
    ecs.attach_provisional_component(&cube_9_entity, cube_9_mesh);
    ecs.attach_provisional_component(&cube_9_entity, cube_9_transform);
    ecs.attach_provisional_component(&cube_9_entity, cube_9_color_material);
    ecs.attach_provisional_component(&cube_9_entity, cube_9_particle);

    let cube_10_mesh = Mesh::new(cube_mesh_id);
    let cube_10_transform = Transform::new(
        vec3(-5.0, 0.0, 2.0),
        Quat::from_axis_spin(&VEC_3_Y_AXIS, 0.0).unwrap(),
        vec3(1.0, 1.0, 1.0),
    );
    let cube_10_color_material = ColorMaterial::new(BROWN);
    let cube_10_particle = Particle::new(VEC_3_ZERO, 1.0, 1.0);
    let cube_10_entity = ecs.create_entity();
    ecs.attach_provisional_component(&cube_10_entity, cube_10_mesh);
    ecs.attach_provisional_component(&cube_10_entity, cube_10_transform);
    ecs.attach_provisional_component(&cube_10_entity, cube_10_color_material);
    ecs.attach_provisional_component(&cube_10_entity, cube_10_particle);

    let cube_11_mesh = Mesh::new(cube_mesh_id);
    let cube_11_transform = Transform::new(
        vec3(-5.0, 0.0, -2.0),
        Quat::from_axis_spin(&VEC_3_Y_AXIS, 0.0).unwrap(),
        vec3(1.0, 1.0, 1.0),
    );
    let cube_11_color_material = ColorMaterial::new(BROWN);
    let cube_11_particle = Particle::new(VEC_3_ZERO, 1.0, 1.0);
    let cube_11_entity = ecs.create_entity();
    let cube_11_rod = ParticleRod::new_provisional(cube_10_entity.clone(), cube_11_entity.clone(), 4.0);
    ecs.attach_provisional_component(&cube_11_entity, cube_11_mesh);
    ecs.attach_provisional_component(&cube_11_entity, cube_11_transform);
    ecs.attach_provisional_component(&cube_11_entity, cube_11_color_material);
    ecs.attach_provisional_component(&cube_11_entity, cube_11_particle);
    ecs.attach_provisional_component(&cube_11_entity, cube_11_rod);

    let cube_12_mesh = Mesh::new(cube_mesh_id);
    let cube_12_transform = Transform::new(
        vec3(5.0, 0.0, 3.0),
        Quat::from_axis_spin(&VEC_3_Y_AXIS, 0.0).unwrap(),
        vec3(2.0, 2.0, 2.0),
    );
    let cube_12_color_material = ColorMaterial::new(CYAN);
    let cube_12_particle = Particle::new(VEC_3_ZERO, 1.0, 4.0);
    let cube_12_entity = ecs.create_entity();
    ecs.attach_provisional_component(&cube_12_entity, cube_12_mesh);
    ecs.attach_provisional_component(&cube_12_entity, cube_12_transform);
    ecs.attach_provisional_component(&cube_12_entity, cube_12_color_material);
    ecs.attach_provisional_component(&cube_12_entity, cube_12_particle);

    let cube_13_mesh = Mesh::new(cube_mesh_id);
    let cube_13_transform = Transform::new(
        vec3(5.0, 0.0, -3.0),
        Quat::from_axis_spin(&VEC_3_Y_AXIS, 0.0).unwrap(),
        vec3(1.0, 1.0, 1.0),
    );
    let cube_13_color_material = ColorMaterial::new(CYAN);
    let cube_13_particle = Particle::new(VEC_3_ZERO, 1.0, 1.0);
    let cube_13_entity = ecs.create_entity();
    let cube_13_cable = ParticleCable::new_provisional(cube_12_entity.clone(), cube_13_entity.clone(), 8.0, 0.5);
    ecs.attach_provisional_component(&cube_13_entity, cube_13_mesh);
    ecs.attach_provisional_component(&cube_13_entity, cube_13_transform);
    ecs.attach_provisional_component(&cube_13_entity, cube_13_color_material);
    ecs.attach_provisional_component(&cube_13_entity, cube_13_particle);
    ecs.attach_provisional_component(&cube_12_entity, cube_13_cable);

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

    let particle_collision_detector = ParticleCollisionDetector::new(0.1);
    let particle_collision_detector_entity = ecs.create_entity();
    ecs.attach_provisional_component(&particle_collision_detector_entity, particle_collision_detector);

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
    ecs.register_system(APPLY_DAMPED_HARMONIC_MOTION, HashSet::from([ecs.get_system_signature_3::<Transform, Particle, ColorMaterial>().unwrap(), ecs.get_system_signature_1::<TimeDelta>().unwrap()]), -350);
    ecs.register_system(TURN_CUBES, HashSet::from([ecs.get_system_signature_1::<VulkanComponent>().unwrap(), ecs.get_system_signature_1::<Transform>().unwrap(), ecs.get_system_signature_1::<TimeDelta>().unwrap()]), -300);
    ecs.register_system(SHOOT_FIREWORKS, HashSet::from([ecs.get_system_signature_3::<Timer, MeshWrapper, Transform>().unwrap()]), -299);
    ecs.register_system(SHOOT_PROJECTILE, HashSet::from([ecs.get_system_signature_1::<VulkanComponent>().unwrap(), ecs.get_system_signature_1::<MeshWrapper>().unwrap(), ecs.get_system_signature_1::<Viewport2D>().unwrap()]), -250);
    ecs.register_system(UPDATE_PARTICLES, HashSet::from([ecs.get_system_signature_2::<Transform, Particle>().unwrap(), ecs.get_system_signature_1::<TimeDelta>().unwrap()]), -200);
    ecs.register_system(DETECT_PARTICLE_COLLISIONS, HashSet::from([ecs.get_system_signature_2::<Transform, Particle>().unwrap(), ecs.get_system_signature_1::<ParticleCollisionDetector>().unwrap()]), -100);
    ecs.register_system(DETECT_PARTICLE_CABLE_COLLISIONS, HashSet::from([ecs.get_system_signature_1::<ParticleCable>().unwrap()]), -100);
    ecs.register_system(DETECT_PARTICLE_ROD_COLLISIONS, HashSet::from([ecs.get_system_signature_1::<ParticleRod>().unwrap()]), -100);
    ecs.register_system(RESOLVE_PARTICLE_COLLISIONS, HashSet::from([ecs.get_system_signature_1::<TimeDelta>().unwrap(), ecs.get_system_signature_1::<ParticleCollision>().unwrap()]), -99);
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
            particle.vel *= particle.damping.powf(delta);

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

    const PUSH_DIST: f32 = 2.0;
    const FORCE_FACTOR: f32 = 75.0;

    for (_, transform, particle, material) in get_cubes(entites, components) {
        if material.color != PURPLE && material.color != BLUE && material.color != GRAY {
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

// TODO: make built in
const APPLY_DRAG: System = |entites: Iter<Entity>, components: &ComponentManager, _: &mut ECSCommands| {
    const K1: f32 = 0.05;
    const K2: f32 = 0.05;

    for (_, _, particle, material) in get_cubes(entites, components) {
        let speed = particle.vel.len();

        if let Ok(vel_dir) = particle.vel.normalized() {
            if material.color == PURPLE {
                particle.force_accum += -vel_dir * (K1 * speed + K2 * speed * speed);
            } else if material.color == GREEN {
                particle.force_accum += -vel_dir * (2.0 * speed + 2.0 * speed * speed);
            } else if material.color == BLACK || material.color == WHITE {
                particle.force_accum += -vel_dir * (1.0 * speed + 1.0 * speed * speed);
            } else if material.color == MAGENTA {
                particle.force_accum += -vel_dir * (1.0 * speed + 1.0 * speed * speed);
            } else if material.color == BROWN || material.color == CYAN {
                particle.force_accum += -vel_dir * (0.1 * speed + 0.1 * speed * speed);
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

// TODO: this is really just for demonstration purposes and prob shouldn't make it into the actual engine, or only some minimally used version of it
//  This also seems to be EXTREMELY unstable for high frame rates because the calculated accereleration is huge. We might be able to mitigate it by
//  applying a large multiplier to delta like below and finding the right K value, but the whole thing seemed pretty scuffed. They also don't really
//  work at all when combined with any other forces, like gravity, because those are not included in the target position calculation. Best thing is
//  probably just to avoid these types of forces calculated by target destination. Note though that the point of this type of force calculation is
//  to avoid issues related to very high K values, i.e. "stiff springs". A trivial spring force implementation would also likely have issues caused
//  by very large acceleration values when K values are high. From book - "When the action of the spring is faster than the time between simulated
//  frames, then the spring can get unruly and out of control."
const APPLY_DAMPED_HARMONIC_MOTION: System = |entites: Iter<Entity>, components: &ComponentManager, _: &mut ECSCommands| {
    let time_delta = entites.clone().find_map(|e| components.get_component::<TimeDelta>(e)).unwrap();
    let delta = time_delta.since_last_frame.as_secs_f32();

    let delta = delta * 1000.0; // YIKES

    const DAMPING: f32 = 0.995;
    const K: f32 = 100.0;

    if delta > 0.0 {
        for (_, transform, particle, material) in get_cubes(entites, components) {
            if material.color == GRAY {
                let anchor_pos = vec3(-10.0, 12.0, 30.0);
                let rel_pos = transform.pos - anchor_pos;

                let gamma = 0.5 * (4.0 * K - DAMPING * DAMPING).sqrt();

                if gamma != 0.0 {
                    let c = rel_pos * (DAMPING / (2.0 * gamma)) + particle.vel / gamma;

                    let target = (rel_pos * (gamma * delta).cos() + c * (gamma * delta).sin()) * std::f32::consts::E.powf(-0.5 * delta * DAMPING);

                    let acceleration = (target - rel_pos) / (delta * delta) - particle.vel * delta;
                    particle.force_accum += acceleration * particle.mass;
                }
            }
        }
    }
};

// Built-in
const DETECT_PARTICLE_COLLISIONS: System = |entites: Iter<Entity>, components: &ComponentManager, commands: &mut ECSCommands| {
    // TODO: this system will be overhauled later, this is just a simple collision detector for testing

    let collision_detector = entites.clone().find_map(|e| components.get_component::<ParticleCollisionDetector>(e)).unwrap();

    const FLOOR_HEIGHT: f32 = -15.0;

    let mut rng = rand::rng();

    for e in entites {
        let transform = components.get_component::<Transform>(e);
        let particle = components.get_component::<Particle>(e);

        if transform.is_some() && particle.is_some() { // Only check that a Particle component exists, otherwise it's not needed
            let transform = transform.unwrap();

            if transform.pos.y <= FLOOR_HEIGHT {
                let restitution = if rng.random_range(0.0..1.0) < 0.5 {
                    rng.random_range(0.0..1.0)
                } else {
                    collision_detector.default_restitution
                };

                let collision = ParticleCollision::new(e.clone(), None, restitution, VEC_3_Y_AXIS, FLOOR_HEIGHT - transform.pos.y);

                let collision_entity = commands.create_entity();
                commands.attach_provisional_component(&collision_entity, collision);
            }
        }
    }
};

// Built-in
const DETECT_PARTICLE_CABLE_COLLISIONS: System = |entites: Iter<Entity>, components: &ComponentManager, commands: &mut ECSCommands| {
    let cables = entites
        .map(|e| components.get_component::<ParticleCable>(e))
        .filter(|c| c.is_some())
        .map(|c| c.unwrap());

    for c in cables {
        // TODO: something other than panicking here...
        let transform_a = components.get_component::<Transform>(&c.particle_a)
            .unwrap_or_else(|| panic!("Internal error: no Transform component for entity {:?}", &c.particle_a));
        let transform_b = components.get_component::<Transform>(&c.particle_b)
            .unwrap_or_else(|| panic!("Internal error: no Transform component for entity {:?}", &c.particle_b));

        let delta_pos = transform_b.pos - transform_a.pos;
        let length = delta_pos.len();

        if length >= c.max_length {
            if let Ok(normal) = delta_pos.normalized() {
                let collision = ParticleCollision::new(
                    c.particle_a,
                    Some(c.particle_b),
                    c.restitution,
                    normal,
                    length - c.max_length,
                );

                let collision_entity = commands.create_entity();
                commands.attach_provisional_component(&collision_entity, collision);
            }
        }
    }
};

// Built-in
const DETECT_PARTICLE_ROD_COLLISIONS: System = |entites: Iter<Entity>, components: &ComponentManager, commands: &mut ECSCommands| {
    let rods = entites
        .map(|e| components.get_component::<ParticleRod>(e))
        .filter(|r| r.is_some())
        .map(|r| r.unwrap());

    for r in rods {
        let transform_a = components.get_component::<Transform>(&r.particle_a)
            .unwrap_or_else(|| panic!("Internal error: no Transform component for entity {:?}", &r.particle_a));
        let transform_b = components.get_component::<Transform>(&r.particle_b)
            .unwrap_or_else(|| panic!("Internal error: no Transform component for entity {:?}", &r.particle_b));

        let delta_pos = transform_b.pos - transform_a.pos;
        let curr_length = delta_pos.len();

        if let Ok(mut normal) = delta_pos.normalized() {
            let mut penetration = curr_length - r.length;

            if penetration < 0.0 {
                normal *= -1.0;
                penetration *= -1.0;
            }

            let collision = ParticleCollision::new(
                r.particle_a,
                Some(r.particle_b),
                0.0,
                normal,
                penetration,
            );

            let collision_entity = commands.create_entity();
            commands.attach_provisional_component(&collision_entity, collision);
        }
    }
};

// Built-in
const RESOLVE_PARTICLE_COLLISIONS: System = |entites: Iter<Entity>, components: &ComponentManager, commands: &mut ECSCommands| {
    let time_delta = entites.clone().find_map(|e| components.get_component::<TimeDelta>(e)).unwrap();
    let delta_sec = time_delta.since_last_frame.as_secs_f32();

    let mut collisions = entites
        .map(|e| (e, components.get_component::<ParticleCollision>(e)))
        .filter(|(_, c)| c.is_some())
        .map(|(e, c)| (e, c.unwrap()))
        .collect::<Vec<_>>();

    collisions.sort_unstable_by(|(_, c0), (_, c1)|
        calculate_separating_velocity(c0, components)
            .partial_cmp(&calculate_separating_velocity(c1, components))
            .unwrap_or(Ordering::Less)
    );

    // NOTE: Since we're not recalculating collisions here, we can't use multiple iterations. Otherwise, resolve_iterpenetration would move
    //  the particles with every iteration for the same collision, even if they're no longer penetrating after the first iteration. So, just
    //  sort the collisions by separation velocity, then resolve them each once.
    for (_, c) in collisions.clone() {
        resolve_velocity(&c, components, delta_sec);
        resolve_interpenetration(&c, components);
    }

    // TODO: can we reuse ParticleCollision entities frame to frame, or otherwise make this more efficient?
    for (e, _) in collisions {
        commands.destroy_entity(e);
    }
};

fn calculate_separating_velocity(collision: &ParticleCollision, components: &ComponentManager) -> f32 {
    let particle_a = components.get_component::<Particle>(&collision.particle_a)
        .unwrap_or_else(|| panic!("Internal error: no Particle component for entity {:?}", &collision.particle_a));

    let mut rel_vel = particle_a.vel;

    if let Some(entity_b) = collision.particle_b {
        let particle_b = components.get_component::<Particle>(&entity_b)
            .unwrap_or_else(|| panic!("Internal error: no Particle component for entity {:?}", &collision.particle_b));

        rel_vel -= particle_b.vel;
    }

    rel_vel.dot(&collision.normal)
}

fn resolve_velocity(collision: &ParticleCollision, components: &ComponentManager, delta_sec: f32) {
    let sep_vel = calculate_separating_velocity(collision, components);

    if sep_vel < f32::EPSILON {
        let particle_a = components.get_mut_component::<Particle>(&collision.particle_a)
            .unwrap_or_else(|| panic!("Internal error: no Particle component for entity {:?}", &collision.particle_a));
        let particle_b = collision.particle_b.map(|b| components.get_mut_component::<Particle>(&b)
            .unwrap_or_else(|| panic!("Internal error: no Particle component for entity {:?}", &b)));

        let mut new_sep_vel = -sep_vel * collision.restitution;

        let acc_a = particle_a.acc;
        let acc_b = particle_b.as_ref().map(|b| b.acc).unwrap_or(VEC_3_ZERO);
        let sep_vel_caused_by_acc = (acc_a - acc_b).dot(&collision.normal) * delta_sec;

        // Adjust for resting collisions
        if sep_vel_caused_by_acc < 0.0 {
            new_sep_vel = (new_sep_vel + sep_vel_caused_by_acc * collision.restitution).max(0.0);
        }

        let delta_sep_vel = new_sep_vel - sep_vel;

        let mass_factor_a = particle_b.as_ref().map(|b| b.mass / (particle_a.mass + b.mass)).unwrap_or(1.0);

        particle_a.vel += mass_factor_a * delta_sep_vel * collision.normal;

        if let Some(b) = particle_b {
            let mass_factor_b = particle_a.mass / (particle_a.mass + b.mass);

            b.vel += mass_factor_b * delta_sep_vel * -collision.normal;
        }
    }
}

fn resolve_interpenetration(collision: &ParticleCollision, components: &ComponentManager) {
    if collision.penetration > f32::EPSILON {
        let particle_a = components.get_component::<Particle>(&collision.particle_a)
            .unwrap_or_else(|| panic!("Internal error: no Particle component for entity {:?}", &collision.particle_a));
        let particle_b = collision.particle_b.map(|b| components.get_component::<Particle>(&b)
            .unwrap_or_else(|| panic!("Internal error: no Particle component for entity {:?}", &b)));

        let transform_a = components.get_mut_component::<Transform>(&collision.particle_a)
            .unwrap_or_else(|| panic!("Internal error: no Transform component for entity {:?}", &collision.particle_a));
        let transform_b = collision.particle_b.map(|b| components.get_mut_component::<Transform>(&b)
            .unwrap_or_else(|| panic!("Internal error: no Transform component for entity {:?}", &b)));

        let mass_factor_a = particle_b.map(|b| b.mass / (particle_a.mass + b.mass)).unwrap_or(1.0);

        transform_a.pos += mass_factor_a * collision.penetration * collision.normal;

        if let Some(particle_b) = particle_b {
            let transform_b = transform_b.unwrap();

            let mass_factor_b = particle_a.mass / (particle_a.mass + particle_b.mass);

            transform_b.pos += mass_factor_b * collision.penetration * -collision.normal;
        }
    }
}

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
impl ComponentActions for MeshWrapper {}
