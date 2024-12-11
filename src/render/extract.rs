use crate::prelude::*;
use bevy::{
    prelude::*,
    render::{extract_component::ExtractComponent, view::RenderLayers, Extract},
    window::PrimaryWindow,
};

#[derive(Component, Clone)]
pub struct ExtractedRenderAsset {
    pub asset: VelloAsset,
    pub asset_anchor: VelloAssetAnchor,
    pub transform: GlobalTransform,
    pub render_mode: CoordinateSpace,
    pub ui_node: Option<ComputedNode>,
    pub render_layers: Option<RenderLayers>,
    pub alpha: f32,
    #[cfg(feature = "lottie")]
    pub theme: Option<crate::Theme>,
    #[cfg(feature = "lottie")]
    pub playhead: f64,
}

#[cfg(feature = "svg")]
pub fn extract_svg_assets(
    mut commands: Commands,
    query_vectors: Extract<
        Query<
            (
                &VelloAssetHandle,
                &VelloAssetAnchor,
                &CoordinateSpace,
                &GlobalTransform,
                Option<&ComputedNode>,
                Option<&RenderLayers>,
                &ViewVisibility,
                &InheritedVisibility,
            ),
            Without<SkipEncoding>,
        >,
    >,
    assets: Extract<Res<Assets<VelloAsset>>>,
) {
    for (
        asset,
        asset_anchor,
        coord_space,
        transform,
        ui_node,
        render_layers,
        view_visibility,
        inherited_visibility,
    ) in query_vectors.iter()
    {
        if let Some(
            asset @ VelloAsset {
                file: _file @ crate::VectorFile::Svg(_),
                alpha,
                ..
            },
        ) = assets.get(asset.id())
        {
            if view_visibility.get() && inherited_visibility.get() {
                commands.spawn(ExtractedRenderAsset {
                    asset: asset.to_owned(),
                    transform: *transform,
                    asset_anchor: *asset_anchor,
                    render_mode: *coord_space,
                    ui_node: ui_node.cloned(),
                    render_layers: render_layers.cloned(),
                    alpha: *alpha,
                    #[cfg(feature = "lottie")]
                    theme: None,
                    #[cfg(feature = "lottie")]
                    playhead: 0.0,
                });
            }
        }
    }
}

#[cfg(feature = "lottie")]
pub fn extract_lottie_assets(
    mut commands: Commands,
    query_vectors: Extract<
        Query<
            (
                &VelloAssetHandle,
                &VelloAssetAnchor,
                &CoordinateSpace,
                &GlobalTransform,
                &crate::Playhead,
                Option<&crate::Theme>,
                Option<&ComputedNode>,
                Option<&RenderLayers>,
                &ViewVisibility,
                &InheritedVisibility,
            ),
            Without<SkipEncoding>,
        >,
    >,
    assets: Extract<Res<Assets<VelloAsset>>>,
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
        if let Some(
            asset @ VelloAsset {
                file: _file @ crate::VectorFile::Lottie(_),
                alpha,
                ..
            },
        ) = assets.get(asset.id())
        {
            if view_visibility.get() && inherited_visibility.get() {
                let playhead = playhead.frame();
                commands.spawn(ExtractedRenderAsset {
                    asset: asset.to_owned(),
                    transform: *transform,
                    asset_anchor: *asset_anchor,
                    theme: theme.cloned(),
                    render_mode: *coord_space,
                    playhead,
                    alpha: *alpha,
                    ui_node: ui_node.cloned(),
                    render_layers: render_layers.cloned(),
                });
            }
        }
    }
}

#[derive(Component, Clone)]
pub struct ExtractedRenderScene {
    pub scene: VelloScene,
    pub transform: GlobalTransform,
    pub render_mode: CoordinateSpace,
    pub ui_node: Option<ComputedNode>,
    pub render_layers: Option<RenderLayers>,
}

pub fn extract_scenes(
    mut commands: Commands,
    query_scenes: Extract<
        Query<
            (
                &VelloScene,
                &CoordinateSpace,
                &GlobalTransform,
                &ViewVisibility,
                &InheritedVisibility,
                Option<&ComputedNode>,
                Option<&RenderLayers>,
            ),
            Without<SkipEncoding>,
        >,
    >,
) {
    for (
        scene,
        coord_space,
        transform,
        view_visibility,
        inherited_visibility,
        ui_node,
        render_layers,
    ) in query_scenes.iter()
    {
        if view_visibility.get() && inherited_visibility.get() {
            commands.spawn(ExtractedRenderScene {
                transform: *transform,
                render_mode: *coord_space,
                scene: scene.clone(),
                ui_node: ui_node.cloned(),
                render_layers: render_layers.cloned(),
            });
        }
    }
}

#[derive(Component, Clone)]
pub struct ExtractedRenderText {
    pub text: VelloTextSection,
    pub text_anchor: VelloTextAnchor,
    pub transform: GlobalTransform,
    pub render_space: CoordinateSpace,
    pub render_layers: Option<RenderLayers>,
}

pub fn extract_text(
    mut commands: Commands,
    query_scenes: Extract<
        Query<
            (
                &VelloTextSection,
                &VelloTextAnchor,
                &GlobalTransform,
                &ViewVisibility,
                &InheritedVisibility,
                &CoordinateSpace,
                Option<&RenderLayers>,
            ),
            Without<SkipEncoding>,
        >,
    >,
) {
    for (
        text,
        text_anchor,
        transform,
        view_visibility,
        inherited_visibility,
        render_space,
        render_layers,
    ) in query_scenes.iter()
    {
        if view_visibility.get() && inherited_visibility.get() {
            commands.spawn(ExtractedRenderText {
                text: text.clone(),
                text_anchor: *text_anchor,
                transform: *transform,
                render_space: *render_space,
                render_layers: render_layers.cloned(),
            });
        }
    }
}

/// A screenspace render target. We use a resizable fullscreen quad.
#[derive(Component, Default)]
pub struct SSRenderTarget(pub Handle<Image>);

impl ExtractComponent for SSRenderTarget {
    type QueryData = &'static SSRenderTarget;

    type QueryFilter = ();

    type Out = Self;

    fn extract_component(
        ss_render_target: bevy::ecs::query::QueryItem<'_, Self::QueryData>,
    ) -> Option<Self> {
        Some(Self(ss_render_target.0.clone()))
    }
}

#[derive(Resource)]
pub struct ExtractedPixelScale(pub f32);

pub fn extract_pixel_scale(
    mut pixel_scale: ResMut<ExtractedPixelScale>,
    windows: Extract<Query<&Window, With<PrimaryWindow>>>,
) {
    let scale_factor = windows
        .get_single()
        .map(|window| window.resolution.scale_factor())
        .unwrap_or(1.0);
    pixel_scale.0 = scale_factor;
}
