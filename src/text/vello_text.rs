use crate::VelloFont;
use bevy::prelude::*;
use vello::peniko::Brush;

/// Describes how to position text from the origin
#[derive(Component, Default, Clone, Copy, PartialEq, Eq)]
pub enum VelloTextAlignment {
    /// Bounds start from the render position and advance up and to the right.
    #[default]
    BottomLeft,
    /// Bounds start from the render position and advance up.
    Bottom,
    /// Bounds start from the render position and advance up and to the left.
    BottomRight,

    /// Bounds start from the render position and advance right.
    Left,
    /// Bounds start from the render position and advance equally on both axes.
    Center,
    /// Bounds start from the render position and advance left.
    Right,

    /// Bounds start from the render position and advance down and to the right.
    TopLeft,
    /// Bounds start from the render position and advance down.
    Top,
    /// Bounds start from the render position and advance down and to the left.
    TopRight,
}

#[derive(Component, Default, Clone)]
pub struct VelloText {
    pub content: String,
    pub size: f32,
    pub brush: Option<Brush>,
}

impl VelloText {
    /// Returns the bounding box in world space
    pub fn bb_in_world_space(&self, font: &VelloFont, gtransform: &GlobalTransform) -> Rect {
        let size = font.sizeof(self);

        // Convert local coordinates to world coordinates
        let local_min = Vec3::new(0.0, 0.0, 0.0).extend(1.0);
        let local_max = Vec3::new(size.x, size.y, 0.0).extend(1.0);

        let min_world = gtransform.compute_matrix() * local_min;
        let max_world = gtransform.compute_matrix() * local_max;

        // Calculate the distance between the vertices to get the size in world space
        let min = Vec2::new(min_world.x, min_world.y);
        let max = Vec2::new(max_world.x, max_world.y);
        Rect { min, max }
    }

    /// Returns the bounding box in screen space
    pub fn bb_in_screen_space(
        &self,
        font: &VelloFont,
        gtransform: &GlobalTransform,
        camera: &Camera,
        camera_transform: &GlobalTransform,
    ) -> Option<Rect> {
        let Rect { min, max } = self.bb_in_world_space(font, gtransform);
        camera
            .viewport_to_world_2d(camera_transform, min)
            .zip(camera.viewport_to_world_2d(camera_transform, max))
            .map(|(min, max)| Rect { min, max })
    }
}
