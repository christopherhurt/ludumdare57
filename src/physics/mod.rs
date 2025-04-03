use anyhow::{anyhow, Result};
use core::f32;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

use crate::core::{Camera, Transform};
use crate::core::mesh::{Edge, Face, Mesh, Vertex};
use crate::ecs::{ComponentActions, ProvisionalEntity};
use crate::ecs::component::Component;
use crate::ecs::entity::Entity;
use crate::math::{get_proj_matrix, mat3, vec3, vec4, Mat3, Quat, Vec2, Vec3, VEC_3_ZERO};
use crate::render_engine::Window;

// Common

pub(in crate) fn apply_ang_vel(rot: &Quat, ang_vel: &Vec3, delta: f32) -> Quat {
    if let Ok(ang_vel_norm) = ang_vel.normalized() {
        let to_apply = Quat::from_axis_spin(&ang_vel_norm, ang_vel.len() * delta)
            .unwrap_or_else(|_| panic!("Internal error: failed to get quaternion for normalized axis"));

        (*rot * to_apply).normalized()
    } else {
        *rot
    }
}

pub fn local_to_world_point(local_point: &Vec3, transform: &mut Transform) -> Vec3 {
    (*transform.to_world_mat() * local_point.to_vec4(1.0)).xyz()
}

pub fn local_to_world_force(local_force: &Vec3, transform: &mut Transform) -> Vec3 {
    transform.to_rot_mat().to_mat3() * *local_force
}

// Particle

#[derive(Clone, Debug)]
pub struct Particle {
    pub vel: Vec3,
    pub acc: Vec3,
    pub damping: f32,
    pub mass: f32,
    pub gravity: f32,
    pub force_accum: Vec3,
    // TODO: add material which can be used to derive the coefficient of restitution - for now, it's being generated randomly for each collision
}

impl Particle {
    pub fn new(vel: Vec3, damping: f32, mass: f32, gravity: f32) -> Self {
        Self {
            vel,
            acc: VEC_3_ZERO,
            damping,
            mass,
            gravity,
            force_accum: VEC_3_ZERO,
        }
    }
}

impl Default for Particle {
    fn default() -> Self {
        Self {
            vel: VEC_3_ZERO,
            acc: VEC_3_ZERO,
            damping: 1.0,
            mass: 1.0,
            gravity: 10.0,
            force_accum: VEC_3_ZERO,
        }
    }
}

impl Component for Particle {}
impl ComponentActions for Particle {}

// ParticleCable

#[derive(Clone, Debug)]
pub struct ParticleCable {
    pub particle_a: Entity,
    pub particle_b: Entity,
    pub particle_a_prov: ProvisionalEntity,
    pub particle_b_prov: ProvisionalEntity,
    pub max_length: f32,
    pub restitution: f32,
}

impl ParticleCable {
    pub fn new(
        particle_a: Entity,
        particle_b: Entity,
        max_length: f32,
        restitution: f32,
    ) -> Self {
        Self {
            particle_a,
            particle_b,
            particle_a_prov: ProvisionalEntity(0),
            particle_b_prov: ProvisionalEntity(0),
            max_length,
            restitution,
        }
    }

    pub fn new_provisional(
        particle_a: ProvisionalEntity,
        particle_b: ProvisionalEntity,
        max_length: f32,
        restitution: f32,
    ) -> Self {
        Self {
            particle_a: Entity(0),
            particle_b: Entity(0),
            particle_a_prov: particle_a,
            particle_b_prov: particle_b,
            max_length,
            restitution,
        }
    }
}

impl Component for ParticleCable {}

impl ComponentActions for ParticleCable {
    fn update_provisional_entities(&mut self, provisional_to_entities: &HashMap<ProvisionalEntity, Entity>) {
        self.particle_a = provisional_to_entities.get(&self.particle_a_prov).unwrap_or_else(|| panic!("Failed to map provisional entity {:?}", &self.particle_a_prov)).clone();
        self.particle_b = provisional_to_entities.get(&self.particle_b_prov).unwrap_or_else(|| panic!("Failed to map provisional entity {:?}", &self.particle_b_prov)).clone();
    }
}

// ParticleRod

#[derive(Clone, Debug)]
pub struct ParticleRod {
    pub particle_a: Entity,
    pub particle_b: Entity,
    pub particle_a_prov: ProvisionalEntity,
    pub particle_b_prov: ProvisionalEntity,
    pub length: f32,
}

impl ParticleRod {
    pub fn new(
        particle_a: Entity,
        particle_b: Entity,
        length: f32,
    ) -> Self {
        Self {
            particle_a,
            particle_b,
            particle_a_prov: ProvisionalEntity(0),
            particle_b_prov: ProvisionalEntity(0),
            length,
        }
    }

    pub fn new_provisional(
        particle_a: ProvisionalEntity,
        particle_b: ProvisionalEntity,
        length: f32,
    ) -> Self {
        Self {
            particle_a: Entity(0),
            particle_b: Entity(0),
            particle_a_prov: particle_a,
            particle_b_prov: particle_b,
            length,
        }
    }
}

