use std::{f32::consts::PI, mem, time::Duration};

use bevy::{
    prelude::{shape::Box, *},
    ui::FocusPolicy,
};

#[cfg(feature = "inspector")]
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_mod_picking::{DefaultPickingPlugins, PickableMesh, PickingCameraBundle};
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

/// `PickableBundle` without the other stuff
#[derive(Bundle, Default)]
struct ClickableBundle {
    pickable_mesh: PickableMesh,
    interaction: Interaction,
    focus_policy: FocusPolicy,
}
/// The current state of the game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum SimonState {
    MonkeySee, // Showing the pattern
    MonkeyDo,  // Waiting for the player
}

/// Event for things that happen in game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum SimonEvent {
    Success,
    Next,
    Failure,
}

/// Resource for ending `MonkeyDo`
// I don't like this :(
#[derive(Default)]
struct StateSwitch;

/// The pattern to remember
#[derive(Default)]
struct Pattern(Vec<Button>);

/// Progress along the pattern
#[derive(Default)]
struct Progress(usize);

/// The score to be displayed
#[derive(Default)]
struct Score {
    current: usize,
    high: usize,
}

/// Marker component for the scoreboard
#[derive(Component)]
struct Scoreboard;

#[derive(Default)]
struct AudioHandles {
    red: Option<Handle<AudioSource>>,
    green: Option<Handle<AudioSource>>,
    blue: Option<Handle<AudioSource>>,
    yellow: Option<Handle<AudioSource>>,
}

// I don't like using strings for identifiers
const FIXEDUPDATE: &str = "FixedUpdate";

fn main() {
    let mut app = App::new();
    app
        // Black background
        .insert_resource(ClearColor(Color::BLACK))

        // Default plugins (useful!)
        .add_plugins(DefaultPlugins)

        // Mouse support
        .add_plugins(DefaultPickingPlugins)

        // Spawn stuff
        .add_startup_system(setup)
        .add_startup_system(load_assets)

        // Manage the buttons
        .add_event::<ButtonEvent>()
        .init_resource::<AudioHandles>()
        .add_system(button_event_handler)
        .add_system(button_state_manager)
        .add_system(button_controller)
        .add_system(play_button_sound)

        // Store the pattern as a resource
        .init_resource::<Pattern>()
        .init_resource::<Progress>()

        // The "Monkey See" state
        .add_loopless_state(SimonState::MonkeySee)
        .add_enter_system(SimonState::MonkeySee, update_pattern)
        .add_fixed_timestep(Duration::from_secs_f32(1.0), FIXEDUPDATE)
        .add_fixed_timestep_system(
            FIXEDUPDATE,
            0,
            show_button.run_in_state(SimonState::MonkeySee),
        )

        // The "Monkey Do" state
        .add_event::<SimonEvent>()
        .add_system(press_buttons.run_in_state(SimonState::MonkeyDo))
        .add_system(validate_buttons.run_in_state(SimonState::MonkeyDo))
        .add_system(game_event_handler.run_in_state(SimonState::MonkeyDo))
        .add_fixed_timestep_system(
            FIXEDUPDATE,
            0,
            state_switch_event_handler.run_in_state(SimonState::MonkeyDo),
        )

        .init_resource::<Score>()
        .add_system(update_score)
        .add_system(update_scoreboard);

    // Include an inspector if the `inspector` feature is enabled
    #[cfg(feature = "inspector")]
    app.add_plugin(WorldInspectorPlugin::new());

    app.run();
}

