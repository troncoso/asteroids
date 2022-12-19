use bevy::{
    prelude::{Color, Resource},
    time::Stopwatch,
};

pub const FILL_COLOR: Color = Color::rgb(0.04, 0.04, 0.04);

/// a timer that starts each time a laser is fired. Another laser cannot be fired
/// until .2 seconds have lapsed
#[derive(Resource)]
pub struct FireTimer(pub Stopwatch);

impl Default for FireTimer {
    fn default() -> Self {
        Self(Stopwatch::new())
    }
}

/// The current level of the game. This determines how many asteroids will spawn
#[derive(Resource)]
pub struct Level(pub u32);

impl Default for Level {
    fn default() -> Self {
        Level(1)
    }
}
