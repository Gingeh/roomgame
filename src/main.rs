use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 2.2, 0.0).looking_at(Vec3::new(-1.6, 1.6, 0.0), Vec3::Y),
        ..default()
    });
    commands.spawn_bundle(SceneBundle {
        scene: asset_server.load("panel.gltf#Scene0"),
        ..default()
    });
}