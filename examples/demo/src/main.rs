mod ui;

use bevy::{
    asset::{embedded_asset, io::embedded::EmbeddedAssetRegistry, AssetMetaCheck},
    color::palettes::css,
    prelude::*,
};
use bevy_egui::EguiPlugin;
use bevy_vello::{prelude::*, VelloPlugin};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(AssetPlugin {
        meta_check: AssetMetaCheck::Never,
        ..default()
    }))
    .add_plugins(EguiPlugin)
    .add_plugins(VelloPlugin::default())
    .init_resource::<EmbeddedAssetRegistry>()
    .add_plugins(bevy_pancam::PanCamPlugin)
    .add_systems(Startup, setup_vector_graphics)
    .add_systems(Update, (print_metadata, ui::controls_ui));
    embedded_asset!(app, "assets/calendar.json");
    app.run();
}

fn setup_vector_graphics(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands.spawn((
        Camera2dBundle::default(),
        bevy_pancam::PanCam::default(),
        VelloView,
    ));
    commands
        .spawn(VelloAssetBundle {
            asset: asset_server.load::<VelloAsset>("embedded://demo/assets/calendar.json"),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0))
                .with_scale(Vec3::splat(20.0)),
            debug_visualizations: DebugVisualizations::Visible,
            ..default()
        })
        .insert(
            DotLottiePlayer::new("stopped")
                .with_state({
                    PlayerState::new("stopped")
                        .playback_options(PlaybackOptions {
                            autoplay: false,
                            ..default()
                        })
                        .theme(Theme::new().add("calendar", css::YELLOW.into()))
                        .transition(PlayerTransition::OnMouseEnter { state: "play" })
                        .reset_playhead_on_start()
                })
                .with_state(
                    PlayerState::new("play")
                        .playback_options(PlaybackOptions {
                            looping: PlaybackLoopBehavior::DoNotLoop,
                            speed: 0.75,
                            ..default()
                        })
                        .theme(Theme::new().add("calendar", css::LIME.into()))
                        .transition(PlayerTransition::OnMouseLeave { state: "rev" }),
                )
                .with_state(
                    PlayerState::new("rev")
                        .playback_options(PlaybackOptions {
                            looping: PlaybackLoopBehavior::DoNotLoop,
                            direction: PlaybackDirection::Reverse,
                            speed: 0.75,
                            ..default()
                        })
                        .theme(Theme::new().add("calendar", css::RED.into()))
                        .transition(PlayerTransition::OnMouseEnter { state: "play" })
                        .transition(PlayerTransition::OnComplete { state: "stopped" }),
                ),
        );
}

fn print_metadata(
    mut asset_ev: EventReader<AssetEvent<VelloAsset>>,
    assets: Res<Assets<VelloAsset>>,
) {
    for ev in asset_ev.read() {
        if let AssetEvent::LoadedWithDependencies { id } = ev {
            let asset = assets.get(*id).unwrap();
            #[allow(irrefutable_let_patterns)]
            if let VectorFile::Lottie(composition) = &asset.file {
                info!(
                    "Animated asset loaded. Layers:\n{:#?}",
                    composition.as_ref().get_layers().collect::<Vec<_>>()
                );
            }
        }
    }
}
