use crate::{
    debug::DebugVisualizationsPlugin,
    render::{VelloCanvasSettings, VelloRenderPlugin},
    text::VelloFontLoader,
    VelloFont, VelloRenderSettings,
};
use bevy::{asset::load_internal_binary_asset, prelude::*, render::view::RenderLayers};
use vello::AaConfig;

#[derive(Clone)]
pub struct VelloPlugin {
    /// The render layers that will be used for the Vello canvas mesh.
    pub canvas_render_layers: RenderLayers,

    /// Use CPU instead of GPU
    pub use_cpu: bool,

    /// Which antialiasing strategy to use
    pub antialiasing: AaConfig,
}

impl Default for VelloPlugin {
    fn default() -> Self {
        let default_canvas_settings = VelloCanvasSettings::default();
        let default_render_settings = VelloRenderSettings::default();
        Self {
            canvas_render_layers: default_canvas_settings.render_layers,
            use_cpu: default_render_settings.use_cpu,
            antialiasing: default_render_settings.antialiasing,
        }
    }
}

impl Plugin for VelloPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(VelloRenderPlugin {
            canvas_settings: VelloCanvasSettings {
                render_layers: self.canvas_render_layers.clone(),
            },
            render_settings: VelloRenderSettings {
                use_cpu: self.use_cpu,
                antialiasing: self.antialiasing,
            },
        })
        .add_plugins(DebugVisualizationsPlugin)
        .init_asset::<VelloFont>()
        .init_asset_loader::<VelloFontLoader>();
        #[cfg(feature = "svg")]
        app.add_plugins(crate::integrations::svg::SvgIntegrationPlugin);
        #[cfg(feature = "lottie")]
        app.add_plugins(crate::integrations::lottie::LottieIntegrationPlugin);
        #[cfg(feature = "experimental-dotLottie")]
        app.add_plugins(crate::integrations::dot_lottie::DotLottieIntegrationPlugin);
        #[cfg(feature = "default_font")]
        load_internal_binary_asset!(
            app,
            Handle::default(),
            "text/FiraMono/FiraMono-subset.ttf",
            |bytes: &[u8], _path: String| { VelloFont::new(bytes.to_vec()) }
        );
    }
}
