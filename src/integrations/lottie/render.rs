use super::{
    asset::{VelloLottie, VelloLottieHandle},
    Playhead, Theme,
};
use crate::{
    render::{
        extract::ExtractedPixelScale,
        prepare::{PrepareRenderInstance, PreparedAffine, PreparedTransform},
    },
    CoordinateSpace, SkipEncoding, VelloAssetAnchor,
};
use bevy::{
    prelude::*,
    render::{
        camera::ExtractedCamera,
        view::{ExtractedView, RenderLayers},
        Extract,
    },
};
use vello::kurbo::Affine;

#[derive(Component, Clone)]
pub struct ExtractedLottieAsset {
    pub asset: VelloLottie,
    pub asset_anchor: VelloAssetAnchor,
    pub transform: GlobalTransform,
    pub render_mode: CoordinateSpace,
    pub ui_node: Option<ComputedNode>,
    pub render_layers: Option<RenderLayers>,
    pub alpha: f32,
    pub theme: Option<Theme>,
    pub playhead: f64,
}

pub fn extract_lottie_assets(
    mut commands: Commands,
    query_vectors: Extract<
        Query<
            (
                &VelloLottieHandle,
                &VelloAssetAnchor,
                &CoordinateSpace,
                &GlobalTransform,
                &Playhead,
                Option<&Theme>,
                Option<&ComputedNode>,
                Option<&RenderLayers>,
                &ViewVisibility,
                &InheritedVisibility,
            ),
            Without<SkipEncoding>,
        >,
    >,
    assets: Extract<Res<Assets<VelloLottie>>>,
) {
    for (
        asset,
        asset_anchor,
        coord_space,
        transform,
        playhead,
        theme,
        ui_node,
        render_layers,
        view_visibility,
        inherited_visibility,
    ) in query_vectors.iter()
    {
        if let Some(asset) = assets.get(asset.id()) {
            if view_visibility.get() && inherited_visibility.get() {
                let playhead = playhead.frame();
                commands.spawn(ExtractedLottieAsset {
                    asset: asset.to_owned(),
                    transform: *transform,
                    asset_anchor: *asset_anchor,
                    theme: theme.cloned(),
                    render_mode: *coord_space,
                    playhead,
                    alpha: asset.alpha,
                    ui_node: ui_node.cloned(),
                    render_layers: render_layers.cloned(),
                });
            }
        }
    }
}

pub fn prepare_asset_affines(
    mut commands: Commands,
    views: Query<(&ExtractedCamera, &ExtractedView, Option<&RenderLayers>), With<Camera2d>>,
    mut render_entities: Query<(Entity, &ExtractedLottieAsset)>,
    pixel_scale: Res<ExtractedPixelScale>,
) {
    for (camera, view, maybe_camera_layers) in views.iter() {
        let camera_render_layers = maybe_camera_layers.unwrap_or_default();
        let viewport_size: UVec2 = camera.physical_viewport_size.unwrap();
        for (entity, render_entity) in render_entities.iter_mut() {
            let maybe_entity_layers = render_entity.render_layers.clone();
            let entity_render_layers = maybe_entity_layers.unwrap_or_default();
            if !camera_render_layers.intersects(&entity_render_layers) {
                continue;
            }

            // Prepare render data needed for the subsequent render system
            let final_transform = render_entity.final_transform();
            let affine =
                render_entity.scene_affine(view, *final_transform, pixel_scale.0, viewport_size);

            commands.entity(entity).insert((affine, final_transform));
        }
    }
}

impl PrepareRenderInstance for ExtractedLottieAsset {
    fn final_transform(&self) -> PreparedTransform {
        PreparedTransform(self.asset_anchor.compute(
            self.asset.width,
            self.asset.height,
            &self.transform,
        ))
    }

    fn scene_affine(
        &self,
        view: &ExtractedView,
        world_transform: GlobalTransform,
        pixel_scale: f32,
        viewport_size: UVec2,
    ) -> PreparedAffine {
        let local_center_matrix = self.asset.local_transform_center.compute_matrix().inverse();

        let raw_transform = match self.render_mode {
            CoordinateSpace::ScreenSpace => {
                let mut model_matrix = world_transform.compute_matrix().mul_scalar(pixel_scale);

                let asset_size = Vec2::new(self.asset.width, self.asset.height);

                // Make the screen space vector instance sized to fill the
                // entire UI Node box if it's bundled with a Node
                if let Some(node) = &self.ui_node {
                    let fill_scale = node.size() / asset_size;
                    model_matrix.x_axis.x *= fill_scale.x;
                    model_matrix.y_axis.y *= fill_scale.y;
                }

                let mut local_center_matrix = local_center_matrix;
                local_center_matrix.w_axis.y *= -1.0;
                model_matrix * local_center_matrix
            }
            CoordinateSpace::WorldSpace => {
                let local_matrix = local_center_matrix;

                let (pixels_x, pixels_y) = (viewport_size.x as f32, viewport_size.y as f32);
                let ndc_to_pixels_matrix = Mat4::from_cols_array_2d(&[
                    [pixels_x / 2.0, 0.0, 0.0, pixels_x / 2.0],
                    [0.0, pixels_y / 2.0, 0.0, pixels_y / 2.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [0.0, 0.0, 0.0, 1.0],
                ])
                .transpose();

                let mut model_matrix = world_transform.compute_matrix() * local_matrix;
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

        PreparedAffine(Affine::new(transform))
    }
}
