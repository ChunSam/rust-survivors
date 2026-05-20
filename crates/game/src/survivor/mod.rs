pub mod camera_follow;
pub mod player;
pub mod world_setup;

pub use camera_follow::CameraFollowSystem;
pub use player::{Player, PlayerMovementSystem, PlayerStats, Velocity};
pub use world_setup::spawn_player;