impl Component for ParticleRod {}

impl ComponentActions for ParticleRod {
    fn update_provisional_entities(&mut self, provisional_to_entities: &HashMap<ProvisionalEntity, Entity>) {
        self.particle_a = provisional_to_entities.get(&self.particle_a_prov).unwrap_or_else(|| panic!("Failed to map provisional entity {:?}", &self.particle_a_prov)).clone();
        self.particle_b = provisional_to_entities.get(&self.particle_b_prov).unwrap_or_else(|| panic!("Failed to map provisional entity {:?}", &self.particle_b_prov)).clone();
    }
}

// ParticleCollision

#[derive(Clone, Debug)]
pub(in crate) struct ParticleCollision {
    pub(in crate) particle_a: Entity,
    pub(in crate) particle_b: Option<Entity>, // None indicates particle_2 has infinite mass, i.e. immovable
    pub(in crate) restitution: f32,
    pub(in crate) normal: Vec3,
    pub(in crate) penetration: f32,
}

impl ParticleCollision {
    pub(in crate) fn new(
        particle_a: Entity,
        particle_b: Option<Entity>,
        restitution: f32,
        normal: Vec3,
        penetration: f32,
    ) -> Self {
        Self {
            particle_a,
            particle_b,
            restitution,
            normal,
            penetration,
        }
    }
}

impl Component for ParticleCollision {}
impl ComponentActions for ParticleCollision {}

// ParticleCollisionDetector

#[derive(Clone, Debug)]
pub struct ParticleCollisionDetector {
    pub default_restitution: f32,
}

impl ParticleCollisionDetector {
    pub fn new(default_restitution: f32) -> Self {
        Self { default_restitution }
    }
}

impl Component for ParticleCollisionDetector {}
impl ComponentActions for ParticleCollisionDetector {}

// RigidBody

#[derive(Clone, Debug)]
pub struct RigidBody {
    pub linear_vel: Vec3,
    pub ang_vel: Vec3,
    pub linear_acc: Vec3,
    pub ang_acc: Vec3,
    pub linear_damping: f32,
    pub ang_damping: f32,
    pub gravity: f32,
    pub props: PhysicsMeshProperties, // TODO: prob don't need to store this with every rigid body, but rather alongside the mesh
    pub(in crate) linear_force_accum: Vec3,
    pub(in crate) torque_accum: Vec3,
}

impl RigidBody {
    pub fn new(linear_vel: Vec3, ang_vel: Vec3, linear_damping: f32, ang_damping: f32, gravity: f32, props: PhysicsMeshProperties) -> Self {
        Self {
            linear_vel,
            ang_vel,
            linear_acc: VEC_3_ZERO,
            ang_acc: VEC_3_ZERO,
            linear_damping,
            ang_damping,
            gravity,
            props,
            linear_force_accum: VEC_3_ZERO,
            torque_accum: VEC_3_ZERO,
        }
    }

    pub fn add_linear_force(&mut self, force: &Vec3) {
        self.linear_force_accum += *force;
    }

    pub fn add_torque(&mut self, point: &Vec3, force: &Vec3, rigid_body_pos: &Vec3) {
        self.torque_accum += (*point - *rigid_body_pos).cross(force);
    }

    pub fn add_force_at_point(&mut self, point: &Vec3, force: &Vec3, rigid_body_pos: &Vec3) {
        self.add_linear_force(force);
        self.add_torque(point, force, rigid_body_pos);
    }
}

impl Component for RigidBody {}
impl ComponentActions for RigidBody {}

#[derive(Clone, Debug)]
pub struct PhysicsMeshProperties { // TODO: refactor this alongside the mesh/mesh binding changes
    pub volume: f32, // TODO: remove for now? it's not accurate without scale values
    pub mass: Option<f32>, // None implies the mesh is immovable, i.e. infinite mass
    pub inertia_tensor: Option<Mat3>, // None iff mass is None
    pub center_of_mass_offset: Vec3,
    pub bounding_radius: f32,
}

impl PhysicsMeshProperties {
    pub fn new(volume: f32, mass: f32, inertia_tensor: Mat3, center_of_mass_offset: Vec3, bounding_radius: f32) -> Self {
        Self {
            volume,
            mass: Some(mass),
            inertia_tensor: Some(inertia_tensor),
            center_of_mass_offset,
            bounding_radius,
        }
    }

    pub fn new_immovable(volume: f32, center_of_mass_offset: Vec3, bounding_radius: f32) -> Self {
        Self {
            volume,
            mass: None,
            inertia_tensor: None,
            center_of_mass_offset,
            bounding_radius,
        }
    }
}

impl Component for PhysicsMeshProperties {}
impl ComponentActions for PhysicsMeshProperties {}

