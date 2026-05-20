pub mod camera_follow;
pub mod enemy;
pub mod health;
pub mod player;
pub mod spawn;
pub mod weapon;
pub mod world_setup;
pub mod xp;

pub use camera_follow::CameraFollowSystem;
pub use enemy::{EnemyAi, EnemyAiSystem, Zombie};
pub use health::Health;
pub use player::{Player, PlayerMovementSystem, PlayerStats, Velocity};
pub use spawn::{spawn_zombie, EnemySpawnSystem, SpawnTimer};
pub use weapon::{HitFlash, HitFlashSystem, Whip, WhipSystem};
pub use world_setup::spawn_player;
pub use xp::{spawn_xp_gem, MagnetSystem, XpAccumulator, XpGem};

/// 플레이어 충돌 레이어 비트마스크
pub const LAYER_PLAYER: u32 = 1 << 0;
/// 적 충돌 레이어 비트마스크
pub const LAYER_ENEMY: u32 = 1 << 1;
/// XP 보석 충돌 레이어 비트마스크
pub const LAYER_XP: u32 = 1 << 2;
