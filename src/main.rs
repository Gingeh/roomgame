use std::f32::consts::PI;

use bevy::prelude::{shape::Box, *};
use bevy_inspector_egui::WorldInspectorPlugin;

/// Marker component for the desk/panel thing
#[derive(Component)]
struct Desk;

/// Marker component for the spotlight
#[derive(Component)]
struct Lamp;

/// Marker component for the buttons
#[derive(Component)]
enum Button {
    Red,
    Green,
    Blue,
    Yellow,
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(DefaultPlugins)
        //TODO: Feature gate the inpector
        .add_plugin(WorldInspectorPlugin::new())
        .add_startup_system(setup)
        .run();
}

/// Spawn the camera and panel
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera
    commands.spawn_bundle(Camera3dBundle {
        ..Default::default()
    });

    // Desk
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Box::new(2.0, 1.0, 1.0).into()),
            material: materials.add(Color::ANTIQUE_WHITE.into()),
            transform: Transform::from_translation(Vec3::new(0.0, -0.6, -2.0))
                .with_rotation(Quat::from_rotation_x(PI / 6.0)),
            ..Default::default()
        })
        .insert(Desk)
        .with_children(|parent| {
            // Desk lamp
            parent
                .spawn_bundle(SpotLightBundle {
                    spot_light: SpotLight {
                        intensity: 100.0,
                        outer_angle: 0.3,
                        shadows_enabled: true,
                        shadow_depth_bias: 0.0,
                        ..Default::default()
                    },
                    transform: Transform::from_translation(Vec3::new(-1.0, 1.0, -0.2))
                        .looking_at(Vec3::new(0.0, 0.5, 0.0), Vec3::Y),
                    ..Default::default()
                })
                .insert(Lamp);

            // Buttons
            //TODO: Can this be refactored?
            parent
                .spawn_bundle(PbrBundle {
                    mesh: meshes.add(Box::new(1.0, 1.0, 1.0).into()),
                    material: materials.add(Color::RED.into()),
                    transform: Transform::from_translation(Vec3::new(-0.12, 0.45, -0.12))
                        .with_scale(Vec3::splat(0.2)),
                    ..Default::default()
                })
                .insert(Button::Red);

            parent
                .spawn_bundle(PbrBundle {
                    mesh: meshes.add(Box::new(1.0, 1.0, 1.0).into()),
                    material: materials.add(Color::GREEN.into()),
                    transform: Transform::from_translation(Vec3::new(-0.12, 0.45, 0.12))
                        .with_scale(Vec3::splat(0.2)),
                    ..Default::default()
                })
                .insert(Button::Green);

            parent
                .spawn_bundle(PbrBundle {
                    mesh: meshes.add(Box::new(1.0, 1.0, 1.0).into()),
                    material: materials.add(Color::BLUE.into()),
                    transform: Transform::from_translation(Vec3::new(0.12, 0.45, -0.12))
                        .with_scale(Vec3::splat(0.2)),
                    ..Default::default()
                })
                .insert(Button::Blue);

            parent
                .spawn_bundle(PbrBundle {
                    mesh: meshes.add(Box::new(1.0, 1.0, 1.0).into()),
                    material: materials.add(Color::YELLOW.into()),
                    transform: Transform::from_translation(Vec3::new(0.12, 0.45, 0.12))
                        .with_scale(Vec3::splat(0.2)),
                    ..Default::default()
                })
                .insert(Button::Yellow);
        });
}
