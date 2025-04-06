use anyhow::Result;
use ecs::ComponentActions;
use math::{get_proj_matrix, vec2, vec3, Quat, Vec2, Vec3, QUAT_IDENTITY, VEC_2_ZERO, VEC_3_Y_AXIS, VEC_3_ZERO, VEC_3_Z_AXIS};
use physics::{generate_ray, get_ray_intersection, PotentialRigidBodyCollision};
use render_engine::GuiState;
use core::mesh::create_quad_mesh;
use core::{TextureBinding, IDENTITY_SCALE_VEC, WHITE};
use std::cmp::Ordering;
use std::collections::hash_set::Iter;
use std::collections::HashSet;
use rand::Rng;
use std::time::Duration;

use crate::core::{Camera, Color, ColorMaterial, TimeDelta, Timer, Transform, Viewport2D};
use crate::core::mesh::{create_cube_mesh, create_plane_mesh, Mesh, MeshBinding};
use crate::ecs::component::{Component, ComponentManager};
use crate::ecs::entity::Entity;
use crate::ecs::system::System;
use crate::ecs::{ECSBuilder, ECSCommands, ECS};
use crate::physics::{apply_ang_vel, get_deepest_rigid_body_collision, get_edge_collision, get_point_collision, BoundingSphere, Particle, ParticleCable, ParticleRod, ParticleCollision, ParticleCollisionDetector, PhysicsMeshProperties, QuadTree, RigidBody, RigidBodyCollision};
use crate::render_engine::vulkan::VulkanRenderEngine;
use crate::render_engine::{Device, EntityRenderState, RenderEngine, RenderState, Window, RenderEngineInitProps, VirtualButton, VirtualKey, WindowInitProps};

pub mod core;
pub mod ecs;
pub mod math;
pub mod physics;
pub mod render_engine;

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
        .with_component::<TextureBinding>()
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
        .with_component::<PlaneMeshOwner>()
        .with_component::<QuadMeshOwner>()
        .with_component::<MousePickable>()
        .with_component::<CursorManager>()
        .with_component::<Player>()
        .with_component::<LevelLoader>()
        .with_component::<LevelEntity>()
        .with_component::<Baddie>()
        .with_component::<Wall>()
        .with_component::<BaddieTextureOwner>()
        .with_component::<GuiElement>()
        .build()
}

fn init_render_engine() -> Result<VulkanRenderEngine> {
    let window_props = WindowInitProps {
        width: 1600,
        height: 1200,
        title: String::from("My Cool Game"), // TODO: make this the game name
        is_resizable: true,
    };

    let render_engine_props = RenderEngineInitProps {
        debug_enabled: true,
        clear_color: Color::rgb(0.5, 0.0, 0.0),
        window_props,
    };

    VulkanRenderEngine::new(render_engine_props)
}

