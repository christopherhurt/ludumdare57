use anyhow::Result;
use ecs::ComponentActions;
use math::{get_proj_matrix, vec2, vec3, Vec2, QUAT_IDENTITY, VEC_2_ZERO, VEC_3_Y_AXIS, VEC_3_ZERO, VEC_3_Z_AXIS};
use physics::PotentialRigidBodyCollision;
use core::{IDENTITY_SCALE_VEC, RED};
use std::cmp::Ordering;
use std::collections::hash_set::Iter;
use std::collections::HashSet;

use crate::core::{Camera, Color, ColorMaterial, TimeDelta, Timer, Transform, Viewport2D};
use crate::core::mesh::{create_cube_mesh, create_plane_mesh, Mesh, MeshBinding};
use crate::ecs::component::{Component, ComponentManager};
use crate::ecs::entity::Entity;
use crate::ecs::system::System;
use crate::ecs::{ECSBuilder, ECSCommands, ECS};
use crate::physics::{apply_ang_vel, generate_physics_mesh, get_deepest_rigid_body_collision, get_edge_collision, get_point_collision, BoundingSphere, Particle, ParticleCable, ParticleRod, ParticleCollision, ParticleCollisionDetector, PhysicsMeshProperties, QuadTree, RigidBody, RigidBodyCollision};
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
        .with_component::<CursorManager>()
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
        clear_color: Color::rgb(0.0, 0.3, 0.0),
        window_props,
    };

    VulkanRenderEngine::new(render_engine_props)
}