pub fn generate_physics_mesh(mesh: Mesh, density: Option<f32>) -> Result<(Mesh, PhysicsMeshProperties)> {
    // https://github.com/blackedout01/simkn/blob/main/simkn.h

    // None density implies infinite mass
    if density.is_some_and(|d| d <= 0.0) {
        return Err(anyhow!("Density must be positive"));
    }

    let mut mass = 0.0;
    let mut center_of_mass = VEC_3_ZERO;
    let mut volume = 0.0;
    let mut i_a = 0.0;
    let mut i_b = 0.0;
    let mut i_c = 0.0;
    let mut i_ap = 0.0;
    let mut i_bp = 0.0;
    let mut i_cp = 0.0;

    for i in mesh.vertex_indices.chunks(3) {
        if i.len() != 3 {
            return Err(anyhow!("Mesh is not triangulated"));
        }

        let v0 = &mesh.vertices[i[0] as usize].pos;
        let v1 = &mesh.vertices[i[1] as usize].pos;
        let v2 = &mesh.vertices[i[2] as usize].pos;

        let det = v2.cross(v1).dot(v0);

        let tetrahedron_signed_volume = det / 6.0;

        if let Some(density) = density {
            let tetrahedron_signed_mass = density * tetrahedron_signed_volume;
            let tetrahedron_center_off_mass = (*v0 + *v1 + *v2) / 4.0;

            let tetrahedron_moment_x = calculate_inertia_moment(v0.x, v1.x, v2.x);
            let tetrahedron_moment_y = calculate_inertia_moment(v0.y, v1.y, v2.y);
            let tetrahedron_moment_z = calculate_inertia_moment(v0.z, v1.z, v2.z);

            let tetrahedron_product_yz = calculate_inertia_product(v0.y, v1.y, v2.y, v0.z, v1.z, v2.z);
            let tetrahedron_product_xy = calculate_inertia_product(v0.x, v1.x, v2.x, v0.y, v1.y, v2.y);
            let tetrahedron_product_xz = calculate_inertia_product(v0.x, v1.x, v2.x, v0.z, v1.z, v2.z);

            i_a += det * (tetrahedron_moment_y + tetrahedron_moment_z);
            i_b += det * (tetrahedron_moment_x + tetrahedron_moment_z);
            i_c += det * (tetrahedron_moment_x + tetrahedron_moment_y);

            i_ap += det * tetrahedron_product_yz;
            i_bp += det * tetrahedron_product_xy;
            i_cp += det * tetrahedron_product_xz;

            mass += tetrahedron_signed_mass;
            center_of_mass += tetrahedron_center_off_mass * tetrahedron_signed_mass;
        }

        volume += tetrahedron_signed_volume;
    }

    if density.is_some_and(|_| mass <= 0.0) {
        return Err(anyhow!("Mesh mass was computed as non-positive - consider reversing your triangle winding order"));
    }

    let (mass, inertia_tensor) = if let Some(density) = density {
        center_of_mass /= mass;

        i_a = density * i_a / 60.0 - mass * (center_of_mass.y * center_of_mass.y + center_of_mass.z * center_of_mass.z);
        i_b = density * i_b / 60.0 - mass * (center_of_mass.x * center_of_mass.x + center_of_mass.z * center_of_mass.z);
        i_c = density * i_c / 60.0 - mass * (center_of_mass.x * center_of_mass.x + center_of_mass.y * center_of_mass.y);

        i_ap = density * i_ap / 120.0 - mass * (center_of_mass.y * center_of_mass.z);
        i_bp = density * i_bp / 120.0 - mass * (center_of_mass.x * center_of_mass.y);
        i_cp = density * i_cp / 120.0 - mass * (center_of_mass.x * center_of_mass.z);

        let inertia_tensor = mat3(
            i_a, -i_bp, -i_cp,
            -i_bp, i_b, -i_ap,
            -i_cp, -i_ap, i_c,
        );

        (Some(mass), Some(inertia_tensor))
    } else {
        (None, None)
    };

    let offseted_vertices = mesh.vertices.iter().map(|v| Vertex {
        pos: v.pos - center_of_mass,
        norm: v.norm,
    }).collect();

    let new_mesh = Mesh::new(offseted_vertices, mesh.vertex_indices.to_vec())
        .unwrap_or_else(|_| panic!("Internal error: an invalid mesh was constructed"));

    let bounding_radius = mesh.vertices.iter()
        .map(|v| v.pos.len())
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Less))
        .unwrap_or_else(|| panic!("Internal error: mesh has no vertices"));

    let properties = PhysicsMeshProperties {
        volume,
        mass,
        inertia_tensor,
        center_of_mass_offset: center_of_mass,
        bounding_radius,
    };

    Ok((new_mesh, properties))
}

fn calculate_inertia_moment(v0: f32, v1: f32, v2: f32) -> f32 {
    v0 * v0 + v1 * v2
    + v1 * v1 + v0 * v2
    + v2 * v2 + v0 * v1
}

