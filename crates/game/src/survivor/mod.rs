pub mod camera_follow;
pub mod enemy;
pub mod player;
pub mod spawn;
pub mod world_setup;

pub use camera_follow::CameraFollowSystem;
pub use enemy::{EnemyAi, EnemyAiSystem, Zombie};
pub use player::{Player, PlayerMovementSystem, PlayerStats, Velocity};
pub use spawn::{spawn_zombie, EnemySpawnSystem, SpawnTimer};
pub use world_setup::spawn_player;

/// 플레이어 충돌 레이어 비트마스크
pub const LAYER_PLAYER: u32 = 1 << 0;
/// 적 충돌 레이어 비트마스크
pub const LAYER_ENEMY: u32 = 1 << 1;
