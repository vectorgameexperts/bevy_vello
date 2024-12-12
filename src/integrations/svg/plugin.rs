use super::{asset::VelloSvgHandle, asset_loader::VelloSvgLoader, render, VelloSvg};
use bevy::{
    prelude::*,
    render::{
        view::{check_visibility, VisibilitySystems},
        Render, RenderApp, RenderSet,
    },
};

pub struct SvgIntegrationPlugin;

impl Plugin for SvgIntegrationPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<VelloSvgLoader>()
            .init_asset::<VelloSvg>();

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .add_systems(ExtractSchedule, render::extract_svg_assets)
            .add_systems(
                Render,
                (render::prepare_asset_affines,).in_set(RenderSet::Prepare),
            )
            .add_systems(
                PostUpdate,
                check_visibility::<Or<(With<VelloSvgHandle>,)>>
                    .in_set(VisibilitySystems::CheckVisibility),
            );
    }
}
