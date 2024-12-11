//! Logic for rendering debug visualizations
use crate::{
    text::VelloTextAnchor, CoordinateSpace, VelloAsset, VelloAssetAnchor, VelloAssetHandle,
    VelloFont, VelloTextSection,
};
use bevy::{color::palettes::css, math::Vec3Swizzles, prelude::*};

const RED_X_SIZE: f32 = 8.0;

pub struct DebugVisualizationsPlugin;

impl Plugin for DebugVisualizationsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (render_asset_debug, render_text_debug));
    }
}

#[derive(Clone, Copy, Component, Default, PartialEq, Reflect)]
#[reflect(Component)]
pub enum DebugVisualizations {
    #[default]
    Hidden,
    Visible,
}

/// A system to render debug visualizations for `VelloAsset`.
fn render_asset_debug(
    query_vectors: Query<
        (
            &VelloAssetHandle,
            &VelloAssetAnchor,
            &GlobalTransform,
            &CoordinateSpace,
            &DebugVisualizations,
        ),
        Without<Node>,
    >,
    assets: Res<Assets<VelloAsset>>,
    query_cam: Query<(&Camera, &GlobalTransform, &OrthographicProjection), With<Camera2d>>,
    mut gizmos: Gizmos,
) {
    let Ok((camera, view, projection)) = query_cam.get_single() else {
        return;
    };

    // Show vectors
    for (asset, asset_anchor, gtransform, space, _) in query_vectors
        .iter()
        .filter(|(_, _, _, _, d)| **d == DebugVisualizations::Visible)
    {
        if let Some(asset) = assets.get(asset.id()) {
            match space {
                CoordinateSpace::WorldSpace => {
                    // Origin
                    let origin = gtransform.translation().xy();
                    draw_origin(&mut gizmos, projection, origin);
                    // Bounding box
                    let gtransform = &asset_anchor.compute(asset, gtransform);
                    let rect_center = gtransform.translation().xy();
                    let rect = asset.bb_in_world_space(gtransform);
                    draw_bounding_box(&mut gizmos, rect_center, rect.size());
                }
                CoordinateSpace::ScreenSpace => {
                    // Origin
                    let origin = gtransform.translation().xy();
                    let Ok(origin) = camera.viewport_to_world_2d(view, origin) else {
                        continue;
                    };
                    draw_origin(&mut gizmos, projection, origin);
                    // Bounding box
                    let gtransform = &asset_anchor.compute(asset, gtransform);
                    let rect_center = gtransform.translation().xy();
                    let Ok(rect_center) = camera.viewport_to_world_2d(view, rect_center) else {
                        continue;
                    };
                    let Some(rect) = asset.bb_in_screen_space(gtransform, camera, view) else {
                        continue;
                    };
                    draw_bounding_box(&mut gizmos, rect_center, rect.size());
                }
            }
        }
    }
}

