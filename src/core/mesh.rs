use anyhow::{anyhow, Result};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;

use crate::ecs::{ComponentActions, ProvisionalEntity};
use crate::ecs::component::Component;
use crate::ecs::entity::Entity;
use crate::math::{vec3, Vec3, VEC_3_ZERO};

// MeshBinding

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RenderMeshId(pub(in crate) usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MeshBinding { // TODO: clean up this whole mesh binding structure, and cleanup physics mesh bindings, properties, etc.
    pub mesh_wrapper: Option<Entity>,
    pub id: Option<RenderMeshId>,
    provisional_mesh_wrapper: Option<ProvisionalEntity>,
}

impl MeshBinding {
    pub fn new(id: Option<RenderMeshId>, mesh_wrapper: Option<Entity>) -> Self {
        Self {
            id,
            mesh_wrapper,
            provisional_mesh_wrapper: None,
        }
    }

    pub fn new_provisional(id: Option<RenderMeshId>, provisional_mesh_wrapper: Option<ProvisionalEntity>) -> Self {
        Self {
            id,
            mesh_wrapper: None,
            provisional_mesh_wrapper
        }
    }
}

impl Component for MeshBinding {}

impl ComponentActions for MeshBinding {
    fn update_provisional_entities(&mut self, provisional_to_entities: &HashMap<ProvisionalEntity, Entity>) {
        if let Some(p) = self.provisional_mesh_wrapper.take() {
            self.mesh_wrapper = Some(
                provisional_to_entities.get(&p).unwrap_or_else(|| panic!("Failed to map provisional entity {:?}", &p)).clone()
            );
        }
    }
}

// Vertex

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct Vertex {
    pub pos: Vec3,
    pub norm: Vec3,
}

// Mesh

pub struct Mesh {
    pub vertices: Arc<Vec<Vertex>>,
    pub vertex_indices: Arc<Vec<u32>>,
}

impl Mesh {
    pub fn new(
        vertices: Vec<Vertex>,
        vertex_indices: Vec<u32>,
    ) -> Self {
        Self {
            vertices: Arc::new(vertices),
            vertex_indices: Arc::new(vertex_indices),
        }
    }
}

impl Component for Mesh {}
impl ComponentActions for Mesh {}

pub fn load_obj_mesh(file_path: &str, normalize_positions: bool) -> Result<Mesh> {
    let mut reader = BufReader::new(File::open(file_path)?);

    let (models, _) = tobj::load_obj_buf(
        &mut reader,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
        |_| Ok(Default::default()),
    )?;

    let mut vertices = Vec::new();
    let mut vertex_indices = Vec::new();

    let mut source_indices_to_my_indices = HashMap::new();

    for model in &models {
        if model.mesh.indices.len() % 3 != 0 {
            return Err(anyhow!("Mesh is not triangulated"));
        }

        let has_normals = !model.mesh.normals.is_empty();
        let mut triangle_my_indexes = [0 as usize; 3];

        for (i, source_index) in model.mesh.indices.iter().enumerate() {
            let source_index = *source_index as usize;
            let vec_3_offset = 3 * source_index;

            let pos = vec3(
                model.mesh.positions[vec_3_offset],
                model.mesh.positions[vec_3_offset + 1],
                model.mesh.positions[vec_3_offset + 2],
            );

            let norm = if has_normals {
                vec3(
                    model.mesh.normals[vec_3_offset],
                    model.mesh.normals[vec_3_offset + 1],
                    model.mesh.normals[vec_3_offset + 2],
                )
            } else {
                VEC_3_ZERO
            };

            if let Some(my_index) = source_indices_to_my_indices.get(&source_index) {
                vertex_indices.push(*my_index as u32);
            } else {
                let my_index = vertices.len();
                source_indices_to_my_indices.insert(source_index, my_index);

                vertices.push(Vertex { pos, norm });
                vertex_indices.push(my_index as u32);
            }

            let triangle_index = i % 3;
            triangle_my_indexes[triangle_index] = *vertex_indices.last().unwrap() as usize;

            if !has_normals && triangle_index == 2 {
                let edge_0 = vertices[triangle_my_indexes[0]].pos - vertices[triangle_my_indexes[1]].pos;
                let edge_1 = vertices[triangle_my_indexes[2]].pos - vertices[triangle_my_indexes[1]].pos;

                let computed_normal = edge_0.cross(&edge_1).normalized().unwrap_or(VEC_3_ZERO);

                vertices[triangle_my_indexes[0]].norm += computed_normal;
                vertices[triangle_my_indexes[1]].norm += computed_normal;
                vertices[triangle_my_indexes[2]].norm += computed_normal;
            }
        }
    }

    if vertex_indices.is_empty() {
        return Err(anyhow!("File {:?} contains no vertices", file_path));
    }

    let normalization_factor = if normalize_positions {
        let y_comp = |a: &f32, b: &f32| a.partial_cmp(b).unwrap_or(Ordering::Less);

        let min_y = vertices.iter().map(|v| v.pos.y).min_by(y_comp).unwrap_or_else(|| panic!("Internal error: vertices is empty"));
        let max_y = vertices.iter().map(|v| v.pos.y).max_by(y_comp).unwrap_or_else(|| panic!("Internal error: vertices is empty"));

        Some(1.0 / (max_y - min_y))
    } else {
        None
    };

    for v in vertices.iter_mut() {
        if let Some(normalization_factor) = normalization_factor {
            (*v).pos *= normalization_factor;
        }

        (*v).norm = v.norm.normalized().unwrap_or(VEC_3_ZERO);
    }

    Ok(Mesh::new(vertices, vertex_indices))
}
