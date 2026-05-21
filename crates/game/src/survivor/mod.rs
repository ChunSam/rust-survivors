pub mod area;
pub mod bible;
pub mod data;
pub mod camera_follow;
pub mod damage;
pub mod death;
pub mod enemy;
pub mod health;
pub mod hud;
pub mod inventory;
pub mod levelup;
pub mod lightning;
pub mod passive;
pub mod player;
pub mod projectile;
pub mod spawn;
pub mod stats;
pub mod weapon;
pub mod world_setup;
pub mod xp;

pub use area::{GarlicSystem, HolyWaterPool, HolyWaterPoolSystem, HolyWaterSystem, spawn_holy_water_pool};
pub use data::load_starter_weapons;
pub use bible::{KingBibleSystem, OrbitingBook, OrbitingBookSystem, spawn_book};
pub use camera_follow::CameraFollowSystem;
pub use damage::apply_damage_to_enemy;
pub use death::{restart_world, DeathSystem, EnemyContactDamageSystem, RestartSystem};
pub use enemy::{Enemy, EnemyAi, EnemyAiKind, EnemyAiSystem, EnemyKind, EnemyStats, Zombie};
pub use health::{Health, HealthRegenSystem};
pub use hud::{GameStats, HudSystem};
pub use inventory::{WeaponInventory, WeaponKind, WeaponSlot};
pub use levelup::{CardKind, LevelUpSystem, PendingLevelUp};
pub use lightning::{LightningFlash, LightningFlashSystem, LightningRingSystem, spawn_lightning_flash};
pub use passive::{PassiveInventory, PassiveKind, PassiveSlot};
pub use player::{Player, PlayerMovementSystem, PlayerStats, Velocity};
pub use projectile::{spawn_projectile, spawn_projectile_ex, Projectile, ProjectileBehavior, ProjectileSystem};
pub use spawn::{spawn_enemy, spawn_zombie, EnemySpawnSystem, SpawnTimer};
pub use stats::{read_player_stats, StatRecalcSystem};
pub use weapon::{AxeSystem, CrossSystem, FireWandSystem, HitFlash, HitFlashSystem, KnifeSystem, MagicWandSystem, WhipSystem};
pub use world_setup::{setup_survivor_world, spawn_player};
pub use xp::{spawn_xp_gem, MagnetSystem, XpAccumulator, XpGem};

/// 플레이어 충돌 레이어 비트마스크
pub const LAYER_PLAYER: u32 = 1 << 0;
/// 적 충돌 레이어 비트마스크
pub const LAYER_ENEMY: u32 = 1 << 1;
/// XP 보석 충돌 레이어 비트마스크
pub const LAYER_XP: u32 = 1 << 2;