fn calculate_inertia_product(
    v00: f32, v01: f32, v02: f32,
    v10: f32, v11: f32, v12: f32,
) -> f32 {
    2.0 * v00 * v10 + v01 * v12 + v02 * v11
    + 2.0 * v01 * v11 + v00 * v12 + v02 * v10
    + 2.0 * v02 * v12 + v00 * v11 + v01 * v10
}

pub fn generate_ray(screen_coords: &Vec2, window: &impl Window, cam: &Camera, near_plane: f32, far_plane: f32) -> Result<Vec3> {
    let aspect_ratio = window.get_width() as f32 / window.get_height() as f32;

    let inverse_ndc_to_screen_space = window.get_ndc_to_screen_space_transform().inverted()
        .unwrap_or_else(|_| panic!("Internal error: NDC to screen space transform is not invertible"));
    let inverse_proj_matrix = get_proj_matrix(near_plane, far_plane, cam.fov_rads, aspect_ratio)?
        .inverted()
        .unwrap_or_else(|_| panic!("Internal error: projection matrix is not invertible"));
    let inverse_view_matrix = cam.to_view_mat()?
        .inverted()
        .unwrap_or_else(|_| panic!("Internal error: view matrix is not invertible"));

    let ndc_coords = inverse_ndc_to_screen_space * screen_coords.to_vec3(1.0);
    let clip_coords = vec4(ndc_coords.x, ndc_coords.y, 1.0, 1.0);
    let view_coords = (inverse_proj_matrix * clip_coords).xy().to_vec4(1.0, 0.0);
    let world_coords = (inverse_view_matrix * view_coords).xyz().normalized().unwrap_or_else(|_| panic!("Internal error: ray is length zero"));

    Ok(world_coords)
}

pub fn get_ray_intersection(ray_source: &Vec3, ray_dir: &Vec3, mesh: &Mesh, transform: &mut Transform) -> Option<Vec3> {
    // https://courses.cs.washington.edu/courses/csep557/09sp/lectures/triangle_intersection.pdf

    let world_matrix = transform.to_world_mat();
    let inverted_world_matrix = world_matrix.inverted()
        .unwrap_or_else(|_| panic!("Internal error: failed to invert world matrix"));
    let inverse_rot_matrix = transform.to_rot_mat().to_mat3().inverted()
        .unwrap_or_else(|_| panic!("Internal error: failed to invert rotation matrix"));

    let ray_source = (inverted_world_matrix * ray_source.to_vec4(1.0)).xyz();
    let ray_dir = inverse_rot_matrix * *ray_dir;

    let mut closest_intersection_point=  None;
    let mut closest_intersection_dist = f32::INFINITY;

    if let Ok(ray_dir) = ray_dir.normalized() {
        for i in mesh.vertex_indices.chunks_exact(3) {
            let p0 = &mesh.vertices[i[0] as usize].pos;
            let p1 = &mesh.vertices[i[1] as usize].pos;
            let p2 = &mesh.vertices[i[2] as usize].pos;

            if let Ok(n) = (*p0 - *p1).cross(&(*p2 - *p1)).normalized() {
                let n_dot_dir = n.dot(&ray_dir);

                // Ignore rays running parallel to or intersecting through the "underside" of the triangle
                if n_dot_dir < -f32::EPSILON {
                    let intersection_dist = (n.dot(&p0) - n.dot(&ray_source)) / n_dot_dir;
                    let intersection_point = ray_source + intersection_dist * ray_dir;

                    if is_inside_triangle(p0, p1, p2, &intersection_point, &n, 0.0)
                            && intersection_dist < closest_intersection_dist {
                        closest_intersection_point = Some((*transform.to_world_mat() * intersection_point.to_vec4(1.0)).xyz());
                        closest_intersection_dist = intersection_dist;
                    }
                }
            }
        }
    }

    closest_intersection_point
}

fn is_inside_triangle(p0: &Vec3, p1: &Vec3, p2: &Vec3, q: &Vec3, n: &Vec3, tolerance: f32) -> bool {
    is_inside_edge(p0, p2, q, n, tolerance)
        && is_inside_edge(p1, p0, q, n, tolerance)
        && is_inside_edge(p2, p1, q, n, tolerance)
}

fn is_inside_edge(a: &Vec3, b: &Vec3, q: &Vec3, n: &Vec3, tolerance: f32) -> bool {
    (*b - *a).cross(&(*q - *a)).dot(n) >= tolerance
}

// Coarse collision detection

pub trait BoundingVolume: 'static + Clone {
    fn overlaps_with(&self, other: &Self) -> bool;
    fn get_extent(&self) -> (Vec3, Vec3);
}

#[derive(Clone, Debug)]
pub struct BoundingSphere {
    pub center: Vec3,
    pub radius: f32,
}

impl BoundingSphere {
    pub fn from_transform(transform: &Transform, local_bounding_radius: f32) -> Self {
        let scl = transform.get_scl();

        Self {
            center: *transform.get_pos(),
            radius: local_bounding_radius * scl.x.max(scl.y).max(scl.z),
        }
    }
}

