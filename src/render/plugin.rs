use super::{
    extract::{self, ExtractedPixelScale, SSRenderTarget},
    prepare, systems, VelloCanvasSettings, VelloRenderSettings,
};
use crate::{
    render::{VelloCanvasMaterial, VelloRenderer, SSRT_SHADER_HANDLE},
    VelloAssetHandle, VelloFont, VelloScene, VelloTextSection,
};
use bevy::{
    asset::load_internal_asset,
    prelude::*,
    render::{
        extract_component::ExtractComponentPlugin,
        render_asset::RenderAssetPlugin,
        renderer::RenderDevice,
        view::{check_visibility, VisibilitySystems},
        Render, RenderApp, RenderSet,
    },
    sprite::Material2dPlugin,
};

#[derive(Default)]
pub struct VelloRenderPlugin {
    /// Settings used for the canvas
    pub canvas_settings: VelloCanvasSettings,

    /// Settings used for rendering with Vello
    pub render_settings: VelloRenderSettings,
}

impl Plugin for VelloRenderPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            SSRT_SHADER_HANDLE,
            "../../shaders/vello_ss_rendertarget.wgsl",
            Shader::from_wgsl
        );

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        #[cfg(feature = "svg")]
        render_app.add_systems(ExtractSchedule, extract::extract_svg_assets);
        #[cfg(feature = "lottie")]
        render_app
            .init_resource::<super::VelatoRenderer>()
            .add_systems(ExtractSchedule, extract::extract_lottie_assets);

        render_app
            .insert_resource(self.render_settings.clone())
            .insert_resource(ExtractedPixelScale(1.0))
            .add_systems(
                Render,
                systems::render_settings_change_detection.in_set(RenderSet::Cleanup),
            )
            .add_systems(
                ExtractSchedule,
                (
                    extract::extract_pixel_scale.in_set(RenderSet::ExtractCommands),
                    extract::extract_scenes,
                    extract::extract_text,
                ),
            )
            .add_systems(
                Render,
                (
                    prepare::prepare_asset_affines,
                    prepare::prepare_scene_affines,
                    prepare::prepare_text_affines,
                )
                    .in_set(RenderSet::Prepare),
            )
            .add_systems(
                Render,
                systems::render_frame
                    .in_set(RenderSet::Render)
                    .run_if(resource_exists::<RenderDevice>),
            );

        app.insert_resource(self.canvas_settings.clone())
            .add_plugins((
                Material2dPlugin::<VelloCanvasMaterial>::default(),
                ExtractComponentPlugin::<SSRenderTarget>::default(),
                RenderAssetPlugin::<VelloFont>::default(),
            ))
            .add_systems(Startup, systems::setup_ss_rendertarget)
            .add_systems(
                Update,
                (systems::resize_rendertargets, systems::hide_when_empty),
            )
            .add_systems(
                PostUpdate,
                check_visibility::<
                    Or<(
                        With<VelloScene>,
                        With<VelloAssetHandle>,
                        With<VelloTextSection>,
                    )>,
                >
                    .in_set(VisibilitySystems::CheckVisibility),
            );
    }

    fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app.init_resource::<VelloRenderer>();
    }
}
