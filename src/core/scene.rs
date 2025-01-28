use crate::ecs::ECS;

use super::Viewport2D;

#[derive(Default)]
pub struct Scene {
    ecs: ECS,
    viewports: [Viewport2D],
}
