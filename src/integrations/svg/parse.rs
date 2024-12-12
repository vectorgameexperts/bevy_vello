use super::asset::VelloSvg;
use crate::integrations::VectorLoaderError;
use bevy::transform::components::Transform;
use std::sync::Arc;
use vello_svg::usvg::{self};

/// Deserialize an SVG file from bytes.
pub fn load_svg_from_bytes(bytes: &[u8]) -> Result<VelloSvg, VectorLoaderError> {
    let svg_str = std::str::from_utf8(bytes)?;

    // Parse SVG
    let tree =
        usvg::Tree::from_str(svg_str, &usvg::Options::default()).map_err(vello_svg::Error::Svg)?;

    // Process the loaded SVG into Vello-compatible data
    let scene = vello_svg::render_tree(&tree);

    let width = tree.size().width();
    let height = tree.size().height();

    let asset = VelloSvg {
        scene: Arc::new(scene),
        local_transform_center: {
            let mut transform = Transform::default();
            transform.translation.x = width / 2.0;
            transform.translation.y = -height / 2.0;
            transform
        },
        width,
        height,
        alpha: 1.0,
    };

    Ok(asset)
}

/// Deserialize an SVG file from a string slice.
pub fn load_svg_from_str(svg_str: &str) -> Result<VelloSvg, VectorLoaderError> {
    let bytes = svg_str.as_bytes();

    load_svg_from_bytes(bytes)
}