impl BoundingVolume for BoundingSphere {
    fn overlaps_with(&self, other: &BoundingSphere) -> bool {
        (other.center - self.center).len() <= self.radius + other.radius
    }

    fn get_extent(&self) -> (Vec3, Vec3) {
        let radius_extent = vec3(self.radius, self.radius, self.radius);

        let min_extent = self.center - radius_extent;
        let max_extent = self.center + radius_extent;

        (min_extent, max_extent)
    }
}

// TODO: impl AxisAlignedBoundingBox

#[derive(Clone, Debug)]
pub struct PotentialRigidBodyCollision {
    pub entity_a: Entity,
    pub entity_b: Entity,
}

impl PotentialRigidBodyCollision {
    fn new(entity_a: Entity, entity_b: Entity) -> Self {
        Self { entity_a, entity_b }
    }
}

impl Component for PotentialRigidBodyCollision {}
impl ComponentActions for PotentialRigidBodyCollision {}

#[derive(Clone, Debug)]
struct QuadTreeNode<T: BoundingVolume> {
    pos: Vec3, // y value is ignored
    half_length: f32,
    children: Option<Box<[QuadTreeNode<T>; 4]>>, // In clockwise order from -x, -z quadrant
    entities_in_quad: HashSet<Entity>,
    bounding_volumes: HashMap<Entity, T>, // Assert empty if children is Some
    max_depth: usize,
    max_bounding_volumes_per_node: usize,
    min_child_bounding_volumes_per_node: usize,
}

impl<T: BoundingVolume> QuadTreeNode<T> {
    fn new(
        pos: Vec3,
        half_length: f32,
        max_depth: usize,
        max_bounding_volumes_per_node: usize,
        min_child_bounding_volumes_per_node: usize,
    ) -> Self {
        Self {
            pos,
            half_length,
            children: None,
            entities_in_quad: HashSet::with_capacity(max_bounding_volumes_per_node),
            bounding_volumes: HashMap::with_capacity(max_bounding_volumes_per_node),
            max_depth,
            max_bounding_volumes_per_node,
            min_child_bounding_volumes_per_node,
        }
    }

    fn insert(
        &mut self,
        entity: &Entity,
        bounding_volume: &T,
        current_depth: usize,
    ) {
        if self.overlaps_with(bounding_volume) {
            self.entities_in_quad.insert(*entity);

            if let Some(children) = self.children.as_mut() {
                for c in children.as_mut() {
                    c.insert(entity, bounding_volume, current_depth + 1);
                }
            } else if current_depth >= self.max_depth || self.bounding_volumes.len() < self.max_bounding_volumes_per_node {
                self.bounding_volumes.insert(*entity, bounding_volume.clone());
            } else {
                let child_half_length = self.half_length / 2.0;

                let mut child_nodes: [QuadTreeNode<T>; 4] = [
                    QuadTreeNode::new(
                        vec3(self.pos.x - child_half_length, self.pos.y, self.pos.z - child_half_length),
                        child_half_length,
                        self.max_depth,
                        self.max_bounding_volumes_per_node,
                        self.min_child_bounding_volumes_per_node,
                    ),
                    QuadTreeNode::new(
                        vec3(self.pos.x - child_half_length, self.pos.y, self.pos.z + child_half_length),
                        child_half_length,
                        self.max_depth,
                        self.max_bounding_volumes_per_node,
                        self.min_child_bounding_volumes_per_node,
                    ),
                    QuadTreeNode::new(
                        vec3(self.pos.x + child_half_length, self.pos.y, self.pos.z + child_half_length),
                        child_half_length,
                        self.max_depth,
                        self.max_bounding_volumes_per_node,
                        self.min_child_bounding_volumes_per_node,
                    ),
                    QuadTreeNode::new(
                        vec3(self.pos.x + child_half_length, self.pos.y, self.pos.z - child_half_length),
                        child_half_length,
                        self.max_depth,
                        self.max_bounding_volumes_per_node,
                        self.min_child_bounding_volumes_per_node,
                    ),
                ];

                for c in child_nodes.iter_mut() {
                    for (e, v) in self.bounding_volumes.iter() {
                        c.insert(e, v, current_depth + 1);
                    }

                    c.insert(entity, bounding_volume, current_depth + 1);
                }

                self.children = Some(Box::new(child_nodes));
                self.bounding_volumes.clear();
            }
        }
    }

    fn remove(
        &mut self,
        entity: &Entity,
    ) {
        if self.entities_in_quad.contains(entity) {
            self.entities_in_quad.remove(entity);

            if let Some(children) = self.children.as_mut() {
                for c in children.iter_mut() {
                    c.remove(entity);
                }
            } else {
                self.bounding_volumes.remove(entity);
            }

            if self.entities_in_quad.len() < self.min_child_bounding_volumes_per_node {
                if let Some(children) = self.children.take() {
                    for c in children.into_iter() {
                        self.bounding_volumes.extend(c.bounding_volumes.into_iter());
                    }
                }
            }
        }
    }

