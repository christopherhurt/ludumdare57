use anyhow::Result;
use std::collections::HashMap;
use std::rc::Rc;
use uuid::Uuid;

use crate::math::{vec2, vec3, Vec2, Vec3, VEC_2_ZERO, VEC_3_Y_AXIS, VEC_3_ZERO, VEC_3_Z_AXIS};

/////////////////////////////////////////////////////////////////////////////
/// Common
/////////////////////////////////////////////////////////////////////////////

const DEFAULT_NODE_CAPACITY: usize = 64;

// Color

pub mod color {
    #[derive(Clone, Copy, Debug)]
    #[repr(C)]
    pub struct Color {
        pub r: f32,
        pub g: f32,
        pub b: f32,
        pub a: f32,
    }

    #[inline]
    pub const fn rgb(r: f32, g: f32, b: f32) -> Color {
        Color { r, g, b, a: 1.0 }
    }

    #[inline]
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color { r, g, b, a }
    }

    pub const RED: Color = rgb(1.0, 0.0, 0.0);
    pub const GREEN: Color = rgb(0.0, 1.0, 0.0);
    pub const BLUE: Color = rgb(0.0, 0.0, 1.0);
}

/////////////////////////////////////////////////////////////////////////////
/// Scene
/////////////////////////////////////////////////////////////////////////////

// Event

pub enum Event {
    Update,
}

// Signal

pub struct Signal {
    pub name: str,
}

// Camera

pub struct Camera {
    pub pos: Vec3,
    pub dir: Vec3,
    pub up: Vec3,
}

impl Camera {
    pub fn new(pos: Vec3, dir: Vec3, up: Vec3) -> Self {
        Self { pos, dir, up }
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            pos: VEC_3_ZERO,
            dir: VEC_3_Z_AXIS,
            up: VEC_3_Y_AXIS,
        }
    }
}

// Viewport2D

pub struct Viewport2D {
    pub cam: Camera,
    pub offset: Vec2,
    pub scale: Vec2,
}

impl Viewport2D {
    pub fn new(cam: Camera, offset: Vec2, scale: Vec2) -> Self {
        Self { cam, offset, scale }
    }
}

impl Default for Viewport2D {
    fn default() -> Self {
        Self {
            cam: Camera::default(),
            offset: VEC_2_ZERO,
            scale: vec2(1.0, 1.0),
        }
    }
}

// Scene

pub struct Scene {
    root_nodes: Vec<Node>,
    viewports: Vec<Viewport2D>,
}

impl Scene {
    pub fn new(root_nodes: Vec<Node>, viewports: Vec<Viewport2D>) -> Self {
        Self { root_nodes, viewports }
    }

    pub fn add_root_node(&mut self, node: Node) {
        self.root_nodes.push(node);
    }

    pub fn remove_node(id: Uuid) -> Result<()> {
        todo!() // TODO
    }

    pub fn get_node(&self, id: Uuid) -> Result<&Node> {
        todo!() // TODO
    }

    pub fn get_mut_node(&mut self, id: Uuid) -> Result<&mut Node> {
        todo!() // TODO
    }

    pub fn get_nodes_by_tag(&mut self, tag: &String) -> Vec<&Node> {
        // TODO
    }

    pub fn fire_event(&mut self, evt: &Event) {
        // TODO
        self.root_nodes.iter().for_each(|n| self.handle_event(evt, n));
    }

    fn handle_event(&mut self, evt: &Event, node: &Node) {
        match node.handle_event {
            Some(f) => f(self, evt),
            _ => {},
        }

        // TODO iter over children
    }

    pub fn fire_signal(&mut self, signal: &Signal, tags: &[&str]) {
        // TODO
    }

    fn handle_signal(&mut self, signal: &Signal, tags: &[&str], node: &Node) {
        // TODO
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            root_nodes: Vec::with_capacity(DEFAULT_NODE_CAPACITY),
            viewports: vec![Viewport2D::default()],
        }
    }
}

/////////////////////////////////////////////////////////////////////////////
/// Node
/////////////////////////////////////////////////////////////////////////////

// Node

pub struct Node {
    uuid: Uuid,
    tags: Box<[String]>,
    children: Vec<Node>,

    handle_event: Option<fn(&mut Scene, &Event)>,
    handle_signal: Option<fn(&mut Scene, &Signal)>,

    pub transform: Option<Transform>,
    pub mesh: Option<Rc<Mesh>>,
    pub color_material: Option<Rc<ColorMaterial>>,
}

impl Node {
    pub fn new(
        tags: Option<Box<[String]>>,
        children: Option<Vec<Node>>,
        handle_event: Option<fn(&mut Scene, &Event)>,
        handle_signal: Option<fn(&mut Scene, &Signal)>,
        transform: Option<Transform>,
        mesh: Option<Rc<Mesh>>,
        color_material: Option<Rc<ColorMaterial>>,
    ) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            tags: tags.unwrap_or(Box::new([])).into(),
            children: children.unwrap_or(Vec::with_capacity(DEFAULT_NODE_CAPACITY)),
            handle_event,
            handle_signal,
            transform,
            mesh,
            color_material,
        }
    }

    pub fn get_id(&self) -> Uuid {
        self.uuid
    }

    pub fn get_tags(&self) -> &[String] {
        self.tags.as_ref()
    }

    pub fn get_children(&self) -> &Vec<Node> {
        &self.children
    }

    pub fn get_children_mut(&mut self) -> &mut Vec<Node> {
        &mut self.children
    }
}

impl Default for Node {
    fn default() -> Self {
        Self {
            uuid: Uuid::new_v4(),
            tags: Box::new([String::default(); 0]),
            children: Vec::with_capacity(DEFAULT_NODE_CAPACITY),
            handle_event: None,
            handle_signal: None,
            transform: None,
            mesh: None,
            color_material: None,
        }
    }
}

// Transform

pub struct Transform {
    pub pos: Vec3,
    pub rot: Vec3,
    pub scl: Vec3,
}

impl Transform {
    pub fn new(pos: Vec3, rot: Vec3, scl: Vec3) -> Self {
        Self { pos, rot, scl }
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            pos: VEC_3_ZERO,
            rot: VEC_3_ZERO,
            scl: vec3(1.0, 1.0, 1.0),
        }
    }
}

// Mesh

pub struct Mesh {
    pub id: u32,
}

// ColorMaterial

pub struct ColorMaterial {
    pub color: color::Color,
}

impl Default for ColorMaterial {
    fn default() -> Self {
        Self {
            color: color::RED,
        }
    }
}