/// A system to render debug visualizations for `VelloText`.
fn render_text_debug(
    query_world: Query<
        (
            &VelloTextSection,
            &VelloTextAnchor,
            &GlobalTransform,
            &CoordinateSpace,
            &DebugVisualizations,
        ),
        Without<Node>,
    >,
    query_cam: Query<(&Camera, &GlobalTransform, &OrthographicProjection), With<Camera2d>>,
    fonts: Res<Assets<VelloFont>>,
    mut gizmos: Gizmos,
) {
    let Ok((camera, view, projection)) = query_cam.get_single() else {
        return;
    };

    // Show world-space vectors
    for (text, text_anchor, gtransform, space, _) in query_world
        .iter()
        .filter(|(_, _, _, _, d)| **d == DebugVisualizations::Visible)
    {
        if let Some(font) = fonts.get(text.style.font.id()) {
            let rect = text.bb_in_world_space(font, gtransform);
            let mut origin = gtransform.translation().xy();
            match space {
                CoordinateSpace::WorldSpace => {
                    draw_origin(&mut gizmos, projection, origin);
                    let size = rect.size();
                    let (width, height) = size.into();
                    match text_anchor {
                        VelloTextAnchor::BottomLeft => {}
                        VelloTextAnchor::Bottom => {
                            origin.x += -width / 2.0;
                        }
                        VelloTextAnchor::BottomRight => {
                            origin.x += -width;
                        }
                        VelloTextAnchor::TopLeft => {
                            origin.y += -height;
                        }
                        VelloTextAnchor::Left => {
                            origin.y += -height / 2.0;
                        }
                        VelloTextAnchor::Top => {
                            origin.x += -width / 2.0;
                            origin.y += -height;
                        }
                        VelloTextAnchor::Center => {
                            origin.x += -width / 2.0;
                            origin.y += -height / 2.0;
                        }
                        VelloTextAnchor::TopRight => {
                            origin.x += -width;
                            origin.y += -height;
                        }
                        VelloTextAnchor::Right => {
                            origin.x += -width;
                            origin.y += -height / 2.0;
                        }
                    };
                    let rect_center = origin + rect.size() / 2.0;
                    gizmos.rect_2d(
                        Isometry2d::new(rect_center, Rot2::degrees(0.0)),
                        rect.size(),
                        css::WHITE,
                    );
                }
                CoordinateSpace::ScreenSpace => {
                    let Some(rect) = text.bb_in_screen_space(font, gtransform, camera, view) else {
                        continue;
                    };
                    let Ok(mut origin) =
                        camera.viewport_to_world_2d(view, gtransform.translation().xy())
                    else {
                        continue;
                    };
                    draw_origin(&mut gizmos, projection, origin);
                    let size = rect.size();
                    let (width, height) = size.into();
                    match text_anchor {
                        VelloTextAnchor::BottomLeft => {}
                        VelloTextAnchor::Bottom => {
                            origin.x += -width / 2.0;
                        }
                        VelloTextAnchor::BottomRight => {
                            origin.x += -width;
                        }
                        VelloTextAnchor::TopLeft => {
                            origin.y += height;
                        }
                        VelloTextAnchor::Left => {
                            origin.y += height / 2.0;
                        }
                        VelloTextAnchor::Top => {
                            origin.x += -width / 2.0;
                            origin.y += height;
                        }
                        VelloTextAnchor::Center => {
                            origin.x += -width / 2.0;
                            origin.y += height / 2.0;
                        }
                        VelloTextAnchor::TopRight => {
                            origin.x += -width;
                            origin.y += height;
                        }
                        VelloTextAnchor::Right => {
                            origin.x += -width;
                            origin.y += height / 2.0;
                        }
                    };
                    let rect_center = origin + Vec2::new(rect.width() / 2.0, -rect.height() / 2.0);
                    gizmos.rect_2d(
                        Isometry2d::new(rect_center, Rot2::degrees(0.0)),
                        rect.size() * Vec2::new(1.0, 1.0),
                        css::WHITE,
                    );
                }
            }
        }
    }
}

/// A helper method to draw text gizmos.
fn draw_origin(gizmos: &mut Gizmos, projection: &OrthographicProjection, origin: Vec2) {
    let from = origin + RED_X_SIZE * Vec2::splat(1.0) * projection.scale;
    let to = origin + RED_X_SIZE * Vec2::splat(-1.0) * projection.scale;

    gizmos.line_2d(from, to, css::RED);

    let from = origin + RED_X_SIZE * Vec2::new(1.0, -1.0) * projection.scale;
    let to = origin + RED_X_SIZE * Vec2::new(-1.0, 1.0) * projection.scale;

    gizmos.line_2d(from, to, css::RED);
}

/// A helper method to draw the bounding box
fn draw_bounding_box(gizmos: &mut Gizmos, position: Vec2, size: Vec2) {
    gizmos.rect_2d(
        Isometry2d::new(position, Rot2::degrees(0.0)),
        size,
        css::WHITE,
    );
}
