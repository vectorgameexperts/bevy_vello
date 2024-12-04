use bevy::{
    asset::{embedded_asset, AssetMetaCheck},
    prelude::*,
};
use bevy_vello::{prelude::*, VelloPlugin};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(AssetPlugin {
        meta_check: AssetMetaCheck::Never,
        ..default()
    }))
    .add_plugins(VelloPlugin::default())
    .add_systems(Startup, load_lottie);
    embedded_asset!(app, "assets/Tiger.json");
    app.run();
}

fn load_lottie(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands.spawn((Camera2dBundle::default(), VelloView));

    // Yes, it's this simple.
    commands.spawn(VelloAssetBundle {
        asset: asset_server.load("embedded://lottie/assets/Tiger.json"),
        debug_visualizations: DebugVisualizations::Visible,
        transform: Transform::from_scale(Vec3::splat(0.5)),
        ..default()
    });
}
