use crate::VectorFile;
use bevy::{prelude::*, reflect::TypePath};

#[derive(Component, Default, Debug, Clone, Deref, DerefMut, PartialEq, Eq)]
pub struct VelloAssetHandle(pub Handle<VelloAsset>);

#[derive(Asset, TypePath, Clone)]
pub struct VelloAsset {
    pub file: VectorFile,
    pub local_transform_center: Transform,
    pub width: f32,
    pub height: f32,
    pub alpha: f32,
}

impl VelloAsset {
    /// Returns the bounding box in world space
    pub fn bb_in_world_space(&self, gtransform: &GlobalTransform) -> Rect {
        // Convert local coordinates to world coordinates
        let local_min = Vec3::new(-self.width / 2.0, -self.height / 2.0, 0.0).extend(1.0);
        let local_max = Vec3::new(self.width / 2.0, self.height / 2.0, 0.0).extend(1.0);

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
        gtransform: &GlobalTransform,
        camera: &Camera,
        camera_transform: &GlobalTransform,
    ) -> Option<Rect> {
        let Rect { min, max } = self.bb_in_world_space(gtransform);
        camera
            .viewport_to_world_2d(camera_transform, min)
            .ok()
            .zip(camera.viewport_to_world_2d(camera_transform, max).ok())
            .map(|(min, max)| Rect { min, max })
    }
}

/// Describes how the asset is positioned relative to its [`Transform`]. It defaults to [`VelloAssetAnchor::Center`].
#[derive(Component, Default, Clone, Copy, PartialEq, Eq)]
pub enum VelloAssetAnchor {
    /// Bounds start from the render position and advance up and to the right.
    BottomLeft,
    /// Bounds start from the render position and advance up.
    Bottom,
    /// Bounds start from the render position and advance up and to the left.
    BottomRight,

    /// Bounds start from the render position and advance right.
    Left,
    /// Bounds start from the render position and advance equally on both axes.
    #[default]
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

impl VelloAssetAnchor {
    pub(crate) fn compute(
        &self,
        asset: &VelloAsset,
        transform: &GlobalTransform,
    ) -> GlobalTransform {
        let (width, height) = (asset.width, asset.height);
        // Apply positioning
        let adjustment = match self {
            VelloAssetAnchor::TopLeft => Vec3::new(width / 2.0, -height / 2.0, 0.0),
            VelloAssetAnchor::Left => Vec3::new(width / 2.0, 0.0, 0.0),
            VelloAssetAnchor::BottomLeft => Vec3::new(width / 2.0, height / 2.0, 0.0),
            VelloAssetAnchor::Top => Vec3::new(0.0, -height / 2.0, 0.0),
            VelloAssetAnchor::Center => Vec3::new(0.0, 0.0, 0.0),
            VelloAssetAnchor::Bottom => Vec3::new(0.0, height / 2.0, 0.0),
            VelloAssetAnchor::TopRight => Vec3::new(-width / 2.0, -height / 2.0, 0.0),
            VelloAssetAnchor::Right => Vec3::new(-width / 2.0, 0.0, 0.0),
            VelloAssetAnchor::BottomRight => Vec3::new(-width / 2.0, height / 2.0, 0.0),
        };
        let new_translation: Vec3 = (transform.compute_matrix() * adjustment.extend(1.0)).xyz();
        GlobalTransform::from(
            transform
                .compute_transform()
                .with_translation(new_translation),
        )
    }
}
