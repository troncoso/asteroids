use bevy::prelude::Component;

/// enum for game states
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum GameState {
    Playing,
    GameOver,
}

/// enum for asteroid sizes
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum AsteroidSize {
    Large,
    Medium,
    Small,
    Dead,
}

#[derive(Component)]
pub struct Instructions;

#[derive(Component)]
pub struct Ship;

#[derive(Component)]
pub struct Laser;

#[derive(Component)]
pub struct Asteroid(pub AsteroidSize);

/// any entity that should "wrap" when hitting the edge of the screen
#[derive(Component)]
pub struct Wrapper;

/// describes an entity with a width and height
#[derive(Component, Clone, Copy, PartialEq)]
pub struct Dimensions {
    pub width: f32,
    pub height: f32,
}
