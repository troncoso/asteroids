use crate::components::*;
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;
use resources::{FireTimer, Level, FILL_COLOR};

mod asteroid;
mod components;
mod resources;
mod ship;

fn main() {
    App::new()
        // Set the background color
        .insert_resource(ClearColor(FILL_COLOR))
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(Level::default())
        .insert_resource(FireTimer::default())
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(ship::ShipPlugin)
        .add_plugin(asteroid::AsteroidPlugin)
        .add_state(GameState::GameOver)
        .add_system(bevy::window::close_on_esc)
        .add_startup_system(spawn_camera)
        .add_system_set(
            SystemSet::on_enter(GameState::GameOver)
                .with_system(spawn_instructions)
                .with_system(instructions_input),
        )
        .add_system_set(
            SystemSet::on_update(GameState::GameOver)
                .with_system(instructions_input)
                .with_system(wrap_mover),
        )
        .add_system_set(SystemSet::on_enter(GameState::Playing).with_system(enter_playing))
        .add_system_set(
            SystemSet::on_update(GameState::Playing)
                .with_system(playing_input)
                .with_system(wrap_mover),
        )
        .run();
}

/// Setup the game camera
fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

/// Show the "Press Space to start" prompt when the game is launched
fn spawn_instructions(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(TextBundle {
            text: Text::from_section(
                "Press Space to start\nWhile playing, press R to reset",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 40.0,
                    color: Color::WHITE,
                },
            ),
            style: Style {
                align_self: AlignSelf::Center,
                margin: UiRect {
                    left: Val::Px(100.0),
                    ..default()
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(components::Instructions);
}

// when the game starts (by the GameState changing), removing the instructinos
fn enter_playing(
    mut commands: Commands,
    mut level: ResMut<Level>,
    query: Query<Entity, With<components::Instructions>>,
) {
    // reset level
    level.0 = 1;

    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// enter "Playing" state when user presses space
fn instructions_input(
    mut state: ResMut<State<GameState>>,
    mut keyboard_input: ResMut<Input<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        state.set(GameState::Playing).unwrap();
        keyboard_input.reset(KeyCode::Space);
    }
}

/// while in the "Playing" state, reset the game when user presses R
fn playing_input(mut state: ResMut<State<GameState>>, mut keyboard_input: ResMut<Input<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::R) {
        state.set(GameState::GameOver).unwrap();
        keyboard_input.reset(KeyCode::R);
    }
}

/// Wrap the ship/asteroids around the screen when it goes off the edge
fn wrap_mover(
    windows: Res<Windows>,
    mut query: Query<(&Dimensions, &mut Transform), With<Wrapper>>,
) {
    for (dimensions, mut transform) in query.iter_mut() {
        let window = windows.get_primary().unwrap();
        // World coordinates are in the center, so we get half the window dimensions
        // to figure out the actual edges of the screen
        let window_width = window.width() / 2.0;
        let window_height = window.height() / 2.0;

        // get current position from transform
        let mut position = transform.translation;
        let x_offset = dimensions.width / 2.0;
        let y_offset = dimensions.height / 2.0;

        if position.x > window_width + x_offset {
            position.x = -window_width - x_offset;
        } else if position.x < -window_width - x_offset {
            position.x = window_width + x_offset;
        }

        if position.y > window_height + y_offset {
            position.y = -window_height - y_offset;
        } else if position.y < -window_height - y_offset {
            position.y = window_height + y_offset;
        }

        transform.translation = position;
    }
}
