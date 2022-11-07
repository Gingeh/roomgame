use std::f32::consts::PI;

use bevy::prelude::{shape::Box, *};

#[cfg(feature = "inspector")]
use bevy_inspector_egui::WorldInspectorPlugin;

/// Marker component for the desk/panel thing
#[derive(Component)]
struct Desk;

/// Marker component for the spotlight
#[derive(Component)]
struct Lamp;

/// Marker component for the buttons
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Button {
    Red,
    Green,
    Blue,
    Yellow,
}

/// Event for pressing and lighting up buttons
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ButtonEvent {
    Pressed(Button),
    Lit(Button),
}

/// Stores the button's state, timer and whether it changed recently
#[derive(Component)]
enum ButtonState {
    Inactive { changed: bool },
    Pressed { changed: bool, timer: f32 },
    Lit { changed: bool, timer: f32 },
}

fn main() {
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::BLACK))
        .add_plugins(DefaultPlugins)
        .add_event::<ButtonEvent>()
        .add_startup_system(setup)
        .add_system(button_event_handler)
        .add_system(button_state_manager)
        .add_system(button_controller);

    #[cfg(feature = "inspector")]
    app.add_plugin(WorldInspectorPlugin::new());

    app.run();
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
                    transform: Transform::from_translation(Vec3::new(-0.12, 0.47, -0.12))
                        .with_scale(Vec3::splat(0.2)),
                    ..Default::default()
                })
                .insert(ButtonState::Inactive { changed: false })
                .insert(Button::Red);

            parent
                .spawn_bundle(PbrBundle {
                    mesh: meshes.add(Box::new(1.0, 1.0, 1.0).into()),
                    material: materials.add(Color::GREEN.into()),
                    transform: Transform::from_translation(Vec3::new(-0.12, 0.47, 0.12))
                        .with_scale(Vec3::splat(0.2)),
                    ..Default::default()
                })
                .insert(ButtonState::Inactive { changed: false })
                .insert(Button::Green);

            parent
                .spawn_bundle(PbrBundle {
                    mesh: meshes.add(Box::new(1.0, 1.0, 1.0).into()),
                    material: materials.add(Color::BLUE.into()),
                    transform: Transform::from_translation(Vec3::new(0.12, 0.47, -0.12))
                        .with_scale(Vec3::splat(0.2)),
                    ..Default::default()
                })
                .insert(ButtonState::Inactive { changed: false })
                .insert(Button::Blue);

            parent
                .spawn_bundle(PbrBundle {
                    mesh: meshes.add(Box::new(1.0, 1.0, 1.0).into()),
                    material: materials.add(Color::YELLOW.into()),
                    transform: Transform::from_translation(Vec3::new(0.12, 0.47, 0.12))
                        .with_scale(Vec3::splat(0.2)),
                    ..Default::default()
                })
                .insert(ButtonState::Inactive { changed: false })
                .insert(Button::Yellow);
        });
}

/// Handles `ButtonEvent`s and sets `ButtonState`s
fn button_event_handler(
    mut event_reader: EventReader<ButtonEvent>,
    mut buttons: Query<(&Button, &mut ButtonState)>,
) {
    if let Some(event) = event_reader.iter().next() {
        match event {
            ButtonEvent::Pressed(button) => {
                for (_, mut state) in buttons.iter_mut().filter(|(b, _)| *b == button) {
                    *state = ButtonState::Pressed {
                        timer: 1.0,
                        changed: true,
                    };
                }
            }
            ButtonEvent::Lit(button) => {
                for (_, mut state) in buttons.iter_mut().filter(|(b, _)| *b == button) {
                    *state = ButtonState::Lit {
                        timer: 1.0,
                        changed: true,
                    };
                }
            }
        }
    }
}

/// Manages `ButtonState`s and their timers
fn button_state_manager(mut buttons: Query<&mut ButtonState>, time: Res<Time>) {
    for mut state in buttons.iter_mut() {
        match *state {
            ButtonState::Inactive { changed: _ } => {}
            ButtonState::Pressed { timer, changed } => {
                if timer > 0.0 {
                    *state = ButtonState::Pressed {
                        timer: timer - time.delta_seconds(),
                        changed,
                    }
                } else {
                    *state = ButtonState::Inactive { changed: true }
                }
            }
            ButtonState::Lit { timer, changed } => {
                if timer > 0.0 {
                    *state = ButtonState::Lit {
                        timer: timer - time.delta_seconds(),
                        changed,
                    }
                } else {
                    *state = ButtonState::Inactive { changed: true }
                }
            }
        }
    }
}

fn button_controller(
    mut buttons: Query<(&mut ButtonState, &mut Transform, &Handle<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (mut state, mut transform, material_handle) in buttons.iter_mut() {
        let material = materials.get_mut(material_handle).unwrap();

        match *state {
            ButtonState::Inactive { changed: true } => {
                material.emissive = Color::BLACK;
                transform.translation.y += 0.02;
                *state = ButtonState::Inactive { changed: false }
            }
            ButtonState::Pressed {
                timer,
                changed: true,
            } => {
                material.emissive = material.base_color;
                transform.translation.y -= 0.02;
                *state = ButtonState::Pressed {
                    timer,
                    changed: false,
                };
            }
            ButtonState::Lit {
                timer,
                changed: true,
            } => {
                material.emissive = material.base_color;
                *state = ButtonState::Lit {
                    timer,
                    changed: false,
                };
            }
            _ => {}
        }
    }
}
