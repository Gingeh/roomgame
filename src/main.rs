use std::{f32::consts::PI, mem, time::Duration};

use bevy::prelude::{shape::Box, *};

#[cfg(feature = "inspector")]
use bevy_inspector_egui::WorldInspectorPlugin;
use iyes_loopless::prelude::*;
use rand::{distributions::Standard, prelude::Distribution, Rng};

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

impl Distribution<Button> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Button {
        match rng.gen_range(0..=3) {
            0 => Button::Red,
            1 => Button::Green,
            2 => Button::Blue,
            _ => Button::Yellow,
        }
    }
}

/// Event for pressing and lighting up buttons
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ButtonEvent {
    Pressed(Button),
    Lit(Button),
}

/// Stores the button's state and timer
#[derive(Component, Clone, Copy)]
enum ButtonState {
    Inactive,
    Pressed { timer: f32 },
    Lit { timer: f32 },
}

/// Stores the button's previous state
#[derive(Component)]
struct PreviousButtonState(ButtonState);

/// The current state of the game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum SimonState {
    MonkeySee, // Showing the pattern
    MonkeyDo,  // Waiting for the player
}

/// The pattern to remember
#[derive(Default)]
struct Pattern(Vec<Button>);

/// Progress along the pattern
#[derive(Default)]
struct Progress(usize);

// I don't like using strings for identifiers
const FIXEDUPDATE: &str = "FixedUpdate";

fn main() {
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::BLACK))
        .add_plugins(DefaultPlugins)
        .add_event::<ButtonEvent>()
        .add_startup_system(setup)
        .add_system(button_event_handler)
        .add_system(button_state_manager)
        .add_system(button_controller)
        .init_resource::<Pattern>()
        .init_resource::<Progress>()
        .add_loopless_state(SimonState::MonkeySee)
        .add_enter_system(SimonState::MonkeySee, update_pattern)
        .add_fixed_timestep(Duration::from_secs_f32(1.2), FIXEDUPDATE)
        .add_fixed_timestep_system(
            FIXEDUPDATE,
            0,
            show_button.run_in_state(SimonState::MonkeySee),
        );

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
                .insert(ButtonState::Inactive)
                .insert(PreviousButtonState(ButtonState::Inactive))
                .insert(Button::Red);

            parent
                .spawn_bundle(PbrBundle {
                    mesh: meshes.add(Box::new(1.0, 1.0, 1.0).into()),
                    material: materials.add(Color::GREEN.into()),
                    transform: Transform::from_translation(Vec3::new(-0.12, 0.47, 0.12))
                        .with_scale(Vec3::splat(0.2)),
                    ..Default::default()
                })
                .insert(ButtonState::Inactive)
                .insert(PreviousButtonState(ButtonState::Inactive))
                .insert(Button::Green);

            parent
                .spawn_bundle(PbrBundle {
                    mesh: meshes.add(Box::new(1.0, 1.0, 1.0).into()),
                    material: materials.add(Color::BLUE.into()),
                    transform: Transform::from_translation(Vec3::new(0.12, 0.47, -0.12))
                        .with_scale(Vec3::splat(0.2)),
                    ..Default::default()
                })
                .insert(ButtonState::Inactive)
                .insert(PreviousButtonState(ButtonState::Inactive))
                .insert(Button::Blue);

            parent
                .spawn_bundle(PbrBundle {
                    mesh: meshes.add(Box::new(1.0, 1.0, 1.0).into()),
                    material: materials.add(Color::YELLOW.into()),
                    transform: Transform::from_translation(Vec3::new(0.12, 0.47, 0.12))
                        .with_scale(Vec3::splat(0.2)),
                    ..Default::default()
                })
                .insert(ButtonState::Inactive)
                .insert(PreviousButtonState(ButtonState::Inactive))
                .insert(Button::Yellow);
        });
}

/// Handles `ButtonEvent`s and sets `ButtonState`s
fn button_event_handler(
    mut event_reader: EventReader<ButtonEvent>,
    mut buttons: Query<(&Button, &mut ButtonState, &mut PreviousButtonState)>,
) {
    for event in event_reader.iter() {
        match event {
            ButtonEvent::Pressed(button) => {
                for (_, mut state, mut previous) in
                    buttons.iter_mut().filter(|(b, _, _)| *b == button)
                {
                    *previous = PreviousButtonState(*state);
                    *state = ButtonState::Pressed { timer: 1.0 };
                }
            }
            ButtonEvent::Lit(button) => {
                for (_, mut state, mut previous) in
                    buttons.iter_mut().filter(|(b, _, _)| *b == button)
                {
                    *previous = PreviousButtonState(*state);
                    *state = ButtonState::Lit { timer: 1.0 };
                }
            }
        }
    }
}

/// Manages `ButtonState`s and their timers
fn button_state_manager(
    mut buttons: Query<(&mut ButtonState, &mut PreviousButtonState)>,
    time: Res<Time>,
) {
    for (mut state, mut previous) in buttons.iter_mut() {
        match *state {
            ButtonState::Inactive => {}
            ButtonState::Pressed { timer } => {
                if timer > 0.0 {
                    *state = ButtonState::Pressed {
                        timer: timer - time.delta_seconds(),
                    }
                } else {
                    *previous = PreviousButtonState(*state);
                    *state = ButtonState::Inactive;
                }
            }
            ButtonState::Lit { timer } => {
                if timer > 0.0 {
                    *state = ButtonState::Lit {
                        timer: timer - time.delta_seconds(),
                    }
                } else {
                    *previous = PreviousButtonState(*state);
                    *state = ButtonState::Inactive;
                }
            }
        }
    }
}

fn button_controller(
    mut buttons: Query<(
        &ButtonState,
        &mut PreviousButtonState,
        &mut Transform,
        &Handle<StandardMaterial>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (state, mut previous, mut transform, material_handle) in buttons.iter_mut() {
        let material = materials.get_mut(material_handle).unwrap();

        if mem::discriminant(&previous.0) != mem::discriminant(state) {
            match *state {
                ButtonState::Inactive => {
                    material.emissive = Color::BLACK;
                    if matches!(previous.0, ButtonState::Pressed { .. }) {
                        transform.translation.y += 0.02;
                    }
                    *previous = PreviousButtonState(*state);
                }
                ButtonState::Pressed { .. } => {
                    material.emissive = material.base_color;
                    transform.translation.y -= 0.02;
                    *previous = PreviousButtonState(*state);
                }
                ButtonState::Lit { .. } => {
                    material.emissive = material.base_color;
                    *previous = PreviousButtonState(*state);
                }
            }
        }
    }
}

fn update_pattern(mut pattern: ResMut<Pattern>) {
    let button: Button = rand::random();
    pattern.0.push(button);
}

fn show_button(
    mut commands: Commands,
    mut progress: ResMut<Progress>,
    pattern: Res<Pattern>,
    mut button_event_writer: EventWriter<ButtonEvent>,
) {
    if let Some(button) = pattern.0.get(progress.0) {
        button_event_writer.send(ButtonEvent::Lit(*button));
        progress.0 += 1;
    } else {
        progress.0 = 0;
        commands.insert_resource(NextState(SimonState::MonkeyDo));
    }
}
