use anyhow::{anyhow, Result};
use std::collections::HashMap;

use crate::core::Transform;
use crate::core::mesh::{Mesh, Vertex};
use crate::ecs::{ComponentActions, ProvisionalEntity};
use crate::ecs::component::Component;
use crate::ecs::entity::Entity;
use crate::math::{mat3, Mat3, quat, Quat, Vec3, VEC_3_ZERO};

// Common

pub(in crate) fn apply_ang_vel(rot: &Quat, ang_vel: &Vec3, delta: f32) -> Quat {
    let mut result = rot.clone();
    let mut to_apply = quat(0.0, ang_vel.x, ang_vel.y, ang_vel.z);

    to_apply *= *rot;

    result.w += to_apply.w * 0.5 * delta;
    result.i += to_apply.i * 0.5 * delta;
    result.j += to_apply.j * 0.5 * delta;
    result.k += to_apply.k * 0.5 * delta;

    result.normalized()
}

pub fn local_to_world_point(local_point: &Vec3, transform: &Transform) -> Vec3 {
    (transform.to_world_mat() * local_point.to_vec4(1.0)).to_vec3()
}

pub fn local_to_world_force(local_force: &Vec3, transform: &Transform) -> Vec3 {
    transform.rot.to_rotation_matrix().to_mat3() * *local_force
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
    pub props: PhysicsMeshProperties,
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

    pub fn add_force_at_point(&mut self, point: &Vec3, force: &Vec3, rigid_body_pos: &Vec3) {
        self.linear_force_accum += *force;
        self.torque_accum += (*point - *rigid_body_pos).cross(force);
    }
}

impl Component for RigidBody {}
impl ComponentActions for RigidBody {}

#[derive(Clone, Debug)]
pub struct PhysicsMeshProperties {
    pub volume: f32,
    pub mass: f32,
    pub inertia_tensor: Mat3,
    pub center_of_mass_offset: Vec3,
}

impl Component for PhysicsMeshProperties {}
impl ComponentActions for PhysicsMeshProperties {}

pub fn generate_physics_mesh(mesh: Mesh, density: f32) -> Result<(Mesh, PhysicsMeshProperties)> {
    // https://github.com/blackedout01/simkn/blob/main/simkn.h

    if density <= 0.0 {
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
        volume += tetrahedron_signed_volume;
    }

    if mass <= 0.0 {
        return Err(anyhow!("Mesh mass was computed as non-positive - consider reversing your triangle winding order"));
    }

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

    let offseted_vertices = mesh.vertices.iter().map(|v| Vertex {
        pos: v.pos - center_of_mass,
        norm: v.norm,
    }).collect();

    let new_mesh = Mesh::new(offseted_vertices, mesh.vertex_indices.to_vec());

    let properties = PhysicsMeshProperties {
        volume,
        mass,
        inertia_tensor,
        center_of_mass_offset: center_of_mass,
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