    fn get_potential_collisions(
        &self,
        potential_collisions: &mut Vec<PotentialRigidBodyCollision>,
    ) {
        if let Some(children) = self.children.as_ref() {
            for c in children.as_ref() {
                c.get_potential_collisions(potential_collisions);
            }
        } else {
            for (i, (e_a, v_a)) in self.bounding_volumes.iter().enumerate() {
                for (e_b, v_b) in self.bounding_volumes.iter().skip(i + 1) {
                    if v_a.overlaps_with(v_b) {
                        potential_collisions.push(PotentialRigidBodyCollision::new(*e_a, *e_b));
                    }
                }
            }
        }
    }

    fn get_potential_collisions_with(
        &self,
        entity: &Entity,
        bounding_volume: &T,
        potential_collisions: &mut Vec<PotentialRigidBodyCollision>,
    ) {
        if self.overlaps_with(bounding_volume) {
            if let Some(children) = self.children.as_ref() {
                for c in children.as_ref() {
                    c.get_potential_collisions_with(entity, bounding_volume, potential_collisions);
                }
            } else {
                for (e, v) in &self.bounding_volumes {
                    if v.overlaps_with(bounding_volume) {
                        potential_collisions.push(PotentialRigidBodyCollision::new(*entity, *e));
                    }
                }
            }
        }
    }

    fn overlaps_with(&self, bounding_volume: &T) -> bool {
        let (min_extent, max_extent) = bounding_volume.get_extent();

        let min_node_x = self.pos.x - self.half_length;
        let max_node_x = self.pos.x + self.half_length;
        let min_node_z = self.pos.z - self.half_length;
        let max_node_z = self.pos.z + self.half_length;

        min_extent.x <= max_node_x && max_extent.x >= min_node_x
            && min_extent.z <= max_node_z && max_extent.z >= min_node_z
    }
}

#[derive(Clone, Debug)]
pub struct QuadTree<T: BoundingVolume> {
    root_node: QuadTreeNode<T>,
    all_entities: HashSet<Entity>,
}

impl<T: BoundingVolume> QuadTree<T> {
    pub fn new(
        origin: Vec3,
        initial_level_half_length: f32,
        initial_entity_capacity: usize,
        max_depth: usize,
        max_bounding_volumes_per_node: usize,
        min_child_bounding_volumes_per_node: usize,
    ) -> Result<Self> {
        if min_child_bounding_volumes_per_node > max_bounding_volumes_per_node + 1 {
            return Err(anyhow!("min_child_bounding_volumes_per_node may only exceed max_bounding_volumes_per_node by at most 1"));
        }

        Ok(
            Self {
                root_node: QuadTreeNode::new(
                    origin,
                    initial_level_half_length,
                    max_depth,
                    max_bounding_volumes_per_node,
                    min_child_bounding_volumes_per_node,
                ),
                all_entities: HashSet::with_capacity(initial_entity_capacity),
            }
        )
    }

    pub fn insert(
        &mut self,
        entity: Entity,
        bounding_volume: T
    ) -> Result<()> {
        if self.all_entities.contains(&entity) {
            return Err(anyhow!("Quad tree already contains entity {:?}", &entity));
        }

        // TODO: expand the level as needed, and consider shrinking the level on removal as well
        if self.is_outside_level_bounds(&bounding_volume) {
            return Err(anyhow!("Bounding volume is outside of the level bounds"));
        }

        self.root_node.insert(&entity, &bounding_volume, 0);

        self.all_entities.insert(entity);

        Ok(())
    }

    pub fn remove(&mut self, entity: &Entity) -> Result<()> {
        if !self.all_entities.contains(&entity) {
            return Err(anyhow!("Quad tree does not contain entity {:?}", &entity));
        }

        self.root_node.remove(entity);

        self.all_entities.remove(entity);

        Ok(())
    }

    pub fn remove_not_in(&mut self, entities: &HashSet<&Entity>) {
        let to_remove = self.root_node.entities_in_quad
            .iter()
            .filter(|e| !entities.contains(e))
            .map(|e| *e)
            .collect::<HashSet<_>>();

        for e in to_remove {
            self.all_entities.remove(&e);

            self.root_node.remove(&e);
        }
    }

    pub fn get_potential_collisions(&self) -> Vec<PotentialRigidBodyCollision> {
        let mut potential_collisions = Vec::new();

        self.root_node.get_potential_collisions(&mut potential_collisions);

        potential_collisions
    }

    pub fn get_potential_collisions_with(
        &self,
        entity: &Entity,
        bounding_volume: &T,
    ) -> Vec<PotentialRigidBodyCollision> {
        let mut potential_collisions = Vec::new();

        self.root_node.get_potential_collisions_with(entity, bounding_volume, &mut potential_collisions);

        potential_collisions
    }

