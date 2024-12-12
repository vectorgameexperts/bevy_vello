use crate::render::VelatoRenderer;

use super::{
    asset::VelloLottieHandle, asset_loader::VelloLottieLoader, render, systems, VelloLottie,
};
use bevy::{
    prelude::*,
    render::{
        view::{check_visibility, VisibilitySystems},
        Render, RenderApp, RenderSet,
    },
};

pub struct LottieIntegrationPlugin;

impl Plugin for LottieIntegrationPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<VelloLottieLoader>()
            .init_asset::<VelloLottie>()
            .add_systems(
                PostUpdate,
                (
                    systems::advance_playheads_without_options,
                    systems::advance_playheads_with_options,
                ),
            )
            .add_systems(Last, systems::spawn_playheads);

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .init_resource::<VelatoRenderer>()
            .add_systems(ExtractSchedule, render::extract_lottie_assets)
            .add_systems(
                Render,
                (render::prepare_asset_affines).in_set(RenderSet::Prepare),
            )
            .add_systems(
                PostUpdate,
                check_visibility::<Or<(With<VelloLottieHandle>,)>>
                    .in_set(VisibilitySystems::CheckVisibility),
            );
    }
}
