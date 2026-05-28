use engine::{Camera, GameState, World};
use glam::Vec2;

use super::boss::{CameraShake, StageProgress};
use super::hud::GameStats;

pub(crate) fn playing_world() -> World {
    let mut world = World::new();
    world.insert_resource(GameState::Playing);
    world
}

pub(crate) fn playing_world_with_camera() -> World {
    let mut world = playing_world();
    world.insert_resource(Camera::new(Vec2::ZERO, 1.0));
    world
}

pub(crate) fn playing_world_with_boss_resources() -> World {
    let mut world = playing_world_with_camera();
    world.insert_resource(StageProgress::default());
    world.insert_resource(GameStats::default());
    world.insert_resource(CameraShake::default());
    world
}
