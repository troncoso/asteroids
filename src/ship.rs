use crate::{
    components::*,
    resources::{FireTimer, FILL_COLOR},
};
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::{FillMode, *};
use bevy_rapier2d::prelude::*;

pub struct ShipPlugin;

impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(build_enter_system_set(GameState::GameOver))
            .add_system_set(build_enter_system_set(GameState::Playing))
            .add_system_set(build_update_system_set(GameState::GameOver))
            .add_system_set(build_update_system_set(GameState::Playing));
    }
}

fn build_enter_system_set(state: GameState) -> SystemSet {
    SystemSet::on_enter(state).with_system(spawn_ship)
}

fn build_update_system_set(state: GameState) -> SystemSet {
    let system_set = SystemSet::on_update(state)
        .with_system(ship_rotate_input)
        .with_system(ship_move_forward);
    if state != GameState::Playing {
        return system_set;
    }
    return system_set
        .with_system(check_for_collision)
        .with_system(spawn_laser)
        .with_system(destroy_laser);
}

fn spawn_ship(mut commands: Commands, mut query: Query<Entity, With<Ship>>) {
    // Remove ship if it already eists
    for entity in query.iter_mut() {
        commands.entity(entity).despawn();
    }

    let shape_points = [
        Vec2::new(-15.0, -22.5),
        Vec2::new(0.0, 22.5),
        Vec2::new(15.0, -22.5),
    ];

    // This is the actual ship graphics
    let triangle = shapes::Polygon {
        points: shape_points.to_vec(),
        closed: true,
        ..Default::default()
    };

    commands
        .spawn(GeometryBuilder::build_as(
            &triangle,
            DrawMode::Outlined {
                fill_mode: FillMode {
                    color: FILL_COLOR,
                    options: FillOptions::default(),
                },
                outline_mode: StrokeMode {
                    color: Color::WHITE,
                    options: StrokeOptions::default(),
                },
            },
            Transform::default(),
        ))
        .insert(RigidBody::Dynamic)
        .insert(GravityScale(0.0))
        .insert(Collider::triangle(
            shape_points[0],
            shape_points[1],
            shape_points[2],
        ))
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(Sensor)
        .insert(ExternalImpulse {
            impulse: Vec2::new(0.0, 0.0),
            ..default()
        })
        .insert(Damping {
            linear_damping: 0.5,
            ..default()
        })
        .insert(Sleeping::disabled())
        // .insert(LockedAxes::TRANSLATION_LOCKED)
        .insert(Ccd::enabled())
        .insert(Wrapper)
        .insert(Dimensions {
            width: 30.0,
            height: 45.0,
        })
        .insert(Ship);
}

/** Updaters */

fn ship_rotate_input(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Ship>>,
) {
    for mut transform in query.iter_mut() {
        let mut rotation = 0.0;
        if keyboard_input.pressed(KeyCode::A) {
            rotation += 0.1;
        }
        if keyboard_input.pressed(KeyCode::D) {
            rotation -= 0.1;
        }

        // Apply rotation and save angle to resource
        transform.rotate(Quat::from_rotation_z(rotation));
    }
}

fn ship_move_forward(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut ExternalImpulse, &mut Transform), With<Ship>>,
) {
    for (mut ext_impulse, transform) in query.iter_mut() {
        if keyboard_input.pressed(KeyCode::W) {
            // Add velocity in the direction the ship is facing
            let up = transform.up();
            ext_impulse.impulse = Vec2::new(up.x, up.y) * 10000.0;
        }
    }
}

/// Create a laser entity and send it in the direction the ship is facing
fn spawn_laser(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut fire_timer: ResMut<FireTimer>,
    mut query: Query<(&Transform, &Dimensions), With<Ship>>,
) {
    fire_timer.0.tick(time.delta());
    if !keyboard_input.pressed(KeyCode::Space) {
        return;
    }
    if !fire_timer.0.paused() && fire_timer.0.elapsed().as_secs_f32() < 0.2 {
        return;
    }

    for (transform, dimensions) in query.iter_mut() {
        let up = transform.up();
        let position = transform.translation + up * dimensions.height;

        let shape_points = [
            Vec2::new(-1.0, -5.0),
            Vec2::new(1.0, -5.0),
            Vec2::new(1.0, 5.0),
            Vec2::new(-1.0, 5.0),
        ];

        // This is the actual ship graphics
        let rectangle = shapes::Polygon {
            points: shape_points.to_vec(),
            closed: true,
            ..Default::default()
        };

        commands
            .spawn(GeometryBuilder::build_as(
                &rectangle,
                DrawMode::Outlined {
                    fill_mode: FillMode {
                        color: FILL_COLOR,
                        options: FillOptions::default(),
                    },
                    outline_mode: StrokeMode {
                        color: Color::WHITE,
                        options: StrokeOptions::default(),
                    },
                },
                Transform {
                    translation: position,
                    rotation: transform.rotation,
                    ..Default::default()
                },
            ))
            .insert(RigidBody::Dynamic)
            .insert(GravityScale(0.0))
            .insert(Collider::polyline(shape_points.to_vec(), None))
            .insert(ActiveEvents::COLLISION_EVENTS)
            .insert(Sensor)
            .insert(Velocity {
                linvel: Vec2::new(up.x, up.y) * 1000.0,
                ..default()
            })
            .insert(Ccd::enabled())
            .insert(Sleeping::disabled())
            .insert(Laser);
    }

    fire_timer.0.reset();
}

/// remove laser when if goes off screen
fn destroy_laser(
    mut commands: Commands,
    windows: Res<Windows>,
    query: Query<(Entity, &Transform, &Dimensions), With<Laser>>,
) {
    let window = windows.get_primary().unwrap();
    let window_width = window.width() as f32;
    let window_height = window.height() as f32;

    for (entity, transform, dimensions) in query.iter() {
        let position = transform.translation;
        if position.x < -dimensions.width
            || position.x > window_width + dimensions.width
            || position.y < -dimensions.height
            || position.y > window_height + dimensions.height
        {
            commands.entity(entity).despawn();
        }
    }
}

// Check if the ship is making contact with an asteroid
fn check_for_collision(
    mut ship: Query<Entity, With<Ship>>,
    asteroids: Query<Entity, With<Asteroid>>,
    mut state: ResMut<State<GameState>>,
    mut collision_events: EventReader<CollisionEvent>,
) {
    for event in collision_events.iter() {
        if let CollisionEvent::Started(h1, h2, _flags) = event {
            for player in ship.iter_mut() {
                for asteroid in asteroids.iter() {
                    if (h1.to_owned() == player && h2.to_owned() == asteroid)
                        || (h1.to_owned() == asteroid && h2.to_owned() == player)
                    {
                        // the game is over
                        state.set(GameState::GameOver).unwrap();
                    }
                }
            }
        }
    }
}