    fn is_outside_level_bounds(&self, bounding_volume: &T) -> bool {
        let (min_extent, max_extent) = bounding_volume.get_extent();

        let min_level_x = self.root_node.pos.x - self.root_node.half_length;
        let max_level_x = self.root_node.pos.x + self.root_node.half_length;
        let min_level_z = self.root_node.pos.z - self.root_node.half_length;
        let max_level_z = self.root_node.pos.z + self.root_node.half_length;

        min_extent.x <= min_level_x || max_extent.x >= max_level_x
            || min_extent.z <= min_level_z || max_extent.z >= max_level_z
    }
}

impl<T: BoundingVolume> Component for QuadTree<T> {}
impl<T: BoundingVolume> ComponentActions for QuadTree<T> {}

// Rigid body collision detection

// (Index of point vertex, indices of face vertices)
pub type PointCollisionFeatures = (u32, Face);
pub type EdgeCollisionFeatures = (Edge, Edge);

#[derive(Clone, Debug)]
pub struct RigidBodyCollision {
    pub rigid_body_a: Entity,
    pub rigid_body_b: Entity,
    pub point: Vec3,
    pub normal: Vec3,
    pub penetration: f32,

    pub point_features: Option<PointCollisionFeatures>, // Always a point, b face
    pub edge_features: Option<EdgeCollisionFeatures>, // Always a edge, then b edge
}

impl RigidBodyCollision {
    pub fn new(
        rigid_body_a: Entity,
        rigid_body_b: Entity,
        point: Vec3,
        normal: Vec3,
        penetration: f32,
    ) -> Self {
        Self {
            rigid_body_a,
            rigid_body_b,
            point,
            normal,
            penetration,

            point_features: None,
            edge_features: None,
        }
    }
}

impl PartialEq for RigidBodyCollision {
    fn eq(&self, other: &Self) -> bool {
        self.rigid_body_a == other.rigid_body_a
            && self.rigid_body_b == other.rigid_body_b
            && self.point_features == other.point_features
            && self.edge_features == other.edge_features
    }
}

impl Eq for RigidBodyCollision {}

impl Hash for RigidBodyCollision {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.rigid_body_a.hash(state);
        self.rigid_body_b.hash(state);
        self.point_features.hash(state);
        self.edge_features.hash(state);
    }
}

impl Component for RigidBodyCollision {}
impl ComponentActions for RigidBodyCollision {}

pub fn get_deepest_rigid_body_collision(
    mesh_a: (&Entity, &Mesh),
    mesh_b: (&Entity, &Mesh),
    transform_a: &mut Transform,
    transform_b: &mut Transform,
) -> Option<RigidBodyCollision> {
    // TODO: optimize with GJK or another non-naive approach
    // TODO: also, this is probably not rigorous for non-convex polyhedra

    let a_to_b_space = transform_b.to_world_mat().inverted().unwrap() * *transform_a.to_world_mat();
    let b_to_a_space = transform_a.to_world_mat().inverted().unwrap() * *transform_b.to_world_mat();

    let mut result: Option<RigidBodyCollision> = None;
    let mut max_penetration = f32::MIN;

    // Every vertex of mesh_a with faces of mesh_b
    for (i, vertex_a) in mesh_a.1.vertices.iter().enumerate() {
        let vertex_pos = (a_to_b_space * vertex_a.pos.to_vec4(1.0)).xyz();

        if let Some(shallowest) = get_shallowest_point_collision(&vertex_pos, (mesh_a.0, i as u32), mesh_b, transform_b) {
            if shallowest.penetration > max_penetration {
                max_penetration = shallowest.penetration;
                result = Some(shallowest);
            }
        }
    }

    // Every vertex of mesh_b with faces of mesh_a
    for (i, vertex_b) in mesh_b.1.vertices.iter().enumerate() {
        let vertex_pos = (b_to_a_space * vertex_b.pos.to_vec4(1.0)).xyz();

        if let Some(shallowest) = get_shallowest_point_collision(&vertex_pos, (mesh_b.0, i as u32), mesh_a, transform_a) {
            if shallowest.penetration > max_penetration {
                max_penetration = shallowest.penetration;
                result = Some(shallowest);
            }
        }
    }

    // Every edge of mesh_a with edges of mesh_b
    // for edge_a in mesh_a.1.edges.iter() {
    //     let vertex_pos_0 = (*a_to_b_space * mesh_a.1.vertices[edge_a.0 as usize].pos.to_vec4(1.0)).xyz();
    //     let vertex_pos_1 = (*a_to_b_space * mesh_a.1.vertices[edge_a.1 as usize].pos.to_vec4(1.0)).xyz();

    //     if let Some(shallowest) = get_shallowest_edge_collision((&vertex_pos_0, &vertex_pos_1), (mesh_a.0, edge_a), mesh_b) {
    //         if shallowest.penetration > max_penetration {
    //             max_penetration = shallowest.penetration;
    //             result = Some(shallowest);
    //         }
    //     }
    // }

    // Every edge of mesh_b with edges of mesh_a
    // for edge_b in mesh_b.1.edges.iter() {
    //     let vertex_pos_0 = (*b_to_a_space * mesh_b.1.vertices[edge_b.0 as usize].pos.to_vec4(1.0)).xyz();
    //     let vertex_pos_1 = (*b_to_a_space * mesh_b.1.vertices[edge_b.1 as usize].pos.to_vec4(1.0)).xyz();

    //     if let Some(shallowest) = get_shallowest_edge_collision((&vertex_pos_0, &vertex_pos_1), (mesh_b.0, edge_b), mesh_a) {
    //         if shallowest.penetration > max_penetration {
    //             max_penetration = shallowest.penetration;
    //             result = Some(shallowest);
    //         }
    //     }
    // }

    result
}