fn create_scene(ecs: &mut ECS) {
    let mut render_engine = init_render_engine().unwrap_or_else(|e| panic!("{}", e));

    let cam = Camera::new(VEC_3_ZERO, -VEC_3_Z_AXIS, VEC_3_Y_AXIS, 70.0_f32.to_radians());
    let viewport = Viewport2D::new(cam, VEC_2_ZERO, vec2(1.0, 1.0));
    let player_entity = ecs.create_entity();
    ecs.attach_provisional_component(&player_entity, viewport);

    let cube_mesh: Mesh = create_cube_mesh();
    let (cube_mesh, cube_physics_props) = generate_physics_mesh(cube_mesh, Some(2.0)).unwrap();
    let cube_mesh_id = render_engine.get_device_mut()
        .and_then(|d| d.create_mesh(cube_mesh.vertices.clone(), cube_mesh.vertex_indices.clone()))
        .unwrap_or_else(|e| panic!("{}", e));
    let cube_mesh_entity = ecs.create_entity();
    let cube_mesh_binding = MeshBinding::new_provisional(Some(cube_mesh_id), Some(cube_mesh_entity));
    ecs.attach_provisional_component(&cube_mesh_entity, cube_mesh);
    ecs.attach_provisional_component(&cube_mesh_entity, cube_mesh_binding);
    ecs.attach_provisional_component(&cube_mesh_entity, cube_physics_props.clone());
    ecs.attach_provisional_component(&cube_mesh_entity, CubeMeshOwner {});

    let plane_mesh: Mesh = create_plane_mesh();
    let (plane_mesh, _plane_physics_props) = generate_physics_mesh(plane_mesh, None).unwrap();
    let plane_physics_props = PhysicsMeshProperties::new_immovable(1.0, VEC_3_ZERO, 1.0);
    let plane_mesh_id = render_engine.get_device_mut()
        .and_then(|d| d.create_mesh(plane_mesh.vertices.clone(), plane_mesh.vertex_indices.clone()))
        .unwrap_or_else(|e| panic!("{}", e));
    let plane_mesh_entity = ecs.create_entity();
    let plane_mesh_binding = MeshBinding::new_provisional(Some(plane_mesh_id), Some(plane_mesh_entity));
    ecs.attach_provisional_component(&plane_mesh_entity, plane_mesh);
    ecs.attach_provisional_component(&plane_mesh_entity, plane_mesh_binding);
    ecs.attach_provisional_component(&plane_mesh_entity, plane_physics_props.clone());

    let test_cube_transform = Transform::new(vec3(0.0, 0.0, -10.0), QUAT_IDENTITY, IDENTITY_SCALE_VEC);
    let test_cube_material = ColorMaterial::new(RED);
    let test_cube_rigid_body = RigidBody::new(VEC_3_ZERO, VEC_3_ZERO, 0.6, 0.6, 0.0, cube_physics_props.clone());
    let test_cube_entity = ecs.create_entity();
    let test_cube_mesh_binding = MeshBinding::new_provisional(Some(cube_mesh_id), Some(cube_mesh_entity));
    ecs.attach_provisional_component(&test_cube_entity, test_cube_transform);
    ecs.attach_provisional_component(&test_cube_entity, test_cube_rigid_body);
    ecs.attach_provisional_component(&test_cube_entity, test_cube_material);
    ecs.attach_provisional_component(&test_cube_entity, test_cube_mesh_binding.clone());
    ecs.attach_provisional_component(&test_cube_entity, MousePickable {});

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

    let cursor_manager = CursorManager { is_locked: false, cursor_delta: VEC_2_ZERO };
    let cursor_manager_entity = ecs.create_entity();
    ecs.attach_provisional_component(&cursor_manager_entity, cursor_manager);

    ecs.register_system(SHUTDOWN_ECS, HashSet::from([ecs.get_system_signature_1::<VulkanRenderEngine>().unwrap()]), -999);
    ecs.register_system(TIME_SINCE_LAST_FRAME, HashSet::from([ecs.get_system_signature_1::<TimeDelta>().unwrap()]), -500);
    ecs.register_system(MANAGE_CURSOR, HashSet::from([ecs.get_system_signature_1::<VulkanRenderEngine>().unwrap(), ecs.get_system_signature_1::<CursorManager>().unwrap()]), -400);
    ecs.register_system(MOVE_CAMERA, HashSet::from([ecs.get_system_signature_1::<VulkanRenderEngine>().unwrap(), ecs.get_system_signature_1::<Viewport2D>().unwrap(), ecs.get_system_signature_1::<TimeDelta>().unwrap()]), -400);
    ecs.register_system(UPDATE_PARTICLES, HashSet::from([ecs.get_system_signature_2::<Transform, Particle>().unwrap(), ecs.get_system_signature_1::<TimeDelta>().unwrap()]), -200);
    ecs.register_system(UPDATE_RIGID_BODIES, HashSet::from([ecs.get_system_signature_2::<Transform, RigidBody>().unwrap(), ecs.get_system_signature_1::<TimeDelta>().unwrap()]), -200);
    ecs.register_system(UPDATE_QUAD_TREE, HashSet::from([ecs.get_system_signature_1::<QuadTree<BoundingSphere>>().unwrap(), ecs.get_system_signature_2::<Transform, RigidBody>().unwrap()]), -150);
    ecs.register_system(DETECT_PARTICLE_CABLE_COLLISIONS, HashSet::from([ecs.get_system_signature_1::<ParticleCable>().unwrap()]), -100);
    ecs.register_system(DETECT_PARTICLE_ROD_COLLISIONS, HashSet::from([ecs.get_system_signature_1::<ParticleRod>().unwrap()]), -100);
    ecs.register_system(DETECT_POTENTIAL_RIGID_BODY_COLLISIONS, HashSet::from([ecs.get_system_signature_1::<QuadTree<BoundingSphere>>().unwrap()]), -100);
    ecs.register_system(DETECT_RIGID_BODY_COLLISIONS, HashSet::from([ecs.get_system_signature_1::<PotentialRigidBodyCollision>().unwrap(), ecs.get_system_signature_1::<RigidBodyCollision>().unwrap()]), -99);
    ecs.register_system(RESOLVE_PARTICLE_COLLISIONS, HashSet::from([ecs.get_system_signature_1::<TimeDelta>().unwrap(), ecs.get_system_signature_1::<ParticleCollision>().unwrap()]), -50);
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

const MANAGE_CURSOR: System = |entites: Iter<Entity>, components: &ComponentManager, _: &mut ECSCommands| {
    let render_engine = entites.clone().find_map(|e| components.get_mut_component::<VulkanRenderEngine>(e)).unwrap();
    let cursor_manager = entites.clone().find_map(|e| components.get_mut_component::<CursorManager>(e)).unwrap();

    let esc_pressed = render_engine.is_key_pressed(VirtualKey::Escape);

    const DEBUG_CURSOR: bool = false;

    if let Ok(window) = render_engine.get_window_mut() {
        let rel_center_pos = vec2(window.get_width() as f32 / 2.0, window.get_height() as f32 / 2.0);
        let window_screen_pos = window.get_screen_position();

        let center_pos = window_screen_pos + rel_center_pos;

        if !cursor_manager.is_locked {
            if window.is_button_pressed(VirtualButton::Left) {
                window.set_mouse_screen_position(&center_pos).unwrap_or_default();

                cursor_manager.is_locked = true;

                window.set_mouse_cursor_visible(false || DEBUG_CURSOR).unwrap_or_default();
            }

            cursor_manager.cursor_delta = VEC_2_ZERO;
        } else if cursor_manager.is_locked && esc_pressed {
            cursor_manager.is_locked = false;

            window.set_mouse_cursor_visible(true).unwrap_or_default();

            cursor_manager.cursor_delta = VEC_2_ZERO;
        } else {
            if let Some(screen_pos) = window.get_mouse_screen_position() {
                cursor_manager.cursor_delta = *screen_pos - rel_center_pos;

                if cursor_manager.cursor_delta.len() > f32::EPSILON {
                    window.set_mouse_screen_position(&center_pos).unwrap_or_default();
                }
            } else {
                cursor_manager.cursor_delta = VEC_2_ZERO;
            }
        }
    } else {
        cursor_manager.is_locked = false;
        cursor_manager.cursor_delta = VEC_2_ZERO;
    }
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

    // if render_engine.is_key_pressed(VirtualKey::Escape) {
    //     commands.shutdown();
    // }
};

// const PICK_MESHES: System = |entites: Iter<Entity>, components: &ComponentManager, _: &mut ECSCommands| {
//     let render_engine = entites.clone().find_map(|e| components.get_component::<VulkanRenderEngine>(e)).unwrap();
//     let cam = &entites.clone().find_map(|e| components.get_component::<Viewport2D>(e)).unwrap().cam;

//     const FORCE_FACTOR: f32 = 100000.0;

//     if let Ok(window) = render_engine.get_window() {
//         if window.is_button_pressed(VirtualButton::Left) {
//             if let Some(screen_pos) = window.get_mouse_screen_position() {
//                 let ray = generate_ray(screen_pos, window, cam, NEAR_PLANE, FAR_PLANE);

//                 if let Ok(ray) = ray {
//                     for e in entites {
//                         if let Some(_) = components.get_component::<MousePickable>(e) {
//                             let transform = components.get_mut_component::<Transform>(e).unwrap();
//                             let mesh_binding = components.get_component::<MeshBinding>(e).unwrap();
//                             let rigid_body = components.get_mut_component::<RigidBody>(e).unwrap();

//                             let mesh = components.get_component::<Mesh>(&mesh_binding.mesh_wrapper.unwrap()).unwrap();

//                             // TODO: optimize this by first pruning with a simplified mesh, such as a bounding sphere
//                             //  Maybe formalize the MousePickable component into a built-in and add a bounding sphere field, then add a
//                             //  function to it to do all the intersection checks, pruning, etc...
//                             if let Some(intersection_point) = get_ray_intersection(&cam.pos, &ray, mesh, transform) {
//                                 rigid_body.add_force_at_point(&intersection_point, &(ray * FORCE_FACTOR), transform.get_pos());
//                             }
//                         }
//                     }
//                 }
//             }
//         }
//     }
// };

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

// CubeMeshOwner

struct CubeMeshOwner {}

impl Component for CubeMeshOwner {}
impl ComponentActions for CubeMeshOwner {}

// MousePickable

struct MousePickable {}

impl Component for MousePickable {}
impl ComponentActions for MousePickable {}

// CursorManager

struct CursorManager {
    is_locked: bool,
    cursor_delta: Vec2,
}

impl Component for CursorManager {}
impl ComponentActions for CursorManager {}
