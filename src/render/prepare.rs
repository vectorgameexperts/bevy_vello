use super::extract::{ExtractedPixelScale, ExtractedRenderScene, ExtractedRenderText};
use crate::CoordinateSpace;
use bevy::{
    prelude::*,
    render::{
        camera::ExtractedCamera,
        view::{ExtractedView, RenderLayers},
    },
};
use vello::kurbo::Affine;

#[derive(Component, Copy, Clone, Deref, DerefMut)]
pub struct PreparedAffine(pub Affine);

#[derive(Component, Copy, Clone, Deref, DerefMut)]
pub struct PreparedTransform(pub GlobalTransform);

// All extracted bevy_vello render instance types should implement this (RenderAsset, RenderScene,
// RenderText, etc...)
pub trait PrepareRenderInstance {
    fn final_transform(&self) -> PreparedTransform;
    fn scene_affine(
        &self,
        view: &ExtractedView,
        world_transform: GlobalTransform,
        pixel_scale: f32,
        viewport_size: UVec2,
    ) -> PreparedAffine;
}

pub fn prepare_scene_affines(
    mut commands: Commands,
    views: Query<(&ExtractedCamera, &ExtractedView, Option<&RenderLayers>), With<Camera2d>>,
    render_entities: Query<(Entity, &ExtractedRenderScene)>,
    pixel_scale: Res<ExtractedPixelScale>,
) {
    for (camera, view, maybe_camera_layers) in views.iter() {
        let camera_render_layers = maybe_camera_layers.unwrap_or_default();
        let size_pixels: UVec2 = camera.physical_viewport_size.unwrap();
        let (pixels_x, pixels_y) = (size_pixels.x as f32, size_pixels.y as f32);

        // Render scenes
        for (entity, render_entity) in render_entities.iter() {
            let maybe_entity_layers = render_entity.render_layers.clone();
            let entity_render_layers = maybe_entity_layers.unwrap_or_default();
            if !camera_render_layers.intersects(&entity_render_layers) {
                continue;
            }

            let ndc_to_pixels_matrix = Mat4::from_cols_array_2d(&[
                [pixels_x / 2.0, 0.0, 0.0, pixels_x / 2.0],
                [0.0, pixels_y / 2.0, 0.0, pixels_y / 2.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ])
            .transpose();

            let world_transform = render_entity.transform;

            let raw_transform = match render_entity.render_mode {
                CoordinateSpace::ScreenSpace => {
                    let mut model_matrix =
                        world_transform.compute_matrix().mul_scalar(pixel_scale.0);

                    if let Some(node) = &render_entity.ui_node {
                        // The Bevy Transform for a UI node seems to always have the origin
                        // of the translation at the center of its bounding box. Here we
                        // move the origin back to the top left, so that, e.g., drawing a
                        // shape with center=(20,20) inside of a 40x40 UI node results in
                        // the shape being centered within the node.
                        let Vec2 { x, y } = node.size() * pixel_scale.0;
                        model_matrix.w_axis.x -= x / 2.0;
                        model_matrix.w_axis.y -= y / 2.0;

                        // Note that there's no need to flip the Y axis in this case, as
                        // Bevy handles it for us.
                    } else {
                        model_matrix.w_axis.y *= -1.0;
                    }

                    model_matrix
                }
                CoordinateSpace::WorldSpace => {
                    let mut model_matrix = world_transform.compute_matrix();
                    model_matrix.w_axis.y *= -1.0;

                    let (projection_mat, view_mat) = {
                        let mut view_mat = view.world_from_view.compute_matrix();
                        view_mat.w_axis.y *= -1.0;

                        (view.clip_from_view, view_mat)
                    };

                    let view_proj_matrix = projection_mat * view_mat.inverse();

                    ndc_to_pixels_matrix * view_proj_matrix * model_matrix
                }
            };

            let transform: [f32; 16] = raw_transform.to_cols_array();

            // | a c e |
            // | b d f |
            // | 0 0 1 |
            let transform: [f64; 6] = [
                transform[0] as f64,  // a
                -transform[1] as f64, // b
                -transform[4] as f64, // c
                transform[5] as f64,  // d
                transform[12] as f64, // e
                transform[13] as f64, // f
            ];

            commands
                .entity(entity)
                .insert(PreparedAffine(Affine::new(transform)));
        }
    }
}

pub fn prepare_text_affines(
    mut commands: Commands,
    views: Query<(&ExtractedCamera, &ExtractedView, Option<&RenderLayers>), With<Camera2d>>,
    render_entities: Query<(Entity, &ExtractedRenderText)>,
    pixel_scale: Res<ExtractedPixelScale>,
) {
    for (camera, view, maybe_camera_layers) in views.iter() {
        let camera_render_layers = maybe_camera_layers.unwrap_or_default();
        let size_pixels: UVec2 = camera.physical_viewport_size.unwrap();
        let (pixels_x, pixels_y) = (size_pixels.x as f32, size_pixels.y as f32);

        for (entity, render_entity) in render_entities.iter() {
            let maybe_entity_layers = render_entity.render_layers.clone();
            let entity_render_layers = maybe_entity_layers.unwrap_or_default();
            if !camera_render_layers.intersects(&entity_render_layers) {
                continue;
            }

            let ndc_to_pixels_matrix = Mat4::from_cols_array_2d(&[
                [pixels_x / 2.0, 0.0, 0.0, pixels_x / 2.0],
                [0.0, pixels_y / 2.0, 0.0, pixels_y / 2.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ])
            .transpose();

            let world_transform = render_entity.transform;

            let mut model_matrix = world_transform.compute_matrix();
            model_matrix.w_axis.y *= -1.0;

            let (projection_mat, view_mat) = {
                let mut view_mat = view.world_from_view.compute_matrix();
                view_mat.w_axis.y *= -1.0;

                (view.clip_from_view, view_mat)
            };

            let view_proj_matrix = projection_mat * view_mat.inverse();
            let vello_matrix = ndc_to_pixels_matrix * view_proj_matrix;

            let raw_transform = match render_entity.render_space {
                CoordinateSpace::ScreenSpace => {
                    world_transform.compute_matrix().mul_scalar(pixel_scale.0)
                }
                CoordinateSpace::WorldSpace => vello_matrix * model_matrix,
            };

            let transform: [f32; 16] = raw_transform.to_cols_array();

            // | a c e |
            // | b d f |
            // | 0 0 1 |
            let transform: [f64; 6] = [
                transform[0] as f64,  // a
                -transform[1] as f64, // b
                -transform[4] as f64, // c
                transform[5] as f64,  // d
                transform[12] as f64, // e
                transform[13] as f64, // f
            ];

            commands
                .entity(entity)
                .insert(PreparedAffine(Affine::new(transform)));
        }
    }
}
