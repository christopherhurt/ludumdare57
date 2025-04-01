use anyhow::Result;
use ecs::ComponentActions;
use math::{get_proj_matrix, vec2, vec3, Vec3, QUAT_IDENTITY, VEC_2_ZERO, VEC_3_Y_AXIS, VEC_3_ZERO, VEC_3_Z_AXIS};
use physics::PotentialRigidBodyCollision;
use rand::Rng;
use core::{GRAY, IDENTITY_SCALE_VEC, RED};
use std::cmp::Ordering;
use std::collections::hash_set::Iter;
use std::collections::HashSet;

use crate::core::{Camera, Color, ColorMaterial, TimeDelta, Timer, Transform, Viewport2D, BLUE, PURPLE, BLACK, WHITE, MAGENTA, GREEN, BROWN, CYAN};
use crate::core::mesh::{Mesh, MeshBinding, Vertex, load_obj_mesh};
use crate::ecs::component::{Component, ComponentManager};
use crate::ecs::entity::Entity;
use crate::ecs::system::System;
use crate::ecs::{ECSBuilder, ECSCommands, ECS};
use crate::physics::{apply_ang_vel, generate_physics_mesh, generate_ray, get_deepest_rigid_body_collision, get_edge_collision, get_point_collision, get_ray_intersection, local_to_world_force, local_to_world_point, BoundingSphere, Particle, ParticleCable, ParticleRod, ParticleCollision, ParticleCollisionDetector, PhysicsMeshProperties, QuadTree, RigidBody, RigidBodyCollision};
use crate::render_engine::vulkan::VulkanRenderEngine;
use crate::render_engine::{Device, EntityRenderState, RenderEngine, RenderState, Window, RenderEngineInitProps, VirtualButton, VirtualKey, WindowInitProps};

pub mod core;
pub mod ecs;
pub mod math;
pub mod physics;
pub mod render_engine;

const DAMPING: f32 = 0.999;
const NEAR_PLANE: f32 = 0.01;
const FAR_PLANE: f32 = 1000.0;

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
        .with_component::<MeshBinding>()
        .with_component::<ColorMaterial>()
        .with_component::<VulkanRenderEngine>()
        .with_component::<TimeDelta>()
        .with_component::<Particle>()
        .with_component::<ParticleCable>()
        .with_component::<ParticleRod>()
        .with_component::<ParticleCollision>()
        .with_component::<ParticleCollisionDetector>()
        .with_component::<PhysicsMeshProperties>()
        .with_component::<PotentialRigidBodyCollision>()
        .with_component::<RigidBodyCollision>()
        .with_component::<QuadTree<BoundingSphere>>()
        .with_component::<RigidBody>()
        .with_component::<Timer>()
        .with_component::<CubeMeshOwner>()
        .with_component::<MousePickable>()
        .build()
}

fn init_render_engine() -> Result<VulkanRenderEngine> {
    let window_props = WindowInitProps {
        width: 1600,
        height: 1200,
        title: String::from("My Cool Game"),
        is_resizable: true,
    };

    let render_engine_props = RenderEngineInitProps {
        debug_enabled: true,
        clear_color: Color::rgb(0.0, 0.3, 0.0),
        window_props,
    };

    VulkanRenderEngine::new(render_engine_props)
}

