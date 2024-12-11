//! Shows how to use render layers.

use bevy::{color::palettes::css, prelude::*, render::view::RenderLayers};
use bevy_vello::{prelude::*, VelloPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(VelloPlugin {
            canvas_render_layers: RenderLayers::layer(1).with(2),
            ..default()
        })
        .add_systems(Startup, (setup_gizmos, setup_scene))
        .add_systems(Update, (animation, background, run_gizmos))
        .run();
}

/// A tag that will mark the scene with animation.
#[derive(Component)]
struct AnimationScene;

/// A tag that will mark the scene with the blue square.
#[derive(Component)]
struct BackgroundScene;

fn setup_gizmos(mut commands: Commands, mut config_store: ResMut<GizmoConfigStore>) {
    // This camera can only see Gizmos.
    commands.spawn((
        Camera2d,
        Camera {
            // This camera will render LAST.
            order: 1,
            ..default()
        },
        RenderLayers::layer(3),
    ));
    let (config, _) = config_store.config_mut::<DefaultGizmoConfigGroup>();
    config.render_layers = RenderLayers::layer(3);
}

fn setup_scene(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            // This camera will render first.
            order: -1,
            ..default()
        },
        RenderLayers::layer(1).with(2),
    ));

    commands.spawn((
        VelloSceneBundle::default(),
        BackgroundScene,
        RenderLayers::layer(1),
    ));
    commands.spawn((
        VelloSceneBundle::default(),
        AnimationScene,
        RenderLayers::layer(2),
    ));
}

fn animation(
    mut query_scene: Query<(&mut Transform, &mut VelloScene), With<AnimationScene>>,
    time: Res<Time>,
) {
    let sin_time = time.elapsed_secs().sin().mul_add(0.5, 0.5);
    let (mut transform, mut scene) = query_scene.single_mut();
    // Reset scene every frame
    *scene = VelloScene::default();

    // Animate color green to blue
    let c = Vec3::lerp(
        Vec3::new(0.0, 1.0, -1.0),
        Vec3::new(0.0, 1.0, 1.0),
        sin_time + 0.5,
    );

    // Animate the corner radius
    scene.fill(
        peniko::Fill::NonZero,
        kurbo::Affine::default(),
        peniko::Color::new([c.x, c.y, c.z, 1.]),
        None,
        &kurbo::RoundedRect::new(-50.0, -50.0, 50.0, 50.0, (sin_time as f64) * 50.0),
    );

    transform.scale = Vec3::lerp(Vec3::ONE * 0.5, Vec3::ONE * 1.0, sin_time);
    transform.translation = Vec3::lerp(Vec3::X * -100.0, Vec3::X * 100.0, sin_time);
    transform.rotation = Quat::from_rotation_z(-std::f32::consts::TAU * sin_time);
}

fn background(mut query_scene: Query<&mut VelloScene, With<BackgroundScene>>) {
    let mut scene = query_scene.single_mut();
    *scene = VelloScene::default();
    scene.fill(
        peniko::Fill::NonZero,
        kurbo::Affine::default(),
        peniko::Color::new([0.0, 0.0, 1.0, 1.0]),
        None,
        &kurbo::RoundedRect::new(-200.0, -200.0, 200.0, 200.0, 0.0),
    );
}

fn run_gizmos(mut gizmos: Gizmos) {
    gizmos.circle_2d(Vec2::splat(0.0), 100.0, css::MAGENTA);
}
