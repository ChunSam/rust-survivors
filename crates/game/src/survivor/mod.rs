pub mod achievement;
pub mod area;
pub mod background;
pub mod bgm;
pub mod bible;
pub mod boss;
pub mod camera_follow;
pub mod character;
pub mod chest;
pub(crate) mod combat;
pub mod damage;
pub mod damage_number;
pub mod data;
pub mod death;
pub mod debug_input;
pub mod director;
pub mod enemy;
pub mod health;
pub mod hud;
pub mod icons;
pub mod inventory;
pub mod levelup;
pub mod lightning;
pub mod locale;
pub mod meta;
pub mod particle;
pub mod passive;
pub mod pickup;
pub mod player;
pub mod powerup;
pub mod projectile;
pub mod sfx;
pub mod spawn;
pub mod sprites;
pub mod stage;
pub mod stats;
#[cfg(test)]
pub(crate) mod test_support;
pub mod title_visual;
pub mod ui_icons;
pub(crate) mod ui_layout;
pub mod weapon;
pub mod world_setup;
pub mod xp;

pub use achievement::{
    achievement_completed, unlock_achievement, AchievementKind, AchievementSystem,
};
pub use area::{
    spawn_holy_water_pool, GarlicSystem, HolyWaterPool, HolyWaterPoolSystem, HolyWaterSystem,
};
pub use background::BackgroundSystem;
pub use bgm::BgmSystem;
pub use bible::{spawn_book, KingBibleSystem, OrbitingBook, OrbitingBookSystem};
pub use boss::{
    spawn_boss, Boss, BossDeathSystem, BossKind, BossPhaseSystem, BossSpawnQueue, BossSpawnSystem,
    CameraShake, StageProgress,
};
pub use camera_follow::CameraFollowSystem;
pub use character::{CharacterCursor, CharacterKind, CharacterSelectSystem, SelectedCharacter};
pub use chest::{
    spawn_chest, try_evolve, Chest, ChestPickupSystem, EvolutionRule, EVOLUTION_RULES,
};
pub use damage::apply_damage_to_enemy;
pub use damage_number::{spawn_damage_number, DamageNumber, DamageNumberSystem};
pub use data::load_starter_weapons;
pub use death::{
    reset_to_title_world, restart_world, DeathSystem, EnemyContactDamageSystem, RestartSystem,
};
pub use debug_input::{DebugInputSystem, DebugOverlay};
pub use director::{
    spawn_pattern, EnemyEntry, SpawnDirector, SpawnDirectorSystem, SpawnPattern, WaveDef, WavesFile,
};
pub use enemy::{Enemy, EnemyAi, EnemyAiKind, EnemyAiSystem, EnemyKind, EnemyStats, Zombie};
pub use health::{Health, HealthRegenSystem};
pub use hud::{GameStats, HudSystem};
pub use icons::{weapon_icon, IconFrame, UiIcon, ICONS_PATH, ICON_COLS, ICON_ROWS, ICON_SIZE};
pub use inventory::{WeaponInventory, WeaponKind, WeaponSlot};
pub use levelup::{CardKind, LevelUpSystem, PendingLevelUp};
pub use lightning::{
    spawn_lightning_flash, LightningFlash, LightningFlashSystem, LightningRingSystem,
};
pub use locale::{loc, Lang};
pub use meta::{
    AchievementCursor, HudDetail, LanguageSetting, MetaSave, ModeTransitionSystem, PauseMenuCursor,
    ResolutionPreset, SettingsCursor, SurvivorMode, ACHIEVEMENTS_PER_PAGE, SETTINGS_ITEMS,
};
pub use particle::{spawn_collect_burst, spawn_death_burst, ParticleSystem};
pub use passive::{PassiveInventory, PassiveKind, PassiveSlot};
pub use pickup::{
    apply_pickup_effect, drop_boss_pickups, spawn_pickup, try_drop_normal_pickup, GoldWallet,
    Pickup, PickupKind, PickupSystem,
};
pub use player::{Player, PlayerMovementSystem, PlayerStats, Velocity};
pub use powerup::{apply_powerups, try_purchase, PowerUpKind, ShopCursor, ShopInputSystem};
pub use projectile::{
    spawn_projectile, spawn_projectile_ex, Projectile, ProjectileBehavior, ProjectileSystem,
};
pub use sfx::{SfxEvent, SfxQueue, SfxSystem};
pub use spawn::{
    spawn_enemy, spawn_enemy_elite, spawn_enemy_full, spawn_enemy_with_splits, spawn_zombie,
};
pub use sprites::{
    add_sprite, add_tinted_sprite, survivor_textured_sprite, ActorFacingSystem, SurvivorSprite,
    SurvivorTextureHandles, ACTOR_FRAMES_PATH, ATLAS_PATH, BOSS_VISUAL_SCALE,
    DAIRY_PLANT_TILES_PATH, EFFECTS_PATH, ENEMY_VISUAL_SCALE, EVOLUTIONS_PATH,
    GAMEOVER_TITLE_EN_PATH, GAMEOVER_TITLE_KO_PATH, INGAME_LABEL_GOLD_EN_PATH,
    INGAME_LABEL_GOLD_KO_PATH, INGAME_LABEL_HP_EN_PATH, INGAME_LABEL_HP_KO_PATH,
    INGAME_LABEL_KILLS_EN_PATH, INGAME_LABEL_KILLS_KO_PATH, INGAME_LABEL_LV_EN_PATH,
    INGAME_LABEL_LV_KO_PATH, INGAME_LABEL_PASSIVES_EN_PATH, INGAME_LABEL_PASSIVES_KO_PATH,
    INGAME_LABEL_XP_EN_PATH, INGAME_LABEL_XP_KO_PATH, INLAID_LIBRARY_TILES_PATH,
    LEVELUP_TITLE_EN_PATH, LEVELUP_TITLE_KO_PATH, MAD_FOREST_TILES_PATH,
    MENU_BUTTON_ACHIEVEMENTS_EN_PATH, MENU_BUTTON_ACHIEVEMENTS_KO_PATH,
    MENU_BUTTON_CHARACTER_EN_PATH, MENU_BUTTON_CHARACTER_KO_PATH, MENU_BUTTON_CHARACTER_PATH,
    MENU_BUTTON_SETTINGS_EN_PATH, MENU_BUTTON_SETTINGS_KO_PATH, MENU_BUTTON_SETTINGS_PATH,
    MENU_BUTTON_SHOP_EN_PATH, MENU_BUTTON_SHOP_KO_PATH, MENU_BUTTON_SHOP_PATH,
    MENU_BUTTON_STAGE_EN_PATH, MENU_BUTTON_STAGE_KO_PATH, MENU_BUTTON_STAGE_PATH,
    MENU_BUTTON_START_EN_PATH, MENU_BUTTON_START_KO_PATH, MENU_BUTTON_START_PATH, PASSIVES_PATH,
    PLAYER_VISUAL_SIZE, POWERUPS_PATH, RENDER_LAYER_BACKGROUND, RENDER_LAYER_EFFECTS,
    RENDER_LAYER_UI, RENDER_LAYER_WORLD, RESTART_HINT_EN_PATH, RESTART_HINT_KO_PATH,
    SECTION_PASSIVES_EN_PATH, SECTION_PASSIVES_KO_PATH, SECTION_WEAPONS_EN_PATH,
    SECTION_WEAPONS_KO_PATH, TITLE_BACKDROP_PATH, TITLE_LOGO_PLAQUE_PATH, UI_MODAL_PANEL_PATH,
    UI_SLOT_FRAME_PATH,
};
pub use stage::{SelectedStage, StageCursor, StageKind, StageSelectSystem};
pub use stats::{read_player_stats, StatRecalcSystem};
pub use title_visual::{TitleBackdrop, TitleVisualSystem};
pub use ui_icons::UiIconSystem;
pub use weapon::{
    AxeSystem, CrossSystem, FireWandSystem, HitFlash, HitFlashSystem, KnifeSystem, MagicWandSystem,
    WhipSystem, FLASH_DURATION, SCALE_BUMP,
};
pub use world_setup::{setup_survivor_world, spawn_player};
pub use xp::{spawn_xp_gem, MagnetSystem, XpAccumulator, XpGem};

/// 플레이어 충돌 레이어 비트마스크
pub const LAYER_PLAYER: u32 = 1 << 0;
/// 적 충돌 레이어 비트마스크
pub const LAYER_ENEMY: u32 = 1 << 1;
/// XP 보석 충돌 레이어 비트마스크
pub const LAYER_XP: u32 = 1 << 2;