fn create_scene(ecs: &mut ECS) {
    let mut render_engine = init_render_engine().unwrap_or_else(|e| panic!("{}", e));

    let cube_mesh: Mesh = create_cube_mesh();
    let cube_mesh_id = render_engine.get_device_mut()
        .and_then(|d| d.create_mesh(cube_mesh.vertices.clone(), cube_mesh.vertex_indices.clone()))
        .unwrap_or_else(|e| panic!("{}", e));
    let cube_texture_id = render_engine.get_device_mut()
        .and_then(|d| d.create_texture(String::from("res/wall.png")))
        .unwrap_or_else(|e| panic!("{}", e));
    let cube_mesh_entity = ecs.create_entity();
    let cube_mesh_binding = MeshBinding::new_provisional(Some(cube_mesh_id), Some(cube_mesh_entity));
    let cube_texture_binding = TextureBinding::new_provisional(Some(cube_texture_id), Some(cube_mesh_entity));
    ecs.attach_provisional_component(&cube_mesh_entity, cube_mesh);
    ecs.attach_provisional_component(&cube_mesh_entity, cube_mesh_binding);
    ecs.attach_provisional_component(&cube_mesh_entity, cube_texture_binding);
    ecs.attach_provisional_component(&cube_mesh_entity, CubeMeshOwner {});

    let plane_mesh: Mesh = create_plane_mesh();
    let plane_mesh_id = render_engine.get_device_mut()
        .and_then(|d| d.create_mesh(plane_mesh.vertices.clone(), plane_mesh.vertex_indices.clone()))
        .unwrap_or_else(|e| panic!("{}", e));
    let plane_mesh_entity = ecs.create_entity();
    let plane_mesh_binding = MeshBinding::new_provisional(Some(plane_mesh_id), Some(plane_mesh_entity));
    ecs.attach_provisional_component(&plane_mesh_entity, plane_mesh);
    ecs.attach_provisional_component(&plane_mesh_entity, plane_mesh_binding);
    ecs.attach_provisional_component(&cube_mesh_entity, PlaneMeshOwner {});

    let quad_mesh: Mesh = create_quad_mesh();
    let quad_mesh_id = render_engine.get_device_mut()
        .and_then(|d| d.create_mesh(quad_mesh.vertices.clone(), quad_mesh.vertex_indices.clone()))
        .unwrap_or_else(|e| panic!("{}", e));
    let quad_mesh_entity = ecs.create_entity();
    let quad_mesh_binding = MeshBinding::new_provisional(Some(quad_mesh_id), Some(quad_mesh_entity));
    ecs.attach_provisional_component(&quad_mesh_entity, quad_mesh);
    ecs.attach_provisional_component(&quad_mesh_entity, quad_mesh_binding);
    ecs.attach_provisional_component(&quad_mesh_entity, QuadMeshOwner {});

    let baddie_texture_id = render_engine.get_device_mut()
        .and_then(|d| d.create_texture(String::from("res/baddie.png")))
        .unwrap_or_else(|e| panic!("{}", e));
    let baddie_texture_entity = ecs.create_entity();
    let baddie_texture_binding = TextureBinding::new_provisional(Some(baddie_texture_id), Some(baddie_texture_entity));
    ecs.attach_provisional_component(&baddie_texture_entity, baddie_texture_binding);
    ecs.attach_provisional_component(&baddie_texture_entity, BaddieTextureOwner {});

    ////////////////////
    // GUI
    ////////////////////

    // Crosshair
    let crosshair_texture_id = render_engine.get_device_mut()
        .and_then(|d| d.create_texture(String::from("res/crosshair.png")))
        .unwrap_or_else(|e| panic!("{}", e));
    let crosshair_element = GuiElement {
        id: String::from("crosshair"),
        position: vec2(0.0, 0.0),
        dimensions: vec2(1.0, 1.0),
    };
    let crosshair_entity = ecs.create_entity();
    let crosshair_texture_binding = TextureBinding::new_provisional(Some(crosshair_texture_id), Some(crosshair_entity));
    ecs.attach_provisional_component(&crosshair_entity, crosshair_element);
    ecs.attach_provisional_component(&crosshair_entity, crosshair_texture_binding);

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

    let cursor_manager = CursorManager { is_locked: false, just_locked: false, cursor_delta: VEC_2_ZERO };
    let cursor_manager_entity = ecs.create_entity();
    ecs.attach_provisional_component(&cursor_manager_entity, cursor_manager);

    let level_loader = LevelLoader { should_load: true, next_level_id: 0 };
    let level_loader_entity = ecs.create_entity();
    ecs.attach_provisional_component(&level_loader_entity, level_loader);

    ecs.register_system(SHUTDOWN_ECS, HashSet::from([ecs.get_system_signature_1::<VulkanRenderEngine>().unwrap()]), -999);
    ecs.register_system(TIME_SINCE_LAST_FRAME, HashSet::from([ecs.get_system_signature_1::<TimeDelta>().unwrap()]), -500);
    ecs.register_system(LOAD_LEVEL, HashSet::from([ecs.get_system_signature_1::<LevelLoader>().unwrap(), ecs.get_system_signature_1::<LevelEntity>().unwrap(), ecs.get_system_signature_1::<CubeMeshOwner>().unwrap(), ecs.get_system_signature_1::<PlaneMeshOwner>().unwrap(), ecs.get_system_signature_1::<Player>().unwrap()]), -400);
    ecs.register_system(MANAGE_CURSOR, HashSet::from([ecs.get_system_signature_1::<VulkanRenderEngine>().unwrap(), ecs.get_system_signature_1::<CursorManager>().unwrap()]), -400);
    ecs.register_system(SPAWN_BADDIES, HashSet::from([ecs.get_system_signature_1::<QuadMeshOwner>().unwrap(), ecs.get_system_signature_1::<BaddieTextureOwner>().unwrap(), ecs.get_system_signature_1::<Player>().unwrap(), ecs.get_system_signature_1::<Timer>().unwrap(), ecs.get_system_signature_1::<Viewport2D>().unwrap(), ecs.get_system_signature_2::<Wall, Transform>().unwrap()]), -400);
    ecs.register_system(MOVE_CAMERA, HashSet::from([ecs.get_system_signature_1::<VulkanRenderEngine>().unwrap(), ecs.get_system_signature_1::<Viewport2D>().unwrap(), ecs.get_system_signature_1::<TimeDelta>().unwrap(), ecs.get_system_signature_1::<CursorManager>().unwrap(), ecs.get_system_signature_1::<Player>().unwrap()]), -400);
    ecs.register_system(APPLY_PLAYER_WALL_COLLISIONS, HashSet::from([ecs.get_system_signature_1::<Viewport2D>().unwrap(), ecs.get_system_signature_1::<Wall>().unwrap(), ecs.get_system_signature_1::<Player>().unwrap()]), -400);
    ecs.register_system(UPDATE_BADDIE_IS_ACTIVE, HashSet::from([ecs.get_system_signature_1::<Viewport2D>().unwrap(), ecs.get_system_signature_1::<Baddie>().unwrap(), ecs.get_system_signature_2::<Wall, Transform>().unwrap()]), -400);
    ecs.register_system(MOVE_BADDIE, HashSet::from([ecs.get_system_signature_1::<Baddie>().unwrap(), ecs.get_system_signature_1::<TimeDelta>().unwrap(), ecs.get_system_signature_1::<Viewport2D>().unwrap()]), -400);
    ecs.register_system(DAMAGE_PLAYER, HashSet::from([ecs.get_system_signature_1::<Baddie>().unwrap(), ecs.get_system_signature_1::<Player>().unwrap(), ecs.get_system_signature_1::<Viewport2D>().unwrap(), ecs.get_system_signature_1::<LevelLoader>().unwrap()]), -400);
    ecs.register_system(SHOOT_BADDIES, HashSet::from([ecs.get_system_signature_1::<Baddie>().unwrap(), ecs.get_system_signature_1::<Viewport2D>().unwrap(), ecs.get_system_signature_2::<Wall, Transform>().unwrap(), ecs.get_system_signature_1::<VulkanRenderEngine>().unwrap(), ecs.get_system_signature_1::<CursorManager>().unwrap()]), -400);
    ecs.register_system(UPDATE_PARTICLES, HashSet::from([ecs.get_system_signature_2::<Transform, Particle>().unwrap(), ecs.get_system_signature_1::<TimeDelta>().unwrap()]), -200);
    ecs.register_system(UPDATE_RIGID_BODIES, HashSet::from([ecs.get_system_signature_2::<Transform, RigidBody>().unwrap(), ecs.get_system_signature_1::<TimeDelta>().unwrap()]), -200);
    ecs.register_system(UPDATE_QUAD_TREE, HashSet::from([ecs.get_system_signature_1::<QuadTree<BoundingSphere>>().unwrap(), ecs.get_system_signature_2::<Transform, RigidBody>().unwrap()]), -150);
    ecs.register_system(DETECT_PARTICLE_CABLE_COLLISIONS, HashSet::from([ecs.get_system_signature_1::<ParticleCable>().unwrap()]), -100);
    ecs.register_system(DETECT_PARTICLE_ROD_COLLISIONS, HashSet::from([ecs.get_system_signature_1::<ParticleRod>().unwrap()]), -100);
    ecs.register_system(DETECT_POTENTIAL_RIGID_BODY_COLLISIONS, HashSet::from([ecs.get_system_signature_1::<QuadTree<BoundingSphere>>().unwrap()]), -100);
    ecs.register_system(DETECT_RIGID_BODY_COLLISIONS, HashSet::from([ecs.get_system_signature_1::<PotentialRigidBodyCollision>().unwrap(), ecs.get_system_signature_1::<RigidBodyCollision>().unwrap()]), -99);
    ecs.register_system(RESOLVE_PARTICLE_COLLISIONS, HashSet::from([ecs.get_system_signature_1::<TimeDelta>().unwrap(), ecs.get_system_signature_1::<ParticleCollision>().unwrap()]), -50);
    ecs.register_system(DETECT_LOAD_NEXT_LEVEL, HashSet::from([ecs.get_system_signature_1::<LevelLoader>().unwrap(), ecs.get_system_signature_1::<Viewport2D>().unwrap()]), -50);
    ecs.register_system(UPDATE_GUI_ELEMENTS, HashSet::from([ecs.get_system_signature_1::<GuiElement>().unwrap(), ecs.get_system_signature_1::<VulkanRenderEngine>().unwrap()]), 2);
    ecs.register_system(SYNC_RENDER_STATE, HashSet::from([ecs.get_system_signature_0().unwrap()]), 2);
    ecs.register_system(RESET_TRANSFORM_FLAGS, HashSet::from([ecs.get_system_signature_1::<Transform>().unwrap()]), 3);
    ecs.register_system(UPDATE_TIMERS, HashSet::from([ecs.get_system_signature_1::<Timer>().unwrap(), ecs.get_system_signature_1::<TimeDelta>().unwrap()]), 5);
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

const SPAWN_BADDIES: System = |entites: Iter<Entity>, components: &ComponentManager, commands: &mut ECSCommands| {
    let quad_mesh_binding = entites.clone()
        .filter(|e| components.get_component::<QuadMeshOwner>(e).is_some())
        .map(|e| components.get_component::<MeshBinding>(e).unwrap())
        .next()
        .unwrap();
    let baddie_texture_binding = entites.clone()
        .filter(|e| components.get_component::<BaddieTextureOwner>(e).is_some())
        .map(|e| components.get_component::<TextureBinding>(e).unwrap())
        .next()
        .unwrap();
    let _cam = &entites.clone().find_map(|e| components.get_component::<Viewport2D>(e)).unwrap().cam;
    let player = entites.clone().find_map(|e| components.get_component::<Player>(e)).unwrap();
    let spawn_timer = entites.clone().find_map(|e| components.get_mut_component::<Timer>(e)).unwrap();

    if spawn_timer.remaining_duration.is_none() {
        let mut rng = rand::rng();

        if rng.random_range(0.0..1.0) < player.spawn_chance {
            // TODO: consider the proximity to the camera
            let spawn_x = rng.random_range((-player.level_width / 2.0)..(player.level_width / 2.0));
            let spawn_z = rng.random_range((-player.level_height / 2.0)..(player.level_height / 2.0));

            // TODO: check that (spawn_x, spawn_z) isn't inside a wall

            const BADDIE_HEIGHT: f32 = 10.0;
            const BADDIE_SIZE: f32 = 8.0;

            let rot_ang = rng.random_range(0.0..(std::f32::consts::PI * 2.0));

            let baddie_transform = Transform::new(vec3(spawn_x, BADDIE_HEIGHT, spawn_z), Quat::from_axis_spin(&VEC_3_Y_AXIS, rot_ang).unwrap(), IDENTITY_SCALE_VEC * BADDIE_SIZE);
            let baddie_entity = commands.create_entity();
            commands.attach_provisional_component(&baddie_entity, baddie_transform);
            commands.attach_provisional_component(&baddie_entity, quad_mesh_binding.clone());
            commands.attach_provisional_component(&baddie_entity, baddie_texture_binding.clone());
            commands.attach_provisional_component(&baddie_entity, Baddie { is_active: false });
            commands.attach_provisional_component(&baddie_entity, LevelEntity {});
        }

        spawn_timer.reset();
    }
};

const UPDATE_BADDIE_IS_ACTIVE: System = |entites: Iter<Entity>, components: &ComponentManager, _: &mut ECSCommands| {
    let cam = &entites.clone().find_map(|e| components.get_component::<Viewport2D>(e)).unwrap().cam;

    for e in entites.clone() {
        if let Some(baddie) = components.get_mut_component::<Baddie>(e) {
            let baddie_transform = components.get_component::<Transform>(e).unwrap();

            let line_of_sight_blocked = entites.clone()
                .any(|e| {
                    if components.get_component::<Wall>(e).is_some() {
                        let wall_transform = components.get_mut_component::<Transform>(e).unwrap();
                        let wall_mesh = components.get_component::<Mesh>(
                            components.get_component::<MeshBinding>(e).unwrap().mesh_wrapper.as_ref().unwrap()
                        ).unwrap();
    
                        if let Some(dist) = check_ray_intersects(baddie_transform.get_pos(), &(cam.pos - *baddie_transform.get_pos()).normalized().unwrap(), wall_mesh, wall_transform, false) {
                            return dist < (cam.pos - *baddie_transform.get_pos()).len();
                        }
                    }

                    false
                });

            baddie.is_active = !line_of_sight_blocked;
        }
    }
};

const MOVE_BADDIE: System = |entites: Iter<Entity>, components: &ComponentManager, _: &mut ECSCommands| {
    let cam = &entites.clone().find_map(|e| components.get_component::<Viewport2D>(e)).unwrap().cam;
    let time_delta = entites.clone().find_map(|e| components.get_component::<TimeDelta>(e)).unwrap();
    let delta_sec = time_delta.since_last_frame.as_secs_f32();

    const BADDIE_SPEED: f32 = 10.0;

    for e in entites {
        if let Some(baddie) = components.get_component::<Baddie>(e) {
            let transform =  components.get_mut_component::<Transform>(e).unwrap();

            if baddie.is_active {
                let towards_player = (vec3(cam.pos.x, 0.0, cam.pos.z) - vec3(transform.get_pos().x, 0.0, transform.get_pos().z)).normalized().unwrap();

                transform.set_pos(*transform.get_pos() + towards_player * BADDIE_SPEED * delta_sec);

                let mut angle_to_cam = towards_player.angle_rads_from(&VEC_3_Z_AXIS).unwrap();

                if VEC_3_Z_AXIS.cross(&towards_player).y < 0.0 {
                    angle_to_cam = 2.0 * std::f32::consts::PI - angle_to_cam;
                }

                transform.set_rot(Quat::from_axis_spin(&VEC_3_Y_AXIS, angle_to_cam).unwrap());
            }
        }
    }
};

const DAMAGE_PLAYER: System = |entites: Iter<Entity>, components: &ComponentManager, commands: &mut ECSCommands| {
    let cam = &entites.clone().find_map(|e| components.get_component::<Viewport2D>(e)).unwrap().cam;
    let player = entites.clone().find_map(|e| components.get_mut_component::<Player>(e)).unwrap();
    let level_loader = entites.clone().find_map(|e| components.get_mut_component::<LevelLoader>(e)).unwrap();

    const PERCENT_HEALTH_PER_BADDIE: f32 = 0.25;
    const DAMAGE_DISTANCE: f32 = 8.0;

    for e in entites {
        if components.get_component::<Baddie>(e).is_some() {
            let transform =  components.get_mut_component::<Transform>(e).unwrap();

            let dist_to_player = (*transform.get_pos() - cam.pos).len();

            if dist_to_player <= DAMAGE_DISTANCE {
                commands.destroy_entity(e);

                player.health_percentage -= PERCENT_HEALTH_PER_BADDIE;

                if player.health_percentage <= f32::EPSILON {
                    // ur bad
                    level_loader.should_load = true;
                    level_loader.next_level_id = 0;
                }
            }
        }
    }
};

const SHOOT_BADDIES: System = |entites: Iter<Entity>, components: &ComponentManager, commands: &mut ECSCommands| {
    let cam = &entites.clone().find_map(|e| components.get_component::<Viewport2D>(e)).unwrap().cam;
    let render_engine = entites.clone().find_map(|e| components.get_mut_component::<VulkanRenderEngine>(e)).unwrap();
    let cursor_manager = entites.clone().find_map(|e| components.get_mut_component::<CursorManager>(e)).unwrap();

    if let Ok(window) = render_engine.get_window() {
        if cursor_manager.is_locked && window.is_button_pressed(VirtualButton::Left) {
            let screen_coords = window.get_mouse_screen_position().unwrap();
            let ray_dir = generate_ray(&screen_coords, window, cam, NEAR_PLANE, FAR_PLANE).unwrap();

            let mut closest_baddie: Option<Entity> = None;
            let mut closest_obstacle = f32::MAX;

            for e in entites.clone() {
                let is_baddie = components.get_component::<Baddie>(e).is_some();

                if is_baddie || components.get_component::<Wall>(e).is_some() {
                    let mesh = components.get_component::<Mesh>(
                        components.get_component::<MeshBinding>(e).unwrap().mesh_wrapper.as_ref().unwrap()
                    ).unwrap();
                    let transform = components.get_mut_component::<Transform>(e).unwrap();

                    if let Some(dist) = check_ray_intersects(&cam.pos, &ray_dir, mesh, transform, is_baddie) {
                        if dist < closest_obstacle {
                            closest_baddie = if is_baddie {
                                Some(*e)
                            } else {
                                None
                            };

                            closest_obstacle = dist;
                        }
                    }
                }
            }

            if let Some(baddie) = closest_baddie {
                commands.destroy_entity(&baddie); // TODO: spinneroni
            }
        }
    }
};

fn check_ray_intersects(ray_origin: &Vec3, ray_dir: &Vec3, mesh: &Mesh, transform: &mut Transform, is_baddie: bool) -> Option<f32> {
    const BADDIE_COLLISION_Y_THRESHOLD: f32 = 0.25;

    let inverse_world_matrix = transform.to_world_mat().inverted().unwrap();

    get_ray_intersection(ray_origin, ray_dir, mesh, transform)
        .filter(|p| {
            let local_space_p = inverse_world_matrix * p.to_vec4(1.0);

            !is_baddie || local_space_p.x.abs() < BADDIE_COLLISION_Y_THRESHOLD
        })
        .map(|p| (*ray_origin - p).len())
}

const LOAD_LEVEL: System = |entites: Iter<Entity>, components: &ComponentManager, commands: &mut ECSCommands| {
    let level_loader = entites.clone().find_map(|e| components.get_mut_component::<LevelLoader>(e)).unwrap();

    if level_loader.should_load {
        let cube_mesh_binding = entites.clone()
            .filter(|e| components.get_component::<CubeMeshOwner>(e).is_some())
            .map(|e| components.get_component::<MeshBinding>(e).unwrap())
            .next()
            .unwrap();
        let cube_texture_binding = entites.clone()
            .filter(|e| components.get_component::<CubeMeshOwner>(e).is_some())
            .map(|e| components.get_component::<TextureBinding>(e).unwrap())
            .next()
            .unwrap();
        let existing_health = entites.clone().find_map(|e| components.get_component::<Player>(e)).map(|p| p.health_percentage);

        for e in entites.clone() {
            if components.get_component::<LevelEntity>(e).is_some() {
                commands.destroy_entity(e);
            }
        }

        let cam_pos = VEC_3_ZERO; // TODO load from level/level ID
        let cam_forward = -VEC_3_Z_AXIS; // TODO load from level/level ID

        let cam = Camera::new(cam_pos, cam_forward, VEC_3_Y_AXIS, 70.0_f32.to_radians());
        let viewport = Viewport2D::new(cam, VEC_2_ZERO, vec2(1.0, 1.0));
        let viewport_entity = commands.create_entity();
        commands.attach_provisional_component(&viewport_entity, viewport);
        commands.attach_provisional_component(&viewport_entity, LevelEntity {});

        const CUBE_SIZE: f32 = 10.0;

        let level_dim: u32 = 21; // TODO: load from file?

        for i in 0..level_dim {
            for j in 0..level_dim {
                let x_pos = CUBE_SIZE * (i as f32 - (level_dim as f32 - 1.0) / 2.0);
                let z_pos = CUBE_SIZE * (j as f32 - (level_dim as f32 - 1.0) / 2.0);

                let cube_pos = vec3(x_pos, 0.0, z_pos);

                let cube_transform = Transform::new(cube_pos, QUAT_IDENTITY, IDENTITY_SCALE_VEC * CUBE_SIZE);
                let cube_entity = commands.create_entity();
                commands.attach_provisional_component(&cube_entity, cube_transform);
                commands.attach_provisional_component(&cube_entity, cube_texture_binding.clone());
                commands.attach_provisional_component(&cube_entity, cube_mesh_binding.clone());
                commands.attach_provisional_component(&cube_entity, LevelEntity {});

                // TODO: check from level file
                if i == 6 && j > 3 && j < 10 {
                    create_walls(commands, cube_texture_binding, cube_mesh_binding, x_pos, z_pos, CUBE_SIZE, 3);
                }
            }
        }

        const MAX_HEALTH: u32 = 100;

        let spawn_chance = 0.03; // TODO: ever update this with level?

        let player = if level_loader.next_level_id == 0 {
            Player {
                y_vel: 0.0,
                is_jumping: false,
                health_percentage: 1.0,
                max_health: MAX_HEALTH,
                level_width: level_dim as f32 * CUBE_SIZE,
                level_height: level_dim  as f32 * CUBE_SIZE,
                spawn_chance,
                cube_size: CUBE_SIZE,
            }
        } else {
            Player {
                y_vel: 0.0,
                is_jumping: false,
                health_percentage: existing_health.unwrap(),
                max_health: MAX_HEALTH,
                level_width: level_dim as f32 * CUBE_SIZE,
                level_height: level_dim  as f32 * CUBE_SIZE,
                spawn_chance,
                cube_size: CUBE_SIZE,
            }
        };

        let spawn_timer = Timer::for_initial_duration(Duration::from_millis(100));

        let player_entity = commands.create_entity();
        commands.attach_provisional_component(&player_entity, player);
        commands.attach_provisional_component(&player_entity, LevelEntity {});
        commands.attach_provisional_component(&player_entity, spawn_timer);

        level_loader.next_level_id += 1;
        level_loader.should_load = false;
    }
};

fn create_walls(commands: &mut ECSCommands, texture_binding: &TextureBinding, mesh_binding: &MeshBinding, x: f32, z: f32, cube_size: f32, stack_height: u32) {
    for i in 0..stack_height {
        let wall_transform = Transform::new(vec3(x, (i + 1) as f32 * cube_size, z), QUAT_IDENTITY, IDENTITY_SCALE_VEC * cube_size);

        let wall_entity = commands.create_entity();
        commands.attach_provisional_component(&wall_entity, wall_transform);
        commands.attach_provisional_component(&wall_entity, texture_binding.clone());
        commands.attach_provisional_component(&wall_entity, mesh_binding.clone());
        commands.attach_provisional_component(&wall_entity, LevelEntity {});
        commands.attach_provisional_component(&wall_entity, Wall {});
    }
}

const DETECT_LOAD_NEXT_LEVEL: System = |entites: Iter<Entity>, components: &ComponentManager, _: &mut ECSCommands| {
    let level_loader = entites.clone().find_map(|e| components.get_mut_component::<LevelLoader>(e)).unwrap();
    let cam = &entites.clone().find_map(|e| components.get_component::<Viewport2D>(e)).unwrap().cam;

    // TODO: implement the actual logic

    if cam.pos.xz().len() >= 150.0 {
        level_loader.should_load = true;
    }
};

const MANAGE_CURSOR: System = |entites: Iter<Entity>, components: &ComponentManager, _: &mut ECSCommands| {
    let render_engine = entites.clone().find_map(|e| components.get_mut_component::<VulkanRenderEngine>(e)).unwrap();
    let cursor_manager = entites.clone().find_map(|e| components.get_mut_component::<CursorManager>(e)).unwrap();

    let esc_pressed = render_engine.is_key_pressed(VirtualKey::Escape);

    const DEBUG_CURSOR: bool = false;

    if let Ok(window) = render_engine.get_window_mut() {
        let rel_center_pos = vec2(window.get_width() as f32 / 2.0, window.get_height() as f32 / 2.0);
        let window_screen_pos = window.get_screen_position();
        let cursor_screen_pos = window.get_mouse_screen_position().map(|p| *p);

        let center_pos = window_screen_pos + rel_center_pos;

        if let Some(cursor_screen_pos) = cursor_screen_pos {
            if !cursor_manager.is_locked && window.is_button_pressed(VirtualButton::Left) {
                if !cursor_manager.just_locked {
                    window.set_mouse_cursor_visible(false || DEBUG_CURSOR).unwrap_or_default();
                }

                if (cursor_screen_pos - rel_center_pos).len() > f32::EPSILON {
                    window.set_mouse_screen_position(&center_pos).unwrap_or_default();
                }

                cursor_manager.cursor_delta = VEC_2_ZERO;
                cursor_manager.just_locked = true;

                cursor_manager.cursor_delta = VEC_2_ZERO;
            } else if cursor_manager.is_locked && esc_pressed {
                cursor_manager.is_locked = false;

                window.set_mouse_cursor_visible(true).unwrap_or_default();

                cursor_manager.cursor_delta = VEC_2_ZERO;
            } else if cursor_manager.is_locked {
                cursor_manager.cursor_delta = cursor_screen_pos - rel_center_pos;

                if cursor_manager.cursor_delta.len() > f32::EPSILON {
                    window.set_mouse_screen_position(&center_pos).unwrap_or_default();
                }
            } else if cursor_manager.just_locked && (cursor_screen_pos - rel_center_pos).len() < 1.0 {
                cursor_manager.just_locked = false;
                cursor_manager.is_locked = true;
            }
        }
    } else {
        cursor_manager.is_locked = false;
        cursor_manager.just_locked = false;
        cursor_manager.cursor_delta = VEC_2_ZERO;
    }

    cursor_manager.cursor_delta.x = cursor_manager.cursor_delta.x as i32 as f32;
    cursor_manager.cursor_delta.y = cursor_manager.cursor_delta.y as i32 as f32;
};

const MOVE_CAMERA: System = |entites: Iter<Entity>, components: &ComponentManager, _: &mut ECSCommands| {
    let render_engine = entites.clone().find_map(|e| components.get_component::<VulkanRenderEngine>(e)).unwrap();
    let time_delta = entites.clone().find_map(|e| components.get_component::<TimeDelta>(e)).unwrap();
    let delta_sec = time_delta.since_last_frame.as_secs_f32();
    let cursor_manager = entites.clone().find_map(|e| components.get_mut_component::<CursorManager>(e)).unwrap();
    let cam = &mut entites.clone().find_map(|e| components.get_mut_component::<Viewport2D>(e)).unwrap().cam;
    let player = entites.clone().find_map(|e| components.get_mut_component::<Player>(e)).unwrap();

    const PLAYER_GRAVITY: f32 = -40.0;
    const MIN_PLAYER_HEIGHT: f32 = 15.0;
    const JUMP_VEL: f32 = 20.0;

    player.y_vel += PLAYER_GRAVITY * delta_sec;

    if let Ok(window) = render_engine.get_window() {
        if cursor_manager.is_locked {
            let mut move_dir = VEC_3_ZERO;
            let cam_right_norm = cam.dir.cross(&cam.up).normalized().unwrap();

            let move_forward = vec3(cam.dir.x, 0.0, cam.dir.z).normalized().unwrap();
            let move_right = vec3(cam_right_norm.x, 0.0, cam_right_norm.z).normalized().unwrap();

            if window.is_key_down(VirtualKey::W) {
                move_dir += move_forward;
            }
            if window.is_key_down(VirtualKey::S) {
                move_dir -= move_forward;
            }
            if window.is_key_down(VirtualKey::D) {
                move_dir += move_right;
            }
            if window.is_key_down(VirtualKey::A) {
                move_dir -= move_right;
            }

            let move_speed = 25.0 * delta_sec;
            if let Ok(dir) = move_dir.normalized() {
                cam.pos += dir * move_speed;
            }

            let rot_speed = (70.0 * delta_sec).to_radians();
            if cursor_manager.cursor_delta.y.abs() > f32::EPSILON {
                let max_rot = VEC_3_Y_AXIS.angle_rads_from(&cam.dir).unwrap() - 0.1;
                let min_rot = -(-VEC_3_Y_AXIS).angle_rads_from(&cam.dir).unwrap() + 0.1;

                let rot_amt = (rot_speed * -cursor_manager.cursor_delta.y).min(max_rot).max(min_rot);

                cam.dir = cam.dir.rotated(&cam_right_norm, rot_amt).unwrap().normalized().unwrap();
                cam.up = cam_right_norm.cross(&cam.dir).normalized().unwrap();
            }
            if cursor_manager.cursor_delta.x.abs() > f32::EPSILON {
                let rot_amt = rot_speed * -cursor_manager.cursor_delta.x;

                cam.dir = cam.dir.rotated(&VEC_3_Y_AXIS, rot_amt).unwrap().normalized().unwrap();

                let cam_right_norm = cam.dir.cross(&VEC_3_Y_AXIS).normalized().unwrap();
                cam.up = cam_right_norm.cross(&cam.dir).normalized().unwrap();
            }

            if window.is_key_down(VirtualKey::Space) && !player.is_jumping {
                player.y_vel += JUMP_VEL;

                player.is_jumping = true;
            }
        }
    }

    cam.pos.y += player.y_vel * delta_sec;

    if cam.pos.y <= MIN_PLAYER_HEIGHT {
        cam.pos.y = MIN_PLAYER_HEIGHT;

        player.is_jumping = false;
        player.y_vel = 0.0;
    }
};

const APPLY_PLAYER_WALL_COLLISIONS: System = |entites: Iter<Entity>, components: &ComponentManager, _: &mut ECSCommands| {
    let cam = &mut entites.clone().find_map(|e| components.get_mut_component::<Viewport2D>(e)).unwrap().cam;
    let player = entites.clone().find_map(|e| components.get_component::<Player>(e)).unwrap();

    const COLLISION_DIST: f32 = 5.0;

    for e in entites {
        if components.get_component::<Wall>(e).is_some() {
            let wall_transform = components.get_mut_component::<Transform>(e).unwrap();
            let wall_mesh = components.get_component::<Mesh>(
                components.get_component::<MeshBinding>(e).unwrap().mesh_wrapper.as_ref().unwrap()
            ).unwrap();

            let dist_to_cam = (cam.pos - *wall_transform.get_pos()).len();

            if dist_to_cam <= COLLISION_DIST + player.cube_size / 2.0 { // prune
                if let Some(collision) = get_wall_collision(&cam.pos, COLLISION_DIST, wall_mesh, &wall_transform) {
                    cam.pos += collision;
                }
            }
        }
    }
};

fn get_wall_collision(point: &Vec3, collision_dist: f32, mesh: &Mesh, transform: &Transform) -> Option<Vec3> {
    // TODO

    None
}

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
            if let Some(mass) = rigid_body.props.mass {
                rigid_body.linear_acc = rigid_body.linear_force_accum / mass;
                rigid_body.linear_acc.y -= rigid_body.gravity;

                rigid_body.linear_vel += rigid_body.linear_acc * delta;
                rigid_body.linear_vel *= rigid_body.linear_damping.powf(delta);

                transform.set_pos(*transform.get_pos() + rigid_body.linear_vel * delta);
            } else {
                rigid_body.linear_acc = VEC_3_ZERO;

                rigid_body.linear_vel = VEC_3_ZERO;
            }

            rigid_body.linear_force_accum = VEC_3_ZERO;

            // Rotational motion
            if let Some(inertia_tensor) = rigid_body.props.inertia_tensor {
                let world_matrix = transform.to_world_mat().to_mat3();
                let inverse_world_matrix = world_matrix.inverted().unwrap_or_else(|_| panic!("Internal error: failed to invert world matrix"));
                let inverse_inertia_tensor_world = (world_matrix * inertia_tensor * inverse_world_matrix).inverted()
                    .unwrap_or_else(|_| panic!("Internal error: failed to invert inertia tensor world transform"));

                rigid_body.ang_acc = inverse_inertia_tensor_world * rigid_body.torque_accum;

                rigid_body.ang_vel += rigid_body.ang_acc * delta;
                rigid_body.ang_vel *= rigid_body.ang_damping.powf(delta);

                transform.set_rot(apply_ang_vel(transform.get_rot(), &rigid_body.ang_vel, delta));
            } else {
                rigid_body.ang_acc = VEC_3_ZERO;

                rigid_body.ang_vel = VEC_3_ZERO;
            }

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
            let transform_a = components.get_mut_component::<Transform>(&c.entity_a);
            let transform_b = components.get_mut_component::<Transform>(&c.entity_b);

            let mesh_binding_a = components.get_component::<MeshBinding>(&c.entity_a);
            let mesh_binding_b = components.get_component::<MeshBinding>(&c.entity_b);

            let mesh_a = mesh_binding_a.map(|binding| components.get_component::<Mesh>(&binding.mesh_wrapper.unwrap()))
                .filter(|m| m.is_some()).map(|m| m.unwrap());
            let mesh_b = mesh_binding_b.map(|binding| components.get_component::<Mesh>(&binding.mesh_wrapper.unwrap()))
                .filter(|m| m.is_some()).map(|m| m.unwrap());

            if transform_a.is_some() && transform_b.is_some() && mesh_a.is_some() && mesh_b.is_some() {
                let transform_a = transform_a.unwrap();
                let transform_b = transform_b.unwrap();
                let mesh_a = mesh_a.unwrap();
                let mesh_b = mesh_b.unwrap();

                get_deepest_rigid_body_collision(
                    (&c.entity_a, mesh_a),
                    (&c.entity_b, mesh_b),
                    transform_a,
                    transform_b,
                )
            } else {
                None
            }
        })
        .filter(|c| c.is_some())
        .map(|c| c.unwrap())
        .collect::<HashSet<_>>();

    const COLLISION_CACHE_TOLERANCE: f32 = -0.01;

    for e in entites.clone() {
        if let Some(collision) = components.get_component::<RigidBodyCollision>(e) {
            if new_collisions.contains(collision) {
                commands.destroy_entity(e);
            } else {
                let transform_a = components.get_mut_component::<Transform>(&collision.rigid_body_a);
                let transform_b = components.get_mut_component::<Transform>(&collision.rigid_body_b);

                let mesh_binding_a = components.get_component::<MeshBinding>(&collision.rigid_body_a);
                let mesh_binding_b = components.get_component::<MeshBinding>(&collision.rigid_body_b);

                let mesh_a = mesh_binding_a.map(|binding| components.get_component::<Mesh>(&binding.mesh_wrapper.unwrap()))
                    .filter(|m| m.is_some()).map(|m| m.unwrap());
                let mesh_b = mesh_binding_b.map(|binding| components.get_component::<Mesh>(&binding.mesh_wrapper.unwrap()))
                    .filter(|m| m.is_some()).map(|m| m.unwrap());

                if transform_a.is_some() && transform_b.is_some() && mesh_a.is_some() && mesh_b.is_some() {
                    let transform_a = transform_a.unwrap();
                    let transform_b = transform_b.unwrap();
                    let mesh_a = mesh_a.unwrap();
                    let mesh_b = mesh_b.unwrap();

                    if let Some(point_features) = collision.point_features {
                        let vertex_a = &mesh_a.vertices[point_features.0 as usize];
                        let vertex_pos_a = (*transform_a.to_world_mat() * vertex_a.pos.to_vec4(1.0)).xyz();

                        let face_b = (
                            &(*transform_b.to_world_mat() * mesh_b.vertices[point_features.1.0 as usize].pos.to_vec4(1.0)).xyz(),
                            &(*transform_b.to_world_mat() * mesh_b.vertices[point_features.1.1 as usize].pos.to_vec4(1.0)).xyz(),
                            &(*transform_b.to_world_mat() * mesh_b.vertices[point_features.1.2 as usize].pos.to_vec4(1.0)).xyz(),
                        );

                        if let Some(mut retained_collision) = get_point_collision(
                            &collision.rigid_body_a,
                            &collision.rigid_body_b,
                            &vertex_pos_a,
                            face_b,
                            transform_b.get_pos(),
                            COLLISION_CACHE_TOLERANCE,
                        ) {
                            commands.detach_component::<RigidBodyCollision>(e);

                            retained_collision.point_features = Some(point_features);

                            commands.attach_component(e, retained_collision);
                        } else {
                            commands.destroy_entity(e);
                        }
                    } else if let Some(edge_features) = collision.edge_features {
                        let vertex_a_0 = &mesh_a.vertices[edge_features.0.0 as usize];
                        let vertex_a_1 = &mesh_a.vertices[edge_features.0.1 as usize];

                        let vertex_pos_a_0 = (*transform_a.to_world_mat() * vertex_a_0.pos.to_vec4(1.0)).xyz();
                        let vertex_pos_a_1 = (*transform_a.to_world_mat() * vertex_a_1.pos.to_vec4(1.0)).xyz();

                        let vertex_pos_b_0 = (*transform_b.to_world_mat() * mesh_b.vertices[edge_features.1.0 as usize].pos.to_vec4(1.0)).xyz();
                        let vertex_pos_b_1 = (*transform_b.to_world_mat() * mesh_b.vertices[edge_features.1.1 as usize].pos.to_vec4(1.0)).xyz();

                        if let Some(mut retained_collision) = get_edge_collision(
                            &collision.rigid_body_a,
                            &collision.rigid_body_b,
                            (&vertex_pos_a_0, &vertex_pos_a_1),
                            (&vertex_pos_b_0, &vertex_pos_b_1),
                            COLLISION_CACHE_TOLERANCE,
                        ) {
                            commands.detach_component::<RigidBodyCollision>(e);

                            retained_collision.edge_features = Some(edge_features);

                            commands.attach_component(e, retained_collision);
                        } else {
                            commands.destroy_entity(e);
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

const UPDATE_GUI_ELEMENTS: System = |entites: Iter<Entity>, components: &ComponentManager, _: &mut ECSCommands| {
    let render_engine = entites.clone().find_map(|e| components.get_component::<VulkanRenderEngine>(e)).unwrap();

    if let Ok(window) = render_engine.get_window() {
        let aspect_ratio = window.get_width() as f32 / window.get_height() as f32;

        for e in entites {
            if let Some(gui_element) = components.get_mut_component::<GuiElement>(e) {
                if gui_element.id == "crosshair" {
                    const CROSSHAIR_SIZE: f32 = 0.05;

                    gui_element.dimensions = vec2(CROSSHAIR_SIZE, CROSSHAIR_SIZE * aspect_ratio);
                } else {
                    panic!("Bad GUI element ID {:?}", gui_element.id);
                }
            }
        }
    }
};

// Built-in
const SYNC_RENDER_STATE: System = |entites: Iter<Entity>, components: &ComponentManager, _: &mut ECSCommands| {
    let render_engine = entites.clone().find_map(|e| components.get_mut_component::<VulkanRenderEngine>(e)).unwrap();
    let viewport = entites.clone().find_map(|e| components.get_component::<Viewport2D>(e)).unwrap();
    let quad_mesh_id = entites.clone()
        .filter(|e| components.get_component::<QuadMeshOwner>(e).is_some())
        .map(|e| components.get_component::<MeshBinding>(e).unwrap())
        .next()
        .unwrap().id.unwrap();

    let entity_states = entites.clone().filter(|e|
        components.get_component::<Transform>(e).is_some()
        && components.get_component::<MeshBinding>(e).is_some()
        && components.get_component::<TextureBinding>(e).is_some())
    .map(|e| EntityRenderState {
        world: *components.get_mut_component::<Transform>(e).unwrap().to_world_mat(),
        mesh_id: components.get_component::<MeshBinding>(e).unwrap().id.unwrap(),
        texture_id: components.get_component::<TextureBinding>(e).unwrap().id.unwrap(),
        color: WHITE,
    }).collect();

    let gui_states = entites.clone()
        .filter(|e| components.get_component::<TextureBinding>(e).is_some() && components.get_component::<GuiElement>(e).is_some())
        .map(|e| GuiState {
            mesh_id: quad_mesh_id,
            texture_id: components.get_component::<TextureBinding>(e).unwrap().id.unwrap(),
            position: components.get_component::<GuiElement>(e).unwrap().position,
            dimensions: components.get_component::<GuiElement>(e).unwrap().dimensions,
        }).collect();

    let aspect_ratio = render_engine.get_window().and_then(|w| {
        Ok((w.get_width() as f32) / (w.get_height() as f32))
    }).unwrap_or(1.0);
    let proj = get_proj_matrix(NEAR_PLANE, FAR_PLANE, viewport.cam.fov_rads, aspect_ratio).unwrap();

    let render_state = RenderState {
        view: viewport.cam.to_view_mat().unwrap(),
        proj,
        entity_states,
        gui_states,
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

// CubeMeshOwner

struct CubeMeshOwner {}

impl Component for CubeMeshOwner {}
impl ComponentActions for CubeMeshOwner {}

// PlaneMeshOwner

struct PlaneMeshOwner {}

impl Component for PlaneMeshOwner {}
impl ComponentActions for PlaneMeshOwner {}

// QuadMeshOwner

struct QuadMeshOwner {}

impl Component for QuadMeshOwner {}
impl ComponentActions for QuadMeshOwner {}

// MousePickable

struct MousePickable {}

impl Component for MousePickable {}
impl ComponentActions for MousePickable {}

// CursorManager

struct CursorManager {
    is_locked: bool,
    just_locked: bool,
    cursor_delta: Vec2,
}

impl Component for CursorManager {}
impl ComponentActions for CursorManager {}

struct Player {
    y_vel: f32,
    is_jumping: bool,
    health_percentage: f32,
    max_health: u32,
    level_width: f32,
    level_height: f32,
    cube_size: f32,
    spawn_chance: f32,
}

impl Component for Player {}
impl ComponentActions for Player {}

struct LevelLoader {
    should_load: bool,
    next_level_id: usize,
}

impl Component for LevelLoader {}
impl ComponentActions for LevelLoader {}

struct LevelEntity {}

impl Component for LevelEntity {}
impl ComponentActions for LevelEntity {}

struct Baddie {
    is_active: bool,
}

impl Component for Baddie {}
impl ComponentActions for Baddie {}

struct Wall {}

impl Component for Wall {}
impl ComponentActions for Wall {}

struct BaddieTextureOwner {}

impl Component for BaddieTextureOwner {}
impl ComponentActions for BaddieTextureOwner {}

struct GuiElement {
    id: String,
    position: Vec2,
    dimensions: Vec2,
}

impl Component for GuiElement {}
impl ComponentActions for GuiElement {}