fn create_scene(ecs: &mut ECS) {
    let mut render_engine = init_render_engine().unwrap_or_else(|e| panic!("{}", e));

    let cam = Camera::new(VEC_3_ZERO, VEC_3_Z_AXIS, VEC_3_Y_AXIS, 70.0_f32.to_radians());
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
    let cube_mesh: Mesh = Mesh::new(cube_vertices, cube_indexes).unwrap();
    let (cube_mesh, cube_physics_props) = generate_physics_mesh(cube_mesh, 10.0).unwrap();
    let cube_mesh_id = render_engine.get_device_mut()
        .and_then(|d| d.create_mesh(cube_mesh.vertices.clone(), cube_mesh.vertex_indices.clone()))
        .unwrap_or_else(|e| panic!("{}", e));
    let cube_mesh_entity = ecs.create_entity();
    let cube_mesh_binding = MeshBinding::new_provisional(Some(cube_mesh_id), Some(cube_mesh_entity));
    ecs.attach_provisional_component(&cube_mesh_entity, cube_mesh);
    ecs.attach_provisional_component(&cube_mesh_entity, cube_mesh_binding);
    ecs.attach_provisional_component(&cube_mesh_entity, cube_physics_props.clone());
    ecs.attach_provisional_component(&cube_mesh_entity, CubeMeshOwner {});

    let bunny_mesh = load_obj_mesh("res/bunny.obj", true).unwrap();
    let (bunny_mesh, bunny_physics_props) = generate_physics_mesh(bunny_mesh, 100.0).unwrap();
    let bunny_mesh_id = render_engine.get_device_mut()
        .and_then(|d| d.create_mesh(bunny_mesh.vertices.clone(), bunny_mesh.vertex_indices.clone()))
        .unwrap_or_else(|e| panic!("{}", e));
    let bunny_mesh_entity = ecs.create_entity();
    let bunny_mesh_binding = MeshBinding::new_provisional(Some(bunny_mesh_id), Some(bunny_mesh_entity));
    ecs.attach_provisional_component(&bunny_mesh_entity, bunny_mesh);
    ecs.attach_provisional_component(&bunny_mesh_entity, bunny_mesh_binding.clone());

    let test_cube_transform = Transform::new(vec3(0.0, 0.0, 10.0), QUAT_IDENTITY, IDENTITY_SCALE_VEC);
    let test_cube_material = ColorMaterial::new(RED);
    let test_cube_rigid_body = RigidBody::new(VEC_3_ZERO, VEC_3_ZERO, 0.6, 0.6, 0.0, cube_physics_props.clone());
    let test_cube_entity = ecs.create_entity();
    let test_cube_mesh_binding = MeshBinding::new_provisional(Some(cube_mesh_id), Some(cube_mesh_entity));
    ecs.attach_provisional_component(&test_cube_entity, test_cube_transform);
    ecs.attach_provisional_component(&test_cube_entity, test_cube_rigid_body);
    ecs.attach_provisional_component(&test_cube_entity, test_cube_material);
    ecs.attach_provisional_component(&test_cube_entity, test_cube_mesh_binding.clone());
    ecs.attach_provisional_component(&test_cube_entity, MousePickable {});

    let test_bunny_transform = Transform::new(vec3(20.0, -5.0,0.0), QUAT_IDENTITY, IDENTITY_SCALE_VEC * 5.0);
    let test_bunny_material = ColorMaterial::new(WHITE);
    let test_bunny_rigid_body = RigidBody::new(VEC_3_ZERO, VEC_3_ZERO, 0.9, 0.9, 0.0, bunny_physics_props.clone());
    let test_bunny_entity = ecs.create_entity();
    ecs.attach_provisional_component(&test_bunny_entity, test_bunny_transform);
    ecs.attach_provisional_component(&test_bunny_entity, test_bunny_rigid_body);
    ecs.attach_provisional_component(&test_bunny_entity, test_bunny_material);
    ecs.attach_provisional_component(&test_bunny_entity, bunny_mesh_binding.clone());
    ecs.attach_provisional_component(&test_bunny_entity, MousePickable {});

    let tether_cube_transform = Transform::new(vec3(-3.0, 10.0, 0.0), QUAT_IDENTITY, IDENTITY_SCALE_VEC);
    let tether_cube_material = ColorMaterial::new(GRAY);
    let tether_cube_rigid_body = RigidBody::new(VEC_3_ZERO, VEC_3_ZERO, 0.9, 0.9, 15.0, cube_physics_props.clone());
    let tether_cube_entity = ecs.create_entity();
    ecs.attach_provisional_component(&tether_cube_entity, tether_cube_transform);
    ecs.attach_provisional_component(&tether_cube_entity, tether_cube_rigid_body);
    ecs.attach_provisional_component(&tether_cube_entity, tether_cube_material);
    ecs.attach_provisional_component(&tether_cube_entity, test_cube_mesh_binding.clone());
    ecs.attach_provisional_component(&tether_cube_entity, MousePickable {});

    let vulkan_entity = ecs.create_entity();
    ecs.attach_provisional_component(&vulkan_entity, render_engine);

    let time_delta = TimeDelta::default();
    let time_delta_entity = ecs.create_entity();
    ecs.attach_provisional_component(&time_delta_entity, time_delta);

    let particle_collision_detector = ParticleCollisionDetector::new(0.1);
    let particle_collision_detector_entity = ecs.create_entity();
    ecs.attach_provisional_component(&particle_collision_detector_entity, particle_collision_detector);

    let quad_tree: QuadTree<BoundingSphere> = QuadTree::new(VEC_3_ZERO, 125.0, 512, 10, 96, 16).unwrap();
    let quad_tree_entity = ecs.create_entity();
    ecs.attach_provisional_component(&quad_tree_entity, quad_tree);

    ecs.register_system(SHUTDOWN_ECS, HashSet::from([ecs.get_system_signature_1::<VulkanRenderEngine>().unwrap()]), -999);
    ecs.register_system(TIME_SINCE_LAST_FRAME, HashSet::from([ecs.get_system_signature_1::<TimeDelta>().unwrap()]), -500);
    ecs.register_system(RESET_TRANSFORM_FLAGS, HashSet::from([ecs.get_system_signature_1::<Transform>().unwrap()]), -450);
    ecs.register_system(MOVE_CAMERA, HashSet::from([ecs.get_system_signature_1::<VulkanRenderEngine>().unwrap(), ecs.get_system_signature_1::<Viewport2D>().unwrap(), ecs.get_system_signature_1::<TimeDelta>().unwrap()]), -400);
    ecs.register_system(PICK_MESHES, HashSet::from([ecs.get_system_signature_1::<VulkanRenderEngine>().unwrap(), ecs.get_system_signature_1::<Viewport2D>().unwrap(), ecs.get_system_signature_4::<Transform, MeshBinding, RigidBody, MousePickable>().unwrap()]), -400);
    ecs.register_system(CHECK_OUT_OF_BOUNDS, HashSet::from([ecs.get_system_signature_1::<Transform>().unwrap()]), -375);
    ecs.register_system(APPLY_DRAG, HashSet::from([ecs.get_system_signature_3::<Transform, Particle, ColorMaterial>().unwrap()]), -350);
    ecs.register_system(APPLY_CEILING_SPRING, HashSet::from([ecs.get_system_signature_3::<Transform, Particle, ColorMaterial>().unwrap()]), -350);
    ecs.register_system(APPLY_BUNGEE_SPRING, HashSet::from([ecs.get_system_signature_3::<Transform, Particle, ColorMaterial>().unwrap(), ecs.get_system_signature_1::<Viewport2D>().unwrap()]), -350);
    ecs.register_system(APPLY_BUOYANCY, HashSet::from([ecs.get_system_signature_3::<Transform, Particle, ColorMaterial>().unwrap()]), -350);
    ecs.register_system(APPLY_RIGID_BODY_FORCE, HashSet::from([ecs.get_system_signature_1::<VulkanRenderEngine>().unwrap(), ecs.get_system_signature_3::<Transform, RigidBody, ColorMaterial>().unwrap()]), -350);
    ecs.register_system(APPLY_TETHER_BALL, HashSet::from([ecs.get_system_signature_3::<Transform, RigidBody, ColorMaterial>().unwrap()]), -350);
    ecs.register_system(SHOOT_PROJECTILE, HashSet::from([ecs.get_system_signature_1::<VulkanRenderEngine>().unwrap(), ecs.get_system_signature_1::<MeshBinding>().unwrap(), ecs.get_system_signature_1::<Viewport2D>().unwrap(), ecs.get_system_signature_1::<CubeMeshOwner>().unwrap()]), -250);
    ecs.register_system(UPDATE_PARTICLES, HashSet::from([ecs.get_system_signature_2::<Transform, Particle>().unwrap(), ecs.get_system_signature_1::<TimeDelta>().unwrap()]), -200);
    ecs.register_system(UPDATE_RIGID_BODIES, HashSet::from([ecs.get_system_signature_2::<Transform, RigidBody>().unwrap(), ecs.get_system_signature_1::<TimeDelta>().unwrap()]), -200);
    ecs.register_system(UPDATE_QUAD_TREE, HashSet::from([ecs.get_system_signature_1::<QuadTree<BoundingSphere>>().unwrap(), ecs.get_system_signature_2::<Transform, RigidBody>().unwrap()]), -150);
    ecs.register_system(DETECT_PARTICLE_COLLISIONS, HashSet::from([ecs.get_system_signature_2::<Transform, Particle>().unwrap(), ecs.get_system_signature_1::<ParticleCollisionDetector>().unwrap()]), -100);
    ecs.register_system(DETECT_PARTICLE_CABLE_COLLISIONS, HashSet::from([ecs.get_system_signature_1::<ParticleCable>().unwrap()]), -100);
    ecs.register_system(DETECT_PARTICLE_ROD_COLLISIONS, HashSet::from([ecs.get_system_signature_1::<ParticleRod>().unwrap()]), -100);
    ecs.register_system(DETECT_POTENTIAL_RIGID_BODY_COLLISIONS, HashSet::from([ecs.get_system_signature_1::<QuadTree<BoundingSphere>>().unwrap()]), -100);
    ecs.register_system(DETECT_RIGID_BODY_COLLISIONS, HashSet::from([ecs.get_system_signature_1::<PotentialRigidBodyCollision>().unwrap(), ecs.get_system_signature_1::<PotentialRigidBodyCollision>().unwrap()]), -99);
    ecs.register_system(RESOLVE_PARTICLE_COLLISIONS, HashSet::from([ecs.get_system_signature_1::<TimeDelta>().unwrap(), ecs.get_system_signature_1::<ParticleCollision>().unwrap()]), -50);
    ecs.register_system(RESOLVE_RIGID_BODY_COLLISIONS, HashSet::from([ecs.get_system_signature_1::<TimeDelta>().unwrap(), ecs.get_system_signature_1::<RigidBodyCollision>().unwrap()]), -50);
    ecs.register_system(SYNC_RENDER_STATE, HashSet::from([ecs.get_system_signature_0().unwrap()]), 2);
    ecs.register_system(UPDATE_TIMERS, HashSet::from([ecs.get_system_signature_1::<Timer>().unwrap(), ecs.get_system_signature_1::<TimeDelta>().unwrap()]), 3);
    ecs.register_system(SHUTDOWN_RENDER_ENGINE, HashSet::from([ecs.get_system_signature_1::<VulkanRenderEngine>().unwrap()]), 999);
}

