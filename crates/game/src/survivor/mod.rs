pub mod area;
pub mod bible;
pub mod debug_input;
pub mod boss;
pub mod locale;
pub mod character;
pub mod chest;
pub mod data;
pub mod camera_follow;
pub mod damage;
pub mod damage_number;
pub mod death;
pub mod particle;
pub mod director;
pub mod enemy;
pub mod health;
pub mod hud;
pub mod inventory;
pub mod levelup;
pub mod lightning;
pub mod meta;
pub mod passive;
pub mod pickup;
pub mod powerup;
pub mod player;
pub mod projectile;
pub mod sfx;
pub mod spawn;
pub mod stage;
pub mod stats;
pub mod weapon;
pub mod world_setup;
pub mod xp;

pub use area::{GarlicSystem, HolyWaterPool, HolyWaterPoolSystem, HolyWaterSystem, spawn_holy_water_pool};
pub use character::{CharacterCursor, CharacterKind, CharacterSelectSystem, SelectedCharacter};
pub use stage::{SelectedStage, StageCursor, StageKind, StageSelectSystem};
pub use locale::{loc, Lang};
pub use meta::{MetaSave, ModeTransitionSystem, SurvivorMode};
pub use powerup::{apply_powerups, try_purchase, PowerUpKind, ShopCursor, ShopInputSystem};
pub use chest::{Chest, ChestPickupSystem, EvolutionRule, EVOLUTION_RULES, try_evolve, spawn_chest};
pub use pickup::{
    apply_pickup_effect, drop_boss_pickups, spawn_pickup, try_drop_normal_pickup,
    GoldWallet, Pickup, PickupKind, PickupSystem,
};
pub use boss::{
    spawn_boss, Boss, BossDeathSystem, BossKind, BossPhaseSystem, BossSpawnQueue,
    BossSpawnSystem, CameraShake, StageProgress,
};
pub use data::load_starter_weapons;
pub use bible::{KingBibleSystem, OrbitingBook, OrbitingBookSystem, spawn_book};
pub use camera_follow::CameraFollowSystem;
pub use damage::apply_damage_to_enemy;
pub use damage_number::{spawn_damage_number, DamageNumber, DamageNumberSystem};
pub use death::{restart_world, DeathSystem, EnemyContactDamageSystem, RestartSystem};
pub use director::{spawn_pattern, EnemyEntry, SpawnDirector, SpawnDirectorSystem, SpawnPattern, WaveDef, WavesFile};
pub use enemy::{Enemy, EnemyAi, EnemyAiKind, EnemyAiSystem, EnemyKind, EnemyStats, Zombie};
pub use health::{Health, HealthRegenSystem};
pub use hud::{GameStats, HudSystem};
pub use inventory::{WeaponInventory, WeaponKind, WeaponSlot};
pub use levelup::{CardKind, LevelUpSystem, PendingLevelUp};
pub use lightning::{LightningFlash, LightningFlashSystem, LightningRingSystem, spawn_lightning_flash};
pub use passive::{PassiveInventory, PassiveKind, PassiveSlot};
pub use player::{Player, PlayerMovementSystem, PlayerStats, Velocity};
pub use projectile::{spawn_projectile, spawn_projectile_ex, Projectile, ProjectileBehavior, ProjectileSystem};
pub use spawn::{spawn_enemy, spawn_enemy_elite, spawn_enemy_full, spawn_enemy_with_splits, spawn_zombie};
pub use particle::{ParticleSystem, spawn_death_burst, spawn_collect_burst};
pub use sfx::{SfxEvent, SfxQueue, SfxSystem};
pub use stats::{read_player_stats, StatRecalcSystem};
pub use weapon::{AxeSystem, CrossSystem, FireWandSystem, HitFlash, HitFlashSystem, KnifeSystem, MagicWandSystem, WhipSystem, FLASH_DURATION, SCALE_BUMP};
pub use debug_input::DebugInputSystem;
pub use world_setup::{setup_survivor_world, spawn_player};
pub use xp::{spawn_xp_gem, MagnetSystem, XpAccumulator, XpGem};

/// 플레이어 충돌 레이어 비트마스크
pub const LAYER_PLAYER: u32 = 1 << 0;
/// 적 충돌 레이어 비트마스크
pub const LAYER_ENEMY: u32 = 1 << 1;
/// XP 보석 충돌 레이어 비트마스크
pub const LAYER_XP: u32 = 1 << 2;