/// Spawns the camera and panel
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Camera
    commands
        .spawn_bundle(Camera3dBundle::default())
        .insert_bundle(PickingCameraBundle::default());

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
                .insert_bundle(ClickableBundle::default())
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
                .insert_bundle(ClickableBundle::default())
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
                .insert_bundle(ClickableBundle::default())
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
                .insert_bundle(ClickableBundle::default())
                .insert(ButtonState::Inactive)
                .insert(PreviousButtonState(ButtonState::Inactive))
                .insert(Button::Yellow);
        });

    let score_textstyle = TextStyle {
        font: asset_server.load("fonts/comic.ttf"),
        font_size: 36.0,
        color: Color::WHITE,
    };

    commands
        .spawn_bundle(TextBundle {
            text: Text::from_sections([
                TextSection {
                    value: "Score: ".into(),
                    style: score_textstyle.clone(),
                },
                TextSection {
                    value: "0".into(),
                    style: score_textstyle.clone(),
                },
                TextSection {
                    value: "\nHigh Score: ".into(),
                    style: score_textstyle.clone(),
                },
                TextSection {
                    value: "0".into(),
                    style: score_textstyle,
                },
            ])
            .with_alignment(TextAlignment::CENTER_RIGHT),
            style: Style {
                margin: UiRect {
                    left: Val::Percent(2.0),
                    right: Val::Undefined,
                    top: Val::Undefined,
                    bottom: Val::Percent(2.0),
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Scoreboard);
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
                    *state = ButtonState::Pressed { timer: 0.5 };
                }
            }
            ButtonEvent::Lit(button) => {
                for (_, mut state, mut previous) in
                    buttons.iter_mut().filter(|(b, _, _)| *b == button)
                {
                    *previous = PreviousButtonState(*state);
                    *state = ButtonState::Lit { timer: 0.8 };
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

/// Animates the buttons
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

/// Adds a random button to the pattern
fn update_pattern(mut pattern: ResMut<Pattern>) {
    let button: Button = rand::random();
    pattern.0.push(button);
}

/// Shows the next button in the pattern or ends the "Monkey See" state
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

fn press_buttons(
    interactions: Query<(&Interaction, &Button), Changed<Interaction>>,
    mut button_event_writer: EventWriter<ButtonEvent>,
) {
    for (interaction, button) in interactions.iter() {
        if *interaction == Interaction::Clicked {
            button_event_writer.send(ButtonEvent::Pressed(*button));
        }
    }
}

/// Handles button events during `MonkeyDo`
fn validate_buttons(
    mut event_writer: EventWriter<SimonEvent>,
    mut event_reader: EventReader<ButtonEvent>,
    pattern: Res<Pattern>,
    progress: Res<Progress>,
) {
    for event in event_reader.iter() {
        if let ButtonEvent::Pressed(button) = event {
            if *button == pattern.0[progress.0] {
                if progress.0 == pattern.0.len() - 1 {
                    event_writer.send(SimonEvent::Success);
                } else {
                    event_writer.send(SimonEvent::Next);
                }
            } else {
                event_writer.send(SimonEvent::Failure);
            }
        }
    }
}

fn game_event_handler(
    mut commands: Commands,
    mut event_reader: EventReader<SimonEvent>,
    mut progress: ResMut<Progress>,
    mut pattern: ResMut<Pattern>,
) {
    for event in event_reader.iter() {
        match event {
            SimonEvent::Success => {
                progress.0 = 0;
                commands.insert_resource(StateSwitch);
            }
            SimonEvent::Next => {
                progress.0 += 1;
            }
            SimonEvent::Failure => {
                progress.0 = 0;
                pattern.0 = Vec::new();
                commands.insert_resource(StateSwitch);
            }
        }
    }
}

fn state_switch_event_handler(mut commands: Commands, state_switch: Option<Res<StateSwitch>>) {
    if state_switch.is_some() {
        commands.remove_resource::<StateSwitch>();
        commands.insert_resource(NextState(SimonState::MonkeySee));
    }
}

fn update_score(mut event_reader: EventReader<SimonEvent>, mut score: ResMut<Score>) {
    for event in event_reader.iter() {
        match event {
            SimonEvent::Success => {
                score.current += 1;
                if score.current > score.high {
                    score.high = score.current;
                }
            }
            SimonEvent::Failure => score.current = 0,
            SimonEvent::Next => {}
        }
    }
}

fn update_scoreboard(score: Res<Score>, mut score_text_query: Query<&mut Text, With<Scoreboard>>) {
    if score.is_changed() {
        for mut score_text in score_text_query.iter_mut() {
            score_text.sections[1].value = score.current.to_string();
            score_text.sections[3].value = score.high.to_string();
        }
    }
}

fn load_assets(asset_server: Res<AssetServer>, mut audio_handles: ResMut<AudioHandles>) {
    audio_handles.red = Some(asset_server.load("sounds/buttons/red.ogg"));
    audio_handles.green = Some(asset_server.load("sounds/buttons/green.ogg"));
    audio_handles.blue = Some(asset_server.load("sounds/buttons/blue.ogg"));
    audio_handles.yellow = Some(asset_server.load("sounds/buttons/yellow.ogg"));
}

fn play_button_sound(
    mut event_reader: EventReader<ButtonEvent>,
    audio: Res<Audio>,
    audio_handles: Res<AudioHandles>,
) {
    for event in event_reader.iter() {
        let button = match event {
            ButtonEvent::Pressed(button) => button,
            ButtonEvent::Lit(button) => button,
        };
        if let Some(audio_handle) = match button {
            Button::Red => &audio_handles.red,
            Button::Green => &audio_handles.green,
            Button::Blue => &audio_handles.blue,
            Button::Yellow => &audio_handles.yellow,
        } {
            audio.play(audio_handle.clone());
        };
    }
}