// Built-in
const SHUTDOWN_ECS: System = |entites: Iter<Entity>, components: &ComponentManager, commands: &mut ECSCommands| {
    entites.for_each(|e| {
        let render_engine = components.get_component::<VulkanRenderEngine>(e).unwrap();

        if render_engine.get_window().map_or(true, |w| w.is_closing()) {
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

// Built-in
const RESET_TRANSFORM_FLAGS: System = |entites: Iter<Entity>, components: &ComponentManager, _: &mut ECSCommands| {
    entites.for_each(|e| {
        let transform = components.get_mut_component::<Transform>(e).unwrap();

        transform.reset_changed_flags();
    });
};

const MOVE_CAMERA: System = |entites: Iter<Entity>, components: &ComponentManager, commands: &mut ECSCommands| {
    let render_engine = entites.clone().find_map(|e| components.get_component::<VulkanRenderEngine>(e)).unwrap();
    let time_delta = entites.clone().find_map(|e| components.get_component::<TimeDelta>(e)).unwrap();

    if let Ok(window) = render_engine.get_window() {
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

                let move_speed = 25.0 * time_delta.since_last_frame.as_secs_f32();
                if let Ok(dir) = move_dir.normalized() {
                    cam.pos += dir * move_speed;
                }

                let rot_speed = (240.0 * time_delta.since_last_frame.as_secs_f32()).to_radians();
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

    if render_engine.is_key_pressed(VirtualKey::Escape) {
        commands.shutdown();
    }
};

const SHOOT_PROJECTILE: System = |entites: Iter<Entity>, components: &ComponentManager, commands: &mut ECSCommands| {
    let render_engine = entites.clone().find_map(|e| components.get_component::<VulkanRenderEngine>(e)).unwrap();
    let mesh_binding = &entites.clone()
        .find(|e| components.get_component::<CubeMeshOwner>(e).is_some())
        .map(|e| *components.get_component::<MeshBinding>(e).unwrap())
        .unwrap();
    let mesh_props = entites.clone()
        .find(|e| components.get_component::<CubeMeshOwner>(e).is_some())
        .map(|e| components.get_component::<PhysicsMeshProperties>(e).unwrap())
        .unwrap();
    let cam = &entites.clone().find_map(|e| components.get_component::<Viewport2D>(e)).unwrap().cam;

    if render_engine.is_key_down(VirtualKey::Space) {
        let cam_dir_norm = cam.dir.normalized().unwrap();

        let color_material = ColorMaterial::new(PURPLE);
        let transform = Transform::new(cam.pos + cam_dir_norm * 5.0, QUAT_IDENTITY, vec3(1.0, 1.0, 1.0));
        // let particle = Particle::new(cam_dir_norm * 35.0, DAMPING, 5.0, 5.0);
        let rigid_body= RigidBody::new(cam_dir_norm * 35.0, VEC_3_ZERO, DAMPING, DAMPING, 5.0, mesh_props.clone());

        let proj_entity = commands.create_entity();
        commands.attach_provisional_component(&proj_entity, *mesh_binding);
        commands.attach_provisional_component(&proj_entity, color_material);
        commands.attach_provisional_component(&proj_entity, transform);
        // commands.attach_provisional_component(&proj_entity, particle);
        commands.attach_provisional_component(&proj_entity, rigid_body);
        commands.attach_provisional_component(&proj_entity, MousePickable {});
    } else if render_engine.is_key_pressed(VirtualKey::Enter) || render_engine.is_key_released(VirtualKey::Enter) {
        let cam_dir_norm = cam.dir.normalized().unwrap();

        let color_material = ColorMaterial::new(BLUE);
        let transform = Transform::new(cam.pos + cam_dir_norm * 5.0, QUAT_IDENTITY, vec3(3.0, 3.0, 3.0));
        let particle = Particle::new(cam_dir_norm * 5.0, 0.9, 1.0, -0.6);

        let proj_entity = commands.create_entity();
        commands.attach_provisional_component(&proj_entity, *mesh_binding);
        commands.attach_provisional_component(&proj_entity, color_material);
        commands.attach_provisional_component(&proj_entity, transform);
        commands.attach_provisional_component(&proj_entity, particle);
    }
};

const PICK_MESHES: System = |entites: Iter<Entity>, components: &ComponentManager, _: &mut ECSCommands| {
    let render_engine = entites.clone().find_map(|e| components.get_component::<VulkanRenderEngine>(e)).unwrap();
    let cam = &entites.clone().find_map(|e| components.get_component::<Viewport2D>(e)).unwrap().cam;

    const FORCE_FACTOR: f32 = 100000.0;

    if let Ok(window) = render_engine.get_window() {
        if window.is_button_pressed(VirtualButton::Left) {
            if let Some(screen_pos) = window.get_mouse_screen_position() {
                let ray = generate_ray(screen_pos, window, cam, NEAR_PLANE, FAR_PLANE);

                if let Ok(ray) = ray {
                    for e in entites {
                        if let Some(_) = components.get_component::<MousePickable>(e) {
                            let transform = components.get_mut_component::<Transform>(e).unwrap();
                            let mesh_binding = components.get_component::<MeshBinding>(e).unwrap();
                            let rigid_body = components.get_mut_component::<RigidBody>(e).unwrap();

                            let mesh = components.get_component::<Mesh>(&mesh_binding.mesh_wrapper.unwrap()).unwrap();

                            // TODO: optimize this by first pruning with a simplified mesh, such as a bounding sphere
                            //  Maybe formalize the MousePickable component into a built-in and add a bounding sphere field, then add a
                            //  function to it to do all the intersection checks, pruning, etc...
                            if let Some(intersection_point) = get_ray_intersection(&cam.pos, &ray, mesh, transform) {
                                rigid_body.add_force_at_point(&intersection_point, &(ray * FORCE_FACTOR), transform.get_pos());
                            }
                        }
                    }
                }
            }
        }
    }
};

// TODO: formalize force generator functions and move and generalize them to the physics module as I see fit
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

            particle.acc = particle.force_accum / particle.mass;
            particle.acc.y -= particle.gravity;

            particle.vel += particle.acc * delta;
            // Raising to the delta power makes the damping more realistic when frame times are inconsistent, especially when damping
            //  is not terribly close to 0. However, this operation is expensive, so we wouldn't want to do it when we're applying this
            //  to a huge number of particles, for example.
            particle.vel *= particle.damping.powf(delta);

            transform.set_pos(*transform.get_pos() + particle.vel * delta);

            particle.force_accum = VEC_3_ZERO;
        }
    }
};

// Built-in
const UPDATE_RIGID_BODIES: System = |entites: Iter<Entity>, components: &ComponentManager, _: &mut ECSCommands| {
    let time_delta = entites.clone().find_map(|e| components.get_component::<TimeDelta>(e)).unwrap();
    let delta = time_delta.since_last_frame.as_secs_f32();

    for e in entites {
        let transform = components.get_mut_component::<Transform>(e);
        let rigid_body = components.get_mut_component::<RigidBody>(e);

        if transform.is_some() && rigid_body.is_some() {
            let transform = transform.unwrap();
            let rigid_body = rigid_body.unwrap();

            // Linear motion
            rigid_body.linear_acc = rigid_body.linear_force_accum / rigid_body.props.mass;
            rigid_body.linear_acc.y -= rigid_body.gravity;

            rigid_body.linear_vel += rigid_body.linear_acc * delta;
            rigid_body.linear_vel *= rigid_body.linear_damping.powf(delta);

            transform.set_pos(*transform.get_pos() + rigid_body.linear_vel * delta);

            rigid_body.linear_force_accum = VEC_3_ZERO;

            // Rotational motion
            let world_matrix = transform.to_world_mat().to_mat3();
            let inverse_world_matrix = world_matrix.inverted().unwrap_or_else(|_| panic!("Internal error: failed to invert world matrix"));
            let inverse_inertia_tensor_world = (world_matrix * rigid_body.props.inertia_tensor * inverse_world_matrix).inverted()
                .unwrap_or_else(|_| panic!("Internal error: failed to invert inertia tensor world transform"));
            rigid_body.ang_acc = inverse_inertia_tensor_world * rigid_body.torque_accum;

            rigid_body.ang_vel += rigid_body.ang_acc * delta;
            rigid_body.ang_vel *= rigid_body.ang_damping.powf(delta);

            transform.set_rot(apply_ang_vel(transform.get_rot(), &rigid_body.ang_vel, delta));

            rigid_body.torque_accum = VEC_3_ZERO;
        }
    }
};

// Built-in
const UPDATE_QUAD_TREE: System = |entites: Iter<Entity>, components: &ComponentManager, _: &mut ECSCommands| {
    let quad_tree = entites.clone().find_map(|e| components.get_mut_component::<QuadTree<BoundingSphere>>(e)).unwrap();

    let entities_to_update = entites
        .filter(|e| {
            components.get_component::<Transform>(e).is_some()
                && components.get_component::<RigidBody>(e).is_some()
        })
        .collect::<HashSet<_>>();

    quad_tree.remove_not_in(&entities_to_update);

    for e in entities_to_update {
        let transform = components.get_component::<Transform>(e).unwrap();
        let rigid_body = components.get_component::<RigidBody>(e).unwrap();

        if transform.is_pos_changed_since_last_frame() || transform.is_scl_changed_since_last_frame() {
            quad_tree.remove(e).unwrap_or_default();

            let bounding_sphere = BoundingSphere::from_transform(transform, rigid_body.props.bounding_radius);

            quad_tree.insert(*e, bounding_sphere).unwrap_or_else(|e| panic!("Failed to insert bounding sphere into quad tree: {:?}", e));
        }
    }
};

// Built-in
const DETECT_POTENTIAL_RIGID_BODY_COLLISIONS: System = |entites: Iter<Entity>, components: &ComponentManager, commands: &mut ECSCommands| {
    let quad_tree = entites.clone().find_map(|e| components.get_component::<QuadTree<BoundingSphere>>(e)).unwrap();

    let potential_collisions = quad_tree.get_potential_collisions();

    for c in potential_collisions {
        let e = commands.create_entity();

        commands.attach_provisional_component(&e, c);
    }
};

// Built-in
const DETECT_RIGID_BODY_COLLISIONS: System = |entites: Iter<Entity>, components: &ComponentManager, commands: &mut ECSCommands| {
    let new_collisions = entites.clone()
        .map(|e| components.get_component::<PotentialRigidBodyCollision>(e).map(|c| (e, c)))
        .filter(|c| c.is_some())
        .map(|c| c.unwrap())
        .inspect(|(e, _)| commands.destroy_entity(e))
        .map(|(_, c)| {
            let mesh_binding_a = components.get_component::<MeshBinding>(&c.entity_a).unwrap_or_else(|| panic!("Failed to get mesh binding for entity {:?}", &c.entity_a));
            let mesh_binding_b = components.get_component::<MeshBinding>(&c.entity_b).unwrap_or_else(|| panic!("Failed to get mesh binding for entity {:?}", &c.entity_b));

            let mesh_a = components.get_component::<Mesh>(&mesh_binding_a.mesh_wrapper.unwrap()).unwrap_or_else(|| panic!("Failed to get mesh binding for wrapper entity {:?}", &mesh_binding_a.mesh_wrapper.unwrap()));
            let mesh_b = components.get_component::<Mesh>(&mesh_binding_b.mesh_wrapper.unwrap()).unwrap_or_else(|| panic!("Failed to get mesh binding for wrapper entity {:?}", &mesh_binding_b.mesh_wrapper.unwrap()));

            get_deepest_rigid_body_collision((&c.entity_a, mesh_a), (&c.entity_b, mesh_b))
        })
        .filter(|c| c.is_some())
        .map(|c| c.unwrap())
        .collect::<HashSet<_>>();

    const COLLISION_CACHE_TOLERANCE: f32 = -0.1;

    for e in entites.clone() {
        if let Some(collision) = components.get_component::<RigidBodyCollision>(e) {
            if new_collisions.contains(collision) {
                commands.destroy_entity(e);
            } else {
                let mesh_binding_a = components.get_component::<MeshBinding>(&collision.rigid_body_a);
                let mesh_binding_b = components.get_component::<MeshBinding>(&collision.rigid_body_b);

                let mesh_a = mesh_binding_a.map(|binding| components.get_component::<Mesh>(&binding.mesh_wrapper.unwrap()))
                    .filter(|m| m.is_some()).map(|m| m.unwrap());
                let mesh_b = mesh_binding_b.map(|binding| components.get_component::<Mesh>(&binding.mesh_wrapper.unwrap()))
                    .filter(|m| m.is_some()).map(|m| m.unwrap());

                if mesh_a.is_some() && mesh_b.is_some() {
                    if let Some(point_features) = collision.point_features {
                        // TODO: get the right values
                        if let Some(retained_collision) = get_point_collision(
                            &collision.rigid_body_a,
                            &collision.rigid_body_b,
                            &VEC_3_ZERO, // TODO
                            face_b, // TODO
                            COLLISION_CACHE_TOLERANCE,
                        ) {
                            commands.detach_component::<RigidBodyCollision>(e);
                            commands.attach_component(e, retained_collision);
                        }
                    } else if let Some(edge_features) = collision.edge_features {
                        // TODO: get the right values
                        if let Some(retained_collision) = get_edge_collision(
                            &collision.rigid_body_a,
                            &collision.rigid_body_b,
                            &VEC_3_ZERO, // TODO
                            face_b, // TODO
                            COLLISION_CACHE_TOLERANCE,
                        ) {
                            commands.detach_component::<RigidBodyCollision>(e);
                            commands.attach_component(e, retained_collision);
                        }
                    } else {
                        panic!("Rigid body collision between entities {:?} and {:?} has no collision features", &collision.rigid_body_a, &collision.rigid_body_b);
                    }
                } else {
                    commands.destroy_entity(e);
                }
            }
        }
    }

    for c in new_collisions.into_iter() {
        let collision_entity = commands.create_entity();

        commands.attach_provisional_component(&collision_entity, c);
    }
};

// Built-in
const RESOLVE_RIGID_BODY_COLLISIONS: System = |entites: Iter<Entity>, components: &ComponentManager, _: &mut ECSCommands| {
    // TODO: actual implementation

    for e in entites {
        if let Some(collision) = components.get_component::<RigidBodyCollision>(e) {
            println!("Resolving collision between entities {:?} and {:?}", &collision.rigid_body_a, &collision.rigid_body_b);
        }
    }
};

// TODO: make built in
const APPLY_TETHER_BALL: System = |entites: Iter<Entity>, components: &ComponentManager, _: &mut ECSCommands| {
    const TETHER_ANCHOR: Vec3 = vec3(0.0, 10.0, 3.0);
    const TETHER_LOCAL_POINT: Vec3 = vec3(0.0, 0.5, 0.0);
    const REST_LENGTH: f32 = 4.0;
    const K: f32 = 10.0;

    for (_, transform, rigid_body, material) in get_rigid_cubes(entites, components) {
        if material.color == GRAY {
            let world_point = local_to_world_point(&TETHER_LOCAL_POINT, transform);

            let to_anchor = TETHER_ANCHOR - world_point;
            let delta_length = to_anchor.len() - REST_LENGTH;

            if delta_length > 0.0 {
                let to_anchor_dir = to_anchor.normalized().unwrap();
                let force = K * delta_length * to_anchor_dir;

                rigid_body.add_force_at_point(&world_point, &force, transform.get_pos());
            }
        }
    }
};

const APPLY_RIGID_BODY_FORCE: System = |entites: Iter<Entity>, components: &ComponentManager, _: &mut ECSCommands| {
    let render_engine = entites.clone().find_map(|e| components.get_component::<VulkanRenderEngine>(e)).unwrap();

    if let Ok(window) = render_engine.get_window() {
        for e in entites {
            let transform = components.get_mut_component::<Transform>(e);
            let rigid_body = components.get_mut_component::<RigidBody>(e);

            if transform.is_some() && rigid_body.is_some() {
                let transform = transform.unwrap();
                let rigid_body = rigid_body.unwrap();

                let local_point = vec3(3.0, 0.0, 0.0);
                let local_force = VEC_3_Z_AXIS * 0.25;

                if window.is_key_down(VirtualKey::I) {
                    let world_point = local_to_world_point(&local_point, transform);
                    let world_force = local_to_world_force(&local_force, transform);

                    rigid_body.add_force_at_point(&world_point, &world_force, transform.get_pos());
                    rigid_body.linear_force_accum = VEC_3_ZERO;
                }
                if window.is_key_down(VirtualKey::K) {
                    let world_point = local_to_world_point(&local_point, transform);
                    let world_force = local_to_world_force(&-local_force, transform);

                    rigid_body.add_force_at_point(&world_point, &world_force, transform.get_pos());
                    rigid_body.linear_force_accum = VEC_3_ZERO;
                }
            }
        }
    }
};

const CHECK_OUT_OF_BOUNDS: System = |entites: Iter<Entity>, components: &ComponentManager, commands: &mut ECSCommands| {
    for e in entites {
        let transform = components.get_component::<Transform>(e).unwrap();

        const GAME_BOUNDS: f32 = 100.0;

        if transform.get_pos().len() >= GAME_BOUNDS {
            commands.destroy_entity(e);
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

// TODO: make a struct for particle springs
const APPLY_CEILING_SPRING: System = |entites: Iter<Entity>, components: &ComponentManager, _: &mut ECSCommands| {
    const CEIL_HEIGHT: f32 = 10.0;
    const REST_LENGTH: f32 = 3.0;
    const K: f32 = 10.0;

    for (_, transform, particle, material) in get_cubes(entites, components) {
        if material.color == WHITE || material.color == BLACK {
            let pos = transform.get_pos();
            let d = *pos - vec3(pos.x, CEIL_HEIGHT, pos.z);
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
            let d = *transform.get_pos() - *cam_pos;
            let delta_length = d.len() - REST_LENGTH;

            if delta_length > 0.0 {
                if let Ok(d_norm) = d.normalized() {
                    particle.force_accum += -K * delta_length * d_norm;
                }
            }
        }
    }
};

// TODO: make built-in
const APPLY_BUOYANCY: System = |entites: Iter<Entity>, components: &ComponentManager, _: &mut ECSCommands| {
    const DENSITY: f32 = 10.0;
    const WATER_HEIGHT: f32 = 0.0;

    for (_, transform, particle, material) in get_cubes(entites, components) {
        if material.color == MAGENTA {
            let submersion_depth = -transform.get_scl().y / 2.0;
            let volume = transform.get_scl().x * transform.get_scl().y * transform.get_scl().z;

            let d = ((transform.get_pos().y - WATER_HEIGHT - submersion_depth) / (2.0 * submersion_depth)).max(0.0).min(1.0);

            particle.force_accum += vec3(0.0, d * DENSITY * volume, 0.0);
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

            if transform.get_pos().y <= FLOOR_HEIGHT {
                let restitution = if rng.random_range(0.0..1.0) < 0.5 {
                    rng.random_range(0.0..1.0)
                } else {
                    collision_detector.default_restitution
                };

                let collision = ParticleCollision::new(e.clone(), None, restitution, VEC_3_Y_AXIS, FLOOR_HEIGHT - transform.get_pos().y);

                let collision_entity = commands.create_entity();
                commands.attach_provisional_component(&collision_entity, collision);
            }
        }
    }
};

// Built-in
const DETECT_PARTICLE_CABLE_COLLISIONS: System = |entites: Iter<Entity>, components: &ComponentManager, commands: &mut ECSCommands| {
    let cables = entites
        .map(|e| (e, components.get_component::<ParticleCable>(e)))
        .filter(|(_, c)| c.is_some())
        .map(|(e, c)| (e, c.unwrap()));

    for (e, c) in cables {
        let transform_a = components.get_component::<Transform>(&c.particle_a);
        let transform_b = components.get_component::<Transform>(&c.particle_b);

        if transform_a.is_some() && transform_b.is_some() {
            let transform_a = transform_a.unwrap();
            let transform_b = transform_b.unwrap();

            let delta_pos = *transform_b.get_pos() - *transform_a.get_pos();
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
        } else {
            commands.destroy_entity(e);
        }
    }
};

// Built-in
const DETECT_PARTICLE_ROD_COLLISIONS: System = |entites: Iter<Entity>, components: &ComponentManager, commands: &mut ECSCommands| {
    let rods = entites
        .map(|e| (e, components.get_component::<ParticleRod>(e)))
        .filter(|(_, r)| r.is_some())
        .map(|(e, r)| (e, r.unwrap()));

    for (e, r) in rods {
        let transform_a = components.get_component::<Transform>(&r.particle_a);
        let transform_b = components.get_component::<Transform>(&r.particle_b);

        if transform_a.is_some() && transform_b.is_some() {
            let transform_a = transform_a.unwrap();
            let transform_b = transform_b.unwrap();

            let delta_pos = *transform_b.get_pos() - *transform_a.get_pos();
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
        } else {
            commands.destroy_entity(e);
        }
    }
};

// Built-in
const RESOLVE_PARTICLE_COLLISIONS: System = |entites: Iter<Entity>, components: &ComponentManager, commands: &mut ECSCommands| {
    let time_delta = entites.clone().find_map(|e| components.get_component::<TimeDelta>(e)).unwrap();
    let delta_sec = time_delta.since_last_frame.as_secs_f32();

    let collisions = entites
        .map(|e| (e, components.get_component::<ParticleCollision>(e)))
        .filter(|(_, c)| c.is_some())
        .map(|(e, c)| (e, c.unwrap()))
        .into_iter();

    for (e, c) in collisions.clone() {
        if !is_particle_collision_valid(c, components) {
            commands.destroy_entity(e);
        }
    }

    let mut collisions = collisions
        .filter(|(_, c)| is_particle_collision_valid(c, components))
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

    for (e, _) in collisions {
        commands.destroy_entity(e);
    }
};

fn is_particle_collision_valid(collision: &ParticleCollision, components: &ComponentManager) -> bool {
    components.get_component::<Particle>(&collision.particle_a).is_some()
        && (collision.particle_b.is_none() || components.get_component::<Particle>(&collision.particle_b.unwrap()).is_some())
}

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

        transform_a.set_pos(*transform_a.get_pos() + mass_factor_a * collision.penetration * collision.normal);

        if let Some(particle_b) = particle_b {
            let transform_b = transform_b.unwrap();

            let mass_factor_b = particle_a.mass / (particle_a.mass + particle_b.mass);

            transform_b.set_pos(*transform_b.get_pos() + mass_factor_b * collision.penetration * -collision.normal);
        }
    }
}

// Built-in
const SYNC_RENDER_STATE: System = |entites: Iter<Entity>, components: &ComponentManager, _: &mut ECSCommands| {
    let render_engine = entites.clone().find_map(|e| components.get_mut_component::<VulkanRenderEngine>(e)).unwrap();
    let viewport = entites.clone().find_map(|e| components.get_component::<Viewport2D>(e)).unwrap();

    let entity_states = entites.clone().filter(|e|
        components.get_component::<Transform>(e).is_some()
        && components.get_component::<MeshBinding>(e).is_some()
        && components.get_component::<ColorMaterial>(e).is_some())
    .map(|e| EntityRenderState {
        world: *components.get_mut_component::<Transform>(e).unwrap().to_world_mat(),
        mesh_id: components.get_component::<MeshBinding>(e).unwrap().id.unwrap(),
        color: components.get_component::<ColorMaterial>(e).unwrap().color,
    }).collect();

    let aspect_ratio = render_engine.get_window().and_then(|w| {
        Ok((w.get_width() as f32) / (w.get_height() as f32))
    }).unwrap_or(1.0);
    let proj = get_proj_matrix(NEAR_PLANE, FAR_PLANE, viewport.cam.fov_rads, aspect_ratio).unwrap();

    let render_state = RenderState {
        view: viewport.cam.to_view_mat().unwrap(),
        proj,
        entity_states,
    };

    render_engine.sync_state(render_state).unwrap_or_default();
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
            let render_engine = components.get_mut_component::<VulkanRenderEngine>(e).unwrap();

            render_engine.join_render_thread().unwrap_or_else(|e| panic!("{}", e));
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

fn get_rigid_cubes<'a>(entities: Iter<'a, Entity>, components: &'a ComponentManager) -> impl Iterator<Item = (&'a Entity, &'a mut Transform, &'a mut RigidBody, &'a ColorMaterial)> {
    entities.map(|e| {
        let transform = components.get_mut_component::<Transform>(e);
        let rigid_body = components.get_mut_component::<RigidBody>(e);
        let material = components.get_component::<ColorMaterial>(e);

        if transform.is_some() && rigid_body.is_some() && material.is_some() {
            Some((e, transform.unwrap(), rigid_body.unwrap(), material.unwrap()))
        } else {
            None
        }
    })
    .filter(|e| e.is_some())
    .map(|e| e.unwrap())
}

// CubeMeshOwner

struct CubeMeshOwner {}

impl Component for CubeMeshOwner {}
impl ComponentActions for CubeMeshOwner {}

// MousePickable

struct MousePickable {}

impl Component for MousePickable {}
impl ComponentActions for MousePickable {}
