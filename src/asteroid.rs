use crate::components::*;
use crate::resources::*;
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::{FillMode, *};
use bevy_rapier2d::prelude::*;
use rand::rngs::ThreadRng;
use rand::*;

pub struct AsteroidPlugin;

impl Plugin for AsteroidPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::Playing).with_system(spawn_asteroids))
            .add_system_set(
                SystemSet::on_update(GameState::Playing).with_system(check_for_laser_collision),
            )
            .add_system_set(SystemSet::on_exit(GameState::Playing).with_system(remove_asteroids));
    }
}

fn spawn_asteroids(mut commands: Commands, level: ResMut<Level>) {
    // determine a random starting velocity
    let mut rng = rand::thread_rng();
    let count = 1 + level.0;

    for _ in 0..count {
        create_asteroid(&mut commands, &mut rng, AsteroidSize::Large, None);
    }
}

fn create_asteroid(
    commands: &mut Commands,
    rng: &mut ThreadRng,
    size: AsteroidSize,
    position: Option<Vec3>,
) {
    let vx = rng.gen_range(-1.0..1.0);
    let vy = rng.gen_range(-1.0..1.0);

    // determine a random starting position outside the screen
    let x = if rng.gen_bool(0.5) {
        rng.gen_range(-1000.0..-100.0)
    } else {
        rng.gen_range(100.0..1000.0)
    };

    let y = if rng.gen_bool(0.5) {
        rng.gen_range(-1000.0..-100.0)
    } else {
        rng.gen_range(100.0..1000.0)
    };

    let dimension = match size {
        AsteroidSize::Large => 80.0,
        AsteroidSize::Medium => 60.0,
        AsteroidSize::Small => 20.0,
        AsteroidSize::Dead => 0.0,
    };
    // determine origination position
    let pos = match position {
        Some(pos) => pos,
        None => Vec3::new(x, y, 0.0),
    };

    let octogon = create_shape(dimension);
    commands
        .spawn(GeometryBuilder::build_as(
            &octogon,
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
                translation: pos,
                ..default()
            },
        ))
        .insert(RigidBody::Dynamic)
        .insert(GravityScale(0.0))
        .insert(Collider::polyline(octogon.points, None))
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(Sensor)
        .insert(Velocity {
            linvel: Vec2::new(vx * 100.0, vy * 100.0),
            angvel: 0.4,
        })
        .insert(Sleeping::disabled())
        .insert(Ccd::enabled())
        .insert(Wrapper)
        .insert(Dimensions {
            width: dimension,
            height: dimension,
        })
        .insert(Asteroid(size));
}

fn create_shape(radius: f32) -> shapes::Polygon {
    // create octogon shape
    let mut shape_points = Vec::new();
    let mut angle: f32 = 0.0;
    for _ in 0..8 {
        let x = angle.cos() * radius;
        let y = angle.sin() * radius;
        shape_points.push(Vec2::new(x, y));
        angle += std::f32::consts::PI / 4.0;
    }

    shapes::Polygon {
        points: shape_points.to_vec(),
        closed: true,
        ..Default::default()
    }
}

fn remove_asteroids(mut commands: Commands, query: Query<Entity, With<Asteroid>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn check_for_laser_collision(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    mut level: ResMut<Level>,
    laser_query: Query<Entity, With<Laser>>,
    asteroid_query: Query<(Entity, &Asteroid, &Transform)>,
) {
    // It is possible for a single laser to collide with multiple asteroids at once.
    // We need to keep track of each so we don't try removing it multiple times
    let mut processed_lasers = Vec::new();
    for event in collision_events.iter() {
        if let CollisionEvent::Started(h1, h2, _flags) = event {
            for (asteroid_entity, asteroid, transform) in asteroid_query.iter() {
                for laser in laser_query.iter() {
                    if (h1.to_owned() == asteroid_entity && h2.to_owned() == laser)
                        || (h1.to_owned() == laser && h2.to_owned() == asteroid_entity)
                    {
                        if processed_lasers.contains(&laser) {
                            continue;
                        }

                        commands.entity(laser).despawn();
                        commands.entity(asteroid_entity).despawn();

                        processed_lasers.push(laser);

                        // spawn 4 smaller asteroids that fly in different directions
                        let size = match asteroid.0 {
                            AsteroidSize::Large => AsteroidSize::Medium,
                            AsteroidSize::Medium => AsteroidSize::Small,
                            AsteroidSize::Small => AsteroidSize::Dead,
                            AsteroidSize::Dead => AsteroidSize::Dead,
                        };

                        let mut rng = rand::thread_rng();
                        if size != AsteroidSize::Dead {
                            // get destroyed asteroid's position
                            let position = transform.translation;
                            for _ in 0..4 {
                                create_asteroid(&mut commands, &mut rng, size, Some(position));
                            }
                        } else {
                            // get total asteroid count
                            let mut count = 0;
                            for _ in asteroid_query.iter() {
                                count += 1;
                            }

                            // we check for one because the one we just destroyed will still be in
                            // the list
                            if count == 1 {
                                level.0 += 1;

                                let count = 1 + level.0;

                                for _ in 0..count {
                                    create_asteroid(
                                        &mut commands,
                                        &mut rng,
                                        AsteroidSize::Large,
                                        None,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
