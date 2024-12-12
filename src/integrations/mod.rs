//! Integrations for supported file types. These are included by cargo feature.
//!
//! # Features
//! - `svg` - Enables SVG loading and rendering
//! - `lottie` - Enable Lottie (JSON) loading and rendering
//! - `experimental-dotLottie` - Enables experimental support for dotLottie interactivity. WIP.

#[cfg(feature = "svg")]
pub mod svg;

#[cfg(feature = "lottie")]
pub mod lottie;

#[cfg(feature = "experimental-dotLottie")]
pub mod dot_lottie;

mod error;
pub use error::VectorLoaderError;

#[cfg(any(feature = "svg", feature = "lottie"))]
use bevy::prelude::*;

#[cfg(any(feature = "svg", feature = "lottie"))]
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

#[cfg(any(feature = "svg", feature = "lottie"))]
impl VelloAssetAnchor {
    pub(crate) fn compute(
        &self,
        width: f32,
        height: f32,
        transform: &GlobalTransform,
    ) -> GlobalTransform {
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