fn get_shallowest_point_collision(
    vertex: &Vec3,
    vertex_index: (&Entity, u32),
    mesh: (&Entity, &Mesh),
    mesh_transform: &mut Transform,
) -> Option<RigidBodyCollision> {
    mesh.1.vertex_indices
        .chunks(3)
        .map(|i| {
            let f_0 = &mesh.1.vertices[i[0] as usize].pos;
            let f_1 = &mesh.1.vertices[i[1] as usize].pos;
            let f_2 = &mesh.1.vertices[i[2] as usize].pos;

            get_point_collision_local_to_face(vertex_index.0, mesh.0, vertex, (f_0, f_1, f_2), 0.0).map(|mut c| {
                let face: Face = (i[0], i[1], i[2]);

                c.point_features = Some((vertex_index.1, face));

                c
            })
        })
        .filter(|c| c.is_some())
        .map(|c| c.unwrap())
        .map(|mut c| {
            c.point = (*mesh_transform.to_world_mat() * c.point.to_vec4(1.0)).xyz();

            let scaled_normal = c.normal * c.penetration;
            let world_normal = mesh_transform.to_rot_mat().to_mat3() * mesh_transform.to_scl_mat().to_mat3() * scaled_normal;

            c.normal = world_normal.normalized().unwrap();
            c.penetration = world_normal.len();

            c
        })
        .min_by(|a, b| a.penetration.partial_cmp(&b.penetration).unwrap_or(Ordering::Less))
}

pub(in crate) fn get_point_collision_local_to_face(
    entity_a: &Entity,
    entity_b: &Entity,
    vertex_a: &Vec3,
    face_b: (&Vec3, &Vec3, &Vec3),
    tolerance: f32,
) -> Option<RigidBodyCollision> {
    let (p0, p1, p2) = face_b;
    let p3 = &VEC_3_ZERO;

    // Face 0 - p0, p1, p2
    if let Ok(n0) = (*p2 - *p0).cross(&(*p1 - *p0)).normalized() {
        // Face 1 - p3, p1, p0
        if let Ok(n1) = p0.cross(p1).normalized() {
            // Face 2 - p3, p2, p1
            if let Ok(n2) = p1.cross(p2).normalized() {
                // Face 3 - p3, p0, p2
                if let Ok(n3) = p2.cross(p0).normalized() {
                    if is_inside_triangle(p0, p1, p2, vertex_a, &n0, tolerance)
                            && is_inside_triangle(p3, p1, p0, vertex_a, &n1, tolerance)
                            && is_inside_triangle(p3, p2, p1, vertex_a, &n2, tolerance)
                            && is_inside_triangle(p3, p0, p2, vertex_a, &n3, tolerance) {
                        let dist_to_plane = vertex_a.dot(&n0);

                        return Some(
                            RigidBodyCollision::new(
                                *entity_a,
                                *entity_b,
                                *vertex_a + n0 * (dist_to_plane / 2.0),
                                n0,
                                dist_to_plane,
                            )
                        );
                    }
                }
            }
        }
    }

    None
}

fn _get_shallowest_edge_collision(edge: (&Vec3, &Vec3), edge_indices: (&Entity, &Edge), mesh: (&Entity, &Mesh)) -> Option<RigidBodyCollision> {
    mesh.1.edges
        .iter()
        .map(|e| {
            let edge_b = (&mesh.1.vertices[e.0 as usize].pos, &mesh.1.vertices[e.1 as usize].pos);

            // TODO: need in world space
            get_edge_collision_local_to_b(edge_indices.0, mesh.0, edge, edge_b, 0.0).map(|mut c| {
                c.edge_features = Some((*edge_indices.1, *e));

                c
            })
        })
        .filter(|c| c.is_some())
        .map(|c| c.unwrap())
        .min_by(|a, b| a.penetration.partial_cmp(&b.penetration).unwrap_or(Ordering::Less))
}

pub(in crate) fn get_edge_collision_local_to_b(
    _entity_a: &Entity,
    _entity_b: &Entity,
    _edge_a: (&Vec3, &Vec3),
    _edge_b: (&Vec3, &Vec3),
    _tolerance: f32,
) -> Option<RigidBodyCollision> {
    // TODO: implement me! and uncomment out above

    None
}
