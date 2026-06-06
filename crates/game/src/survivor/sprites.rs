use engine::{
    AnimationClip, AnimationPlayer, Entity, Handle, ImageAsset, RenderLayer, Sprite, Transform,
    UvRect, World,
};

use super::icons::ICONS_PATH;

/// Shared generated sprite atlas for the survivor mode.
pub const ATLAS_PATH: &str = "assets/textures/survivor/survivor_atlas.png";
pub const EFFECTS_PATH: &str = "assets/textures/survivor/survivor_effects.png";
pub const ACTOR_FRAMES_PATH: &str = "assets/textures/survivor/survivor_actor_frames.png";
pub const EVOLUTIONS_PATH: &str = "assets/textures/survivor/survivor_evolutions.png";
pub const PASSIVES_PATH: &str = "assets/textures/survivor/survivor_passives.png";
pub const POWERUPS_PATH: &str = "assets/textures/survivor/survivor_powerups.png";
pub const TITLE_BACKDROP_PATH: &str = "assets/textures/survivor/menu/title_backdrop_v2.png";
pub const TITLE_LOGO_PLAQUE_PATH: &str = "assets/textures/survivor/menu/title_logo_plaque_v3.png";
pub const MENU_BUTTON_START_KO_PATH: &str =
    "assets/textures/survivor/menu/menu_button_start_ko.png";
pub const MENU_BUTTON_START_EN_PATH: &str =
    "assets/textures/survivor/menu/menu_button_start_en.png";
pub const MENU_BUTTON_CHARACTER_KO_PATH: &str =
    "assets/textures/survivor/menu/menu_button_character_ko.png";
pub const MENU_BUTTON_CHARACTER_EN_PATH: &str =
    "assets/textures/survivor/menu/menu_button_character_en.png";
pub const MENU_BUTTON_STAGE_KO_PATH: &str =
    "assets/textures/survivor/menu/menu_button_stage_ko.png";
pub const MENU_BUTTON_STAGE_EN_PATH: &str =
    "assets/textures/survivor/menu/menu_button_stage_en.png";
pub const MENU_BUTTON_SHOP_KO_PATH: &str = "assets/textures/survivor/menu/menu_button_shop_ko.png";
pub const MENU_BUTTON_SHOP_EN_PATH: &str = "assets/textures/survivor/menu/menu_button_shop_en.png";
pub const MENU_BUTTON_ACHIEVEMENTS_KO_PATH: &str =
    "assets/textures/survivor/menu/menu_button_achievements_ko.png";
pub const MENU_BUTTON_ACHIEVEMENTS_EN_PATH: &str =
    "assets/textures/survivor/menu/menu_button_achievements_en.png";
pub const MENU_BUTTON_SETTINGS_KO_PATH: &str =
    "assets/textures/survivor/menu/menu_button_settings_ko.png";
pub const MENU_BUTTON_SETTINGS_EN_PATH: &str =
    "assets/textures/survivor/menu/menu_button_settings_en.png";
pub const MENU_BUTTON_START_PATH: &str = MENU_BUTTON_START_EN_PATH;
pub const MENU_BUTTON_CHARACTER_PATH: &str = MENU_BUTTON_CHARACTER_EN_PATH;
pub const MENU_BUTTON_STAGE_PATH: &str = MENU_BUTTON_STAGE_EN_PATH;
pub const MENU_BUTTON_SHOP_PATH: &str = MENU_BUTTON_SHOP_EN_PATH;
pub const MENU_BUTTON_SETTINGS_PATH: &str = MENU_BUTTON_SETTINGS_EN_PATH;
pub const UI_MODAL_PANEL_PATH: &str = "assets/textures/survivor/ui/ui_modal_panel.png";
pub const UI_SLOT_FRAME_PATH: &str = "assets/textures/survivor/ui/ui_slot_frame.png";
pub const INGAME_LABEL_LV_KO_PATH: &str = "assets/textures/survivor/ui/ingame_label_lv_ko.png";
pub const INGAME_LABEL_LV_EN_PATH: &str = "assets/textures/survivor/ui/ingame_label_lv_en.png";
pub const INGAME_LABEL_HP_KO_PATH: &str = "assets/textures/survivor/ui/ingame_label_hp_ko.png";
pub const INGAME_LABEL_HP_EN_PATH: &str = "assets/textures/survivor/ui/ingame_label_hp_en.png";
pub const INGAME_LABEL_XP_KO_PATH: &str = "assets/textures/survivor/ui/ingame_label_xp_ko.png";
pub const INGAME_LABEL_XP_EN_PATH: &str = "assets/textures/survivor/ui/ingame_label_xp_en.png";
pub const INGAME_LABEL_GOLD_KO_PATH: &str = "assets/textures/survivor/ui/ingame_label_gold_ko.png";
pub const INGAME_LABEL_GOLD_EN_PATH: &str = "assets/textures/survivor/ui/ingame_label_gold_en.png";
pub const INGAME_LABEL_KILLS_KO_PATH: &str =
    "assets/textures/survivor/ui/ingame_label_kills_ko.png";
pub const INGAME_LABEL_KILLS_EN_PATH: &str =
    "assets/textures/survivor/ui/ingame_label_kills_en.png";
pub const INGAME_LABEL_PASSIVES_KO_PATH: &str =
    "assets/textures/survivor/ui/ingame_label_passives_ko.png";
pub const INGAME_LABEL_PASSIVES_EN_PATH: &str =
    "assets/textures/survivor/ui/ingame_label_passives_en.png";
pub const LEVELUP_TITLE_KO_PATH: &str = "assets/textures/survivor/ui/levelup_title_ko.png";
pub const LEVELUP_TITLE_EN_PATH: &str = "assets/textures/survivor/ui/levelup_title_en.png";
pub const GAMEOVER_TITLE_KO_PATH: &str = "assets/textures/survivor/ui/gameover_title_ko.png";
pub const GAMEOVER_TITLE_EN_PATH: &str = "assets/textures/survivor/ui/gameover_title_en.png";
pub const RESTART_HINT_KO_PATH: &str = "assets/textures/survivor/ui/restart_hint_ko.png";
pub const RESTART_HINT_EN_PATH: &str = "assets/textures/survivor/ui/restart_hint_en.png";
pub const SECTION_WEAPONS_KO_PATH: &str = "assets/textures/survivor/ui/section_weapons_ko.png";
pub const SECTION_WEAPONS_EN_PATH: &str = "assets/textures/survivor/ui/section_weapons_en.png";
pub const SECTION_PASSIVES_KO_PATH: &str = "assets/textures/survivor/ui/section_passives_ko.png";
pub const SECTION_PASSIVES_EN_PATH: &str = "assets/textures/survivor/ui/section_passives_en.png";
pub const PLAYER_VISUAL_SIZE: f32 = 150.0;
pub const ENEMY_VISUAL_SCALE: f32 = 3.75;
pub const BOSS_VISUAL_SCALE: f32 = 1.5;
pub const RENDER_LAYER_BACKGROUND: i32 = -10;
pub const RENDER_LAYER_WORLD: i32 = 0;
pub const RENDER_LAYER_EFFECTS: i32 = 10;
pub const RENDER_LAYER_UI: i32 = 20;

const ATLAS_ROWS: u32 = 4;
const ATLAS_WIDTH: f32 = 1254.0;
const ATLAS_HEIGHT: f32 = 1254.0;
const ATLAS_SLOT_WIDTH: f32 = 209.0;
const EFFECTS_WIDTH: f32 = 1254.0;
const EFFECTS_HEIGHT: f32 = 418.0;
const EFFECTS_SLOT_SIZE: f32 = 209.0;
const ACTOR_FRAMES_COLS: u32 = 5;
const ACTOR_FRAMES_ROWS: u32 = 14;
const ACTOR_FRAMES_WIDTH: f32 = 960.0;
const ACTOR_FRAMES_HEIGHT: f32 = 3584.0;
const ACTOR_FRAME_WIDTH: f32 = ACTOR_FRAMES_WIDTH / ACTOR_FRAMES_COLS as f32;
const ACTOR_FRAME_HEIGHT: f32 = ACTOR_FRAMES_HEIGHT / ACTOR_FRAMES_ROWS as f32;
const ACTOR_MOVE_FRAME_COUNT: u32 = 3;
const ACTOR_HIT_FRAME_COL: u32 = 3;
const ACTOR_DEATH_FRAME_COL: u32 = 4;
const ACTOR_MOVE_FPS: f32 = 6.0;

#[derive(Clone, Debug)]
pub struct SurvivorTextureHandles {
    pub atlas: Handle<ImageAsset>,
    pub effects: Handle<ImageAsset>,
    pub actor_frames: Handle<ImageAsset>,
    pub evolutions: Handle<ImageAsset>,
    pub icons: Handle<ImageAsset>,
    pub passives: Handle<ImageAsset>,
    pub powerups: Handle<ImageAsset>,
    pub title_backdrop: Handle<ImageAsset>,
    pub title_logo_plaque: Handle<ImageAsset>,
    pub menu_button_start_ko: Handle<ImageAsset>,
    pub menu_button_start_en: Handle<ImageAsset>,
    pub menu_button_character_ko: Handle<ImageAsset>,
    pub menu_button_character_en: Handle<ImageAsset>,
    pub menu_button_stage_ko: Handle<ImageAsset>,
    pub menu_button_stage_en: Handle<ImageAsset>,
    pub menu_button_shop_ko: Handle<ImageAsset>,
    pub menu_button_shop_en: Handle<ImageAsset>,
    pub menu_button_achievements_ko: Handle<ImageAsset>,
    pub menu_button_achievements_en: Handle<ImageAsset>,
    pub menu_button_settings_ko: Handle<ImageAsset>,
    pub menu_button_settings_en: Handle<ImageAsset>,
    pub ui_modal_panel: Handle<ImageAsset>,
    pub ui_slot_frame: Handle<ImageAsset>,
    pub ingame_label_lv_ko: Handle<ImageAsset>,
    pub ingame_label_lv_en: Handle<ImageAsset>,
    pub ingame_label_hp_ko: Handle<ImageAsset>,
    pub ingame_label_hp_en: Handle<ImageAsset>,
    pub ingame_label_xp_ko: Handle<ImageAsset>,
    pub ingame_label_xp_en: Handle<ImageAsset>,
    pub ingame_label_gold_ko: Handle<ImageAsset>,
    pub ingame_label_gold_en: Handle<ImageAsset>,
    pub ingame_label_kills_ko: Handle<ImageAsset>,
    pub ingame_label_kills_en: Handle<ImageAsset>,
    pub ingame_label_passives_ko: Handle<ImageAsset>,
    pub ingame_label_passives_en: Handle<ImageAsset>,
    pub levelup_title_ko: Handle<ImageAsset>,
    pub levelup_title_en: Handle<ImageAsset>,
    pub gameover_title_ko: Handle<ImageAsset>,
    pub gameover_title_en: Handle<ImageAsset>,
    pub restart_hint_ko: Handle<ImageAsset>,
    pub restart_hint_en: Handle<ImageAsset>,
    pub section_weapons_ko: Handle<ImageAsset>,
    pub section_weapons_en: Handle<ImageAsset>,
    pub section_passives_ko: Handle<ImageAsset>,
    pub section_passives_en: Handle<ImageAsset>,
}

impl SurvivorTextureHandles {
    pub fn handle_for(&self, path: &str) -> Option<&Handle<ImageAsset>> {
        match path {
            ATLAS_PATH => Some(&self.atlas),
            EFFECTS_PATH => Some(&self.effects),
            ACTOR_FRAMES_PATH => Some(&self.actor_frames),
            EVOLUTIONS_PATH => Some(&self.evolutions),
            ICONS_PATH => Some(&self.icons),
            PASSIVES_PATH => Some(&self.passives),
            POWERUPS_PATH => Some(&self.powerups),
            TITLE_BACKDROP_PATH => Some(&self.title_backdrop),
            TITLE_LOGO_PLAQUE_PATH => Some(&self.title_logo_plaque),
            MENU_BUTTON_START_KO_PATH => Some(&self.menu_button_start_ko),
            MENU_BUTTON_START_EN_PATH => Some(&self.menu_button_start_en),
            MENU_BUTTON_CHARACTER_KO_PATH => Some(&self.menu_button_character_ko),
            MENU_BUTTON_CHARACTER_EN_PATH => Some(&self.menu_button_character_en),
            MENU_BUTTON_STAGE_KO_PATH => Some(&self.menu_button_stage_ko),
            MENU_BUTTON_STAGE_EN_PATH => Some(&self.menu_button_stage_en),
            MENU_BUTTON_SHOP_KO_PATH => Some(&self.menu_button_shop_ko),
            MENU_BUTTON_SHOP_EN_PATH => Some(&self.menu_button_shop_en),
            MENU_BUTTON_ACHIEVEMENTS_KO_PATH => Some(&self.menu_button_achievements_ko),
            MENU_BUTTON_ACHIEVEMENTS_EN_PATH => Some(&self.menu_button_achievements_en),
            MENU_BUTTON_SETTINGS_KO_PATH => Some(&self.menu_button_settings_ko),
            MENU_BUTTON_SETTINGS_EN_PATH => Some(&self.menu_button_settings_en),
            UI_MODAL_PANEL_PATH => Some(&self.ui_modal_panel),
            UI_SLOT_FRAME_PATH => Some(&self.ui_slot_frame),
            INGAME_LABEL_LV_KO_PATH => Some(&self.ingame_label_lv_ko),
            INGAME_LABEL_LV_EN_PATH => Some(&self.ingame_label_lv_en),
            INGAME_LABEL_HP_KO_PATH => Some(&self.ingame_label_hp_ko),
            INGAME_LABEL_HP_EN_PATH => Some(&self.ingame_label_hp_en),
            INGAME_LABEL_XP_KO_PATH => Some(&self.ingame_label_xp_ko),
            INGAME_LABEL_XP_EN_PATH => Some(&self.ingame_label_xp_en),
            INGAME_LABEL_GOLD_KO_PATH => Some(&self.ingame_label_gold_ko),
            INGAME_LABEL_GOLD_EN_PATH => Some(&self.ingame_label_gold_en),
            INGAME_LABEL_KILLS_KO_PATH => Some(&self.ingame_label_kills_ko),
            INGAME_LABEL_KILLS_EN_PATH => Some(&self.ingame_label_kills_en),
            INGAME_LABEL_PASSIVES_KO_PATH => Some(&self.ingame_label_passives_ko),
            INGAME_LABEL_PASSIVES_EN_PATH => Some(&self.ingame_label_passives_en),
            LEVELUP_TITLE_KO_PATH => Some(&self.levelup_title_ko),
            LEVELUP_TITLE_EN_PATH => Some(&self.levelup_title_en),
            GAMEOVER_TITLE_KO_PATH => Some(&self.gameover_title_ko),
            GAMEOVER_TITLE_EN_PATH => Some(&self.gameover_title_en),
            RESTART_HINT_KO_PATH => Some(&self.restart_hint_ko),
            RESTART_HINT_EN_PATH => Some(&self.restart_hint_en),
            SECTION_WEAPONS_KO_PATH => Some(&self.section_weapons_ko),
            SECTION_WEAPONS_EN_PATH => Some(&self.section_weapons_en),
            SECTION_PASSIVES_KO_PATH => Some(&self.section_passives_ko),
            SECTION_PASSIVES_EN_PATH => Some(&self.section_passives_en),
            _ => None,
        }
    }
}

pub fn survivor_textured_sprite(world: &World, path: &str) -> Sprite {
    Sprite::textured_with_handle(path, survivor_texture_handle(world, path))
}

pub fn survivor_texture_handle(world: &World, path: &str) -> Option<Handle<ImageAsset>> {
    world
        .resource::<SurvivorTextureHandles>()
        .and_then(|textures| textures.handle_for(path))
        .cloned()
}

pub fn survivor_texture_aspect(path: &str) -> Option<f32> {
    let (w, h) = match path {
        TITLE_BACKDROP_PATH => (1672.0, 941.0),
        TITLE_LOGO_PLAQUE_PATH => (2172.0, 724.0),
        MENU_BUTTON_START_KO_PATH
        | MENU_BUTTON_START_EN_PATH
        | MENU_BUTTON_CHARACTER_KO_PATH
        | MENU_BUTTON_CHARACTER_EN_PATH
        | MENU_BUTTON_STAGE_KO_PATH
        | MENU_BUTTON_STAGE_EN_PATH
        | MENU_BUTTON_SHOP_KO_PATH
        | MENU_BUTTON_SHOP_EN_PATH
        | MENU_BUTTON_ACHIEVEMENTS_KO_PATH
        | MENU_BUTTON_ACHIEVEMENTS_EN_PATH
        | MENU_BUTTON_SETTINGS_KO_PATH
        | MENU_BUTTON_SETTINGS_EN_PATH => (1600.0, 520.0),
        UI_MODAL_PANEL_PATH => (1478.0, 756.0),
        UI_SLOT_FRAME_PATH => (2125.0, 473.0),
        INGAME_LABEL_LV_KO_PATH
        | INGAME_LABEL_LV_EN_PATH
        | INGAME_LABEL_HP_KO_PATH
        | INGAME_LABEL_HP_EN_PATH
        | INGAME_LABEL_XP_KO_PATH
        | INGAME_LABEL_XP_EN_PATH
        | INGAME_LABEL_GOLD_KO_PATH
        | INGAME_LABEL_GOLD_EN_PATH
        | INGAME_LABEL_KILLS_KO_PATH
        | INGAME_LABEL_KILLS_EN_PATH
        | INGAME_LABEL_PASSIVES_KO_PATH
        | INGAME_LABEL_PASSIVES_EN_PATH => (240.0, 78.0),
        LEVELUP_TITLE_KO_PATH
        | LEVELUP_TITLE_EN_PATH
        | GAMEOVER_TITLE_KO_PATH
        | GAMEOVER_TITLE_EN_PATH => (760.0, 190.0),
        RESTART_HINT_KO_PATH | RESTART_HINT_EN_PATH => (620.0, 126.0),
        SECTION_WEAPONS_KO_PATH
        | SECTION_WEAPONS_EN_PATH
        | SECTION_PASSIVES_KO_PATH
        | SECTION_PASSIVES_EN_PATH => (340.0, 86.0),
        _ => return None,
    };
    Some(w / h)
}

#[derive(Debug, Clone, Copy)]
struct SpriteFrame {
    texture: &'static str,
    atlas_width: f32,
    atlas_height: f32,
    base_x: f32,
    base_y: f32,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl SpriteFrame {
    fn uv(self) -> UvRect {
        UvRect::from_pixels(
            self.base_x + self.x,
            self.base_y + self.y,
            self.width,
            self.height,
            self.atlas_width,
            self.atlas_height,
        )
    }

    fn fit_scale(self, max_size: f32) -> glam::Vec2 {
        if self.width >= self.height {
            glam::Vec2::new(max_size, max_size * self.height / self.width)
        } else {
            glam::Vec2::new(max_size * self.width / self.height, max_size)
        }
    }

    fn fit_inside(self, bounds: glam::Vec2) -> glam::Vec2 {
        let aspect = self.width / self.height.max(1.0);
        let bounds_aspect = bounds.x / bounds.y.max(1.0);
        if bounds_aspect > aspect {
            glam::Vec2::new(bounds.y * aspect, bounds.y)
        } else {
            glam::Vec2::new(bounds.x, bounds.x / aspect.max(0.001))
        }
    }
}

fn main_atlas_slot_y(row: u32) -> f32 {
    match row {
        0 => 0.0,
        1 => 314.0,
        2 => 627.0,
        3 => 940.0,
        _ => row as f32 * (ATLAS_HEIGHT / ATLAS_ROWS as f32),
    }
}

fn main_frame(col: u32, row: u32, x: f32, y: f32, width: f32, height: f32) -> SpriteFrame {
    SpriteFrame {
        texture: ATLAS_PATH,
        atlas_width: ATLAS_WIDTH,
        atlas_height: ATLAS_HEIGHT,
        base_x: col as f32 * ATLAS_SLOT_WIDTH,
        base_y: main_atlas_slot_y(row),
        x,
        y,
        width,
        height,
    }
}

fn effect_frame(col: u32, row: u32) -> SpriteFrame {
    SpriteFrame {
        texture: EFFECTS_PATH,
        atlas_width: EFFECTS_WIDTH,
        atlas_height: EFFECTS_HEIGHT,
        base_x: col as f32 * EFFECTS_SLOT_SIZE,
        base_y: row as f32 * EFFECTS_SLOT_SIZE,
        x: 0.0,
        y: 0.0,
        width: EFFECTS_SLOT_SIZE,
        height: EFFECTS_SLOT_SIZE,
    }
}

fn actor_frame(col: u32, row: u32) -> SpriteFrame {
    SpriteFrame {
        texture: ACTOR_FRAMES_PATH,
        atlas_width: ACTOR_FRAMES_WIDTH,
        atlas_height: ACTOR_FRAMES_HEIGHT,
        base_x: col as f32 * ACTOR_FRAME_WIDTH,
        base_y: row as f32 * ACTOR_FRAME_HEIGHT,
        x: 0.0,
        y: 0.0,
        width: ACTOR_FRAME_WIDTH,
        height: ACTOR_FRAME_HEIGHT,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurvivorSprite {
    Hero,
    Zombie,
    Bat,
    Ghost,
    Skeleton,
    Mage,
    Mantis,
    Plant,
    Slime,
    Mummy,
    Knight,
    GiantSlime,
    GhostKing,
    Death,
    XpGem,
    Coin,
    Chicken,
    Vacuum,
    Bomb,
    Rosary,
    Chest,
    MagicBolt,
    Knife,
    Axe,
    WhipSlash,
    WhipSlashLeft,
    WhipSlashRight,
    WhipSlashUp,
    WhipSlashDown,
    Fireball,
    CrossProjectile,
    HolyWaterPool,
    HolyBook,
    LightningStrike,
    GarlicAura,
    ImpactSpark,
}

impl SurvivorSprite {
    fn frame(self) -> SpriteFrame {
        let frame = match self {
            SurvivorSprite::Hero => (0, 0, 44.0, 136.0, 151.0, 164.0),
            SurvivorSprite::Zombie => (1, 0, 53.0, 131.0, 135.0, 172.0),
            SurvivorSprite::Bat => (2, 0, 16.0, 167.0, 193.0, 117.0),
            SurvivorSprite::Ghost => (3, 0, 0.0, 130.0, 185.0, 173.0),
            SurvivorSprite::Skeleton => (4, 0, 55.0, 132.0, 137.0, 177.0),
            SurvivorSprite::Mage => (5, 0, 35.0, 132.0, 126.0, 177.0),
            SurvivorSprite::Mantis => (0, 1, 24.0, 77.0, 181.0, 184.0),
            SurvivorSprite::Plant => (1, 1, 30.0, 66.0, 170.0, 204.0),
            SurvivorSprite::Slime => (2, 1, 35.0, 112.0, 160.0, 136.0),
            SurvivorSprite::Mummy => (3, 1, 42.0, 73.0, 149.0, 181.0),
            SurvivorSprite::Knight => (4, 1, 19.0, 66.0, 190.0, 196.0),
            SurvivorSprite::GiantSlime => (5, 1, 0.0, 57.0, 189.0, 210.0),
            SurvivorSprite::GhostKing => (0, 2, 24.0, 4.0, 171.0, 309.0),
            SurvivorSprite::Death => (1, 2, 18.0, 4.0, 191.0, 309.0),
            SurvivorSprite::XpGem => (2, 2, 0.0, 44.0, 161.0, 165.0),
            SurvivorSprite::Coin => (3, 2, 45.0, 76.0, 114.0, 120.0),
            SurvivorSprite::Chicken => (4, 2, 9.0, 65.0, 172.0, 143.0),
            SurvivorSprite::Vacuum => (5, 2, 19.0, 65.0, 137.0, 134.0),
            SurvivorSprite::Bomb => (0, 3, 46.0, 0.0, 120.0, 155.0),
            SurvivorSprite::Rosary => (1, 3, 22.0, 0.0, 129.0, 174.0),
            SurvivorSprite::Chest => (2, 3, 13.0, 9.0, 178.0, 160.0),
            SurvivorSprite::MagicBolt => (3, 3, 44.0, 16.0, 137.0, 133.0),
            SurvivorSprite::Knife => (4, 3, 18.0, 9.0, 191.0, 155.0),
            SurvivorSprite::Axe => (5, 3, 0.0, 5.0, 184.0, 172.0),
            SurvivorSprite::WhipSlash => return effect_frame(0, 0),
            SurvivorSprite::WhipSlashLeft => return effect_frame(2, 1),
            SurvivorSprite::WhipSlashRight => return effect_frame(3, 1),
            SurvivorSprite::WhipSlashUp => return effect_frame(4, 1),
            SurvivorSprite::WhipSlashDown => return effect_frame(5, 1),
            SurvivorSprite::Fireball => return effect_frame(1, 0),
            SurvivorSprite::CrossProjectile => return effect_frame(2, 0),
            SurvivorSprite::HolyWaterPool => return effect_frame(3, 0),
            SurvivorSprite::HolyBook => return effect_frame(4, 0),
            SurvivorSprite::LightningStrike => return effect_frame(5, 0),
            SurvivorSprite::GarlicAura => return effect_frame(0, 1),
            SurvivorSprite::ImpactSpark => return effect_frame(1, 1),
        };
        main_frame(frame.0, frame.1, frame.2, frame.3, frame.4, frame.5)
    }

    pub fn uv(self) -> UvRect {
        self.frame().uv()
    }

    pub fn fit_scale(self, max_size: f32) -> glam::Vec2 {
        self.render_frame().fit_scale(max_size)
    }

    pub fn fit_inside(self, bounds: glam::Vec2) -> glam::Vec2 {
        self.render_frame().fit_inside(bounds)
    }

    fn actor_row(self) -> Option<u32> {
        Some(match self {
            SurvivorSprite::Hero => 0,
            SurvivorSprite::Zombie => 1,
            SurvivorSprite::Bat => 2,
            SurvivorSprite::Ghost => 3,
            SurvivorSprite::Skeleton => 4,
            SurvivorSprite::Mage => 5,
            SurvivorSprite::Mantis => 6,
            SurvivorSprite::Plant => 7,
            SurvivorSprite::Slime => 8,
            SurvivorSprite::Mummy => 9,
            SurvivorSprite::Knight => 10,
            SurvivorSprite::GiantSlime => 11,
            SurvivorSprite::GhostKing => 12,
            SurvivorSprite::Death => 13,
            _ => return None,
        })
    }

    fn render_frame(self) -> SpriteFrame {
        self.actor_row()
            .map(|row| actor_frame(0, row))
            .unwrap_or_else(|| self.frame())
    }

    fn animation_player(self) -> AnimationPlayer {
        if let Some(row) = self.actor_row() {
            let move_frames = (0..ACTOR_MOVE_FRAME_COUNT)
                .map(|col| actor_frame(col, row).uv())
                .collect();
            AnimationPlayer::new(vec![
                AnimationClip {
                    frames: move_frames,
                    fps: ACTOR_MOVE_FPS,
                    looping: true,
                },
                AnimationClip {
                    frames: vec![actor_frame(ACTOR_HIT_FRAME_COL, row).uv()],
                    fps: 1.0,
                    looping: false,
                },
                AnimationClip {
                    frames: vec![actor_frame(ACTOR_DEATH_FRAME_COL, row).uv()],
                    fps: 1.0,
                    looping: false,
                },
            ])
        } else {
            single_frame(self)
        }
    }

    fn render_layer(self) -> i32 {
        match self {
            SurvivorSprite::WhipSlash
            | SurvivorSprite::WhipSlashLeft
            | SurvivorSprite::WhipSlashRight
            | SurvivorSprite::WhipSlashUp
            | SurvivorSprite::WhipSlashDown
            | SurvivorSprite::Fireball
            | SurvivorSprite::CrossProjectile
            | SurvivorSprite::HolyWaterPool
            | SurvivorSprite::HolyBook
            | SurvivorSprite::LightningStrike
            | SurvivorSprite::GarlicAura
            | SurvivorSprite::ImpactSpark => RENDER_LAYER_EFFECTS,
            _ => RENDER_LAYER_WORLD,
        }
    }
}

pub fn add_sprite(world: &mut World, entity: Entity, sprite: SurvivorSprite) {
    add_tinted_sprite(world, entity, sprite, [1.0, 1.0, 1.0, 1.0]);
}

pub fn add_tinted_sprite(
    world: &mut World,
    entity: Entity,
    sprite: SurvivorSprite,
    color: [f32; 4],
) {
    let frame = sprite.render_frame();
    let uv = frame.uv();
    let mut sprite_component = survivor_textured_sprite(world, frame.texture);
    sprite_component.color = engine::Color::from(color);
    world.add_component(entity, sprite_component);
    world.add_component(entity, RenderLayer(sprite.render_layer()));
    world.add_component(entity, uv);
    world.add_component(entity, sprite.animation_player());

    if let Some(transform) = world.get_mut::<Transform>(entity) {
        let max_size = transform.scale.x.max(transform.scale.y);
        transform.scale = sprite.fit_scale(max_size);
    }
}

fn single_frame(sprite: SurvivorSprite) -> AnimationPlayer {
    AnimationPlayer::new(vec![AnimationClip {
        frames: vec![sprite.uv()],
        fps: 1.0,
        looping: false,
    }])
}

#[cfg(test)]
mod tests {
    use super::*;

    const ACTOR_SPRITES: &[SurvivorSprite] = &[
        SurvivorSprite::Hero,
        SurvivorSprite::Zombie,
        SurvivorSprite::Bat,
        SurvivorSprite::Ghost,
        SurvivorSprite::Skeleton,
        SurvivorSprite::Mage,
        SurvivorSprite::Mantis,
        SurvivorSprite::Plant,
        SurvivorSprite::Slime,
        SurvivorSprite::Mummy,
        SurvivorSprite::Knight,
        SurvivorSprite::GiantSlime,
        SurvivorSprite::GhostKing,
        SurvivorSprite::Death,
    ];
    const ALL_SURVIVOR_SPRITES: &[SurvivorSprite] = &[
        SurvivorSprite::Hero,
        SurvivorSprite::Zombie,
        SurvivorSprite::Bat,
        SurvivorSprite::Ghost,
        SurvivorSprite::Skeleton,
        SurvivorSprite::Mage,
        SurvivorSprite::Mantis,
        SurvivorSprite::Plant,
        SurvivorSprite::Slime,
        SurvivorSprite::Mummy,
        SurvivorSprite::Knight,
        SurvivorSprite::GiantSlime,
        SurvivorSprite::GhostKing,
        SurvivorSprite::Death,
        SurvivorSprite::XpGem,
        SurvivorSprite::Coin,
        SurvivorSprite::Chicken,
        SurvivorSprite::Vacuum,
        SurvivorSprite::Bomb,
        SurvivorSprite::Rosary,
        SurvivorSprite::Chest,
        SurvivorSprite::MagicBolt,
        SurvivorSprite::Knife,
        SurvivorSprite::Axe,
        SurvivorSprite::WhipSlash,
        SurvivorSprite::WhipSlashLeft,
        SurvivorSprite::WhipSlashRight,
        SurvivorSprite::WhipSlashUp,
        SurvivorSprite::WhipSlashDown,
        SurvivorSprite::Fireball,
        SurvivorSprite::CrossProjectile,
        SurvivorSprite::HolyWaterPool,
        SurvivorSprite::HolyBook,
        SurvivorSprite::LightningStrike,
        SurvivorSprite::GarlicAura,
        SurvivorSprite::ImpactSpark,
    ];

    fn texture_handles_with(handle: Handle<ImageAsset>) -> SurvivorTextureHandles {
        SurvivorTextureHandles {
            atlas: handle.clone(),
            effects: handle.clone(),
            actor_frames: handle.clone(),
            evolutions: handle.clone(),
            icons: handle.clone(),
            passives: handle.clone(),
            powerups: handle.clone(),
            title_backdrop: handle.clone(),
            title_logo_plaque: handle.clone(),
            menu_button_start_ko: handle.clone(),
            menu_button_start_en: handle.clone(),
            menu_button_character_ko: handle.clone(),
            menu_button_character_en: handle.clone(),
            menu_button_stage_ko: handle.clone(),
            menu_button_stage_en: handle.clone(),
            menu_button_shop_ko: handle.clone(),
            menu_button_shop_en: handle.clone(),
            menu_button_achievements_ko: handle.clone(),
            menu_button_achievements_en: handle.clone(),
            menu_button_settings_ko: handle.clone(),
            menu_button_settings_en: handle.clone(),
            ui_modal_panel: handle.clone(),
            ui_slot_frame: handle.clone(),
            ingame_label_lv_ko: handle.clone(),
            ingame_label_lv_en: handle.clone(),
            ingame_label_hp_ko: handle.clone(),
            ingame_label_hp_en: handle.clone(),
            ingame_label_xp_ko: handle.clone(),
            ingame_label_xp_en: handle.clone(),
            ingame_label_gold_ko: handle.clone(),
            ingame_label_gold_en: handle.clone(),
            ingame_label_kills_ko: handle.clone(),
            ingame_label_kills_en: handle.clone(),
            ingame_label_passives_ko: handle.clone(),
            ingame_label_passives_en: handle.clone(),
            levelup_title_ko: handle.clone(),
            levelup_title_en: handle.clone(),
            gameover_title_ko: handle.clone(),
            gameover_title_en: handle.clone(),
            restart_hint_ko: handle.clone(),
            restart_hint_en: handle.clone(),
            section_weapons_ko: handle.clone(),
            section_weapons_en: handle.clone(),
            section_passives_ko: handle.clone(),
            section_passives_en: handle,
        }
    }

    #[test]
    fn survivor_sprite_uvs_use_top_left_engine_uvs() {
        let uv = SurvivorSprite::Hero.uv();

        assert!(uv.v_size > 0.0);
        assert!(uv.v_offset > 0.0);
        assert!(uv.v_offset <= 1.0 / ATLAS_ROWS as f32);
    }

    #[test]
    fn sprite_fit_scale_preserves_source_aspect() {
        let scale = SurvivorSprite::Hero.fit_scale(PLAYER_VISUAL_SIZE);

        assert_eq!(scale.y, PLAYER_VISUAL_SIZE);
        assert!((scale.x - 112.5).abs() < 0.0001);
    }

    #[test]
    fn actor_sprite_scales_match_actor_frame_aspect() {
        let expected = ACTOR_FRAME_WIDTH / ACTOR_FRAME_HEIGHT;

        for &sprite in ACTOR_SPRITES {
            let scale = sprite.fit_scale(PLAYER_VISUAL_SIZE);
            assert!(
                (scale.x / scale.y - expected).abs() < 0.0001,
                "{sprite:?} should render at actor frame aspect"
            );
        }
    }

    #[test]
    fn all_survivor_sprite_fit_scales_match_render_frame_aspect() {
        for &sprite in ALL_SURVIVOR_SPRITES {
            let scale = sprite.fit_scale(PLAYER_VISUAL_SIZE);
            let frame = sprite.render_frame();
            assert!(
                (scale.x / scale.y - frame.width / frame.height).abs() < 0.0001,
                "{sprite:?} fit scale should match rendered frame aspect"
            );
        }
    }

    #[test]
    fn add_sprite_preserves_render_frame_aspect() {
        for &sprite in ALL_SURVIVOR_SPRITES {
            let mut world = World::new();
            let entity = world.spawn();
            world.add_component(
                entity,
                Transform {
                    scale: glam::Vec2::splat(PLAYER_VISUAL_SIZE),
                    ..Default::default()
                },
            );
            add_sprite(&mut world, entity, sprite);

            let transform = world.get::<Transform>(entity).unwrap();
            let frame = sprite.render_frame();
            assert!(
                (transform.scale.x / transform.scale.y - frame.width / frame.height).abs() < 0.0001,
                "{sprite:?} transform should match rendered frame aspect"
            );
        }
    }

    #[test]
    fn visual_size_constants_keep_characters_readable() {
        assert_eq!(PLAYER_VISUAL_SIZE, 150.0);
        assert_eq!(ENEMY_VISUAL_SCALE, 3.75);
    }

    #[test]
    fn effect_sprites_use_effects_atlas() {
        assert_eq!(SurvivorSprite::WhipSlash.frame().texture, EFFECTS_PATH);
        assert_eq!(
            SurvivorSprite::LightningStrike.frame().atlas_height,
            EFFECTS_HEIGHT
        );
    }

    #[test]
    fn actor_sprites_use_generated_actor_frames() {
        let frame = SurvivorSprite::Hero.render_frame();

        assert_eq!(frame.texture, ACTOR_FRAMES_PATH);
        assert_eq!(frame.atlas_width, ACTOR_FRAMES_WIDTH);
        assert_eq!(frame.atlas_height, ACTOR_FRAMES_HEIGHT);
        assert_eq!(SurvivorSprite::Death.actor_row(), Some(13));
    }

    #[test]
    fn actor_animation_uses_three_looping_move_frames() {
        let player = SurvivorSprite::Zombie.animation_player();

        assert_eq!(player.clips.len(), 3);
        assert_eq!(
            player.clips[0].frames.len(),
            ACTOR_MOVE_FRAME_COUNT as usize
        );
        assert!(player.clips[0].looping);
        assert_eq!(player.clips[0].fps, ACTOR_MOVE_FPS);
        assert_eq!(player.clips[1].frames.len(), 1);
        assert_eq!(player.clips[2].frames.len(), 1);
    }

    #[test]
    fn textured_sprite_falls_back_to_path_without_texture_handles() {
        let world = World::new();
        let sprite = survivor_textured_sprite(&world, ATLAS_PATH);

        assert_eq!(sprite.texture.as_deref(), Some(ATLAS_PATH));
        assert!(sprite.image_handle.is_none());
    }

    #[test]
    fn textured_sprite_prefers_handle_while_keeping_path_fallback() {
        let mut world = World::new();
        let handle = engine::AssetServer::new().load_image(ATLAS_PATH);
        world.insert_resource(texture_handles_with(handle.clone()));

        let sprite = survivor_textured_sprite(&world, ATLAS_PATH);
        let textures = world.resource::<SurvivorTextureHandles>().unwrap();

        assert_eq!(sprite.texture.as_deref(), Some(ATLAS_PATH));
        assert_eq!(
            sprite.image_handle.as_ref().map(|handle| handle.path()),
            Some(handle.path())
        );
        assert!(textures.handle_for(INGAME_LABEL_LV_KO_PATH).is_some());
        assert!(textures.handle_for(INGAME_LABEL_KILLS_EN_PATH).is_some());
        assert!(textures.handle_for(LEVELUP_TITLE_EN_PATH).is_some());
        assert!(textures.handle_for(GAMEOVER_TITLE_KO_PATH).is_some());
        assert!(textures.handle_for(RESTART_HINT_EN_PATH).is_some());
        assert!(textures.handle_for(SECTION_WEAPONS_KO_PATH).is_some());
        assert!(textures.handle_for(SECTION_PASSIVES_EN_PATH).is_some());
    }

    #[test]
    fn textured_sprite_passes_canonicalized_handle_key() {
        let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let absolute_atlas = manifest_dir
            .join("../../assets/textures/survivor/survivor_atlas.png")
            .canonicalize()
            .expect("test atlas path should exist");
        let handle = engine::AssetServer::new().load_image(absolute_atlas);
        assert_ne!(handle.path(), ATLAS_PATH);
        let expected_path = handle.path().to_string();

        let mut world = World::new();
        world.insert_resource(texture_handles_with(handle));
        let sprite = survivor_textured_sprite(&world, ATLAS_PATH);

        assert_eq!(sprite.texture.as_deref(), Some(ATLAS_PATH));
        assert_eq!(
            sprite.image_handle.as_ref().map(|handle| handle.path()),
            Some(expected_path.as_str())
        );
    }

    #[test]
    fn whole_ui_texture_aspects_are_registered() {
        assert!(
            (survivor_texture_aspect(TITLE_LOGO_PLAQUE_PATH).unwrap() - 2172.0 / 724.0).abs()
                < 0.001
        );
        assert!(
            (survivor_texture_aspect(MENU_BUTTON_START_EN_PATH).unwrap() - 1600.0 / 520.0).abs()
                < 0.001
        );
        assert!(
            (survivor_texture_aspect(INGAME_LABEL_LV_KO_PATH).unwrap() - 240.0 / 78.0).abs()
                < 0.001
        );
        assert!(
            (survivor_texture_aspect(LEVELUP_TITLE_EN_PATH).unwrap() - 760.0 / 190.0).abs() < 0.001
        );
        assert!(
            (survivor_texture_aspect(RESTART_HINT_KO_PATH).unwrap() - 620.0 / 126.0).abs() < 0.001
        );
        assert!(
            (survivor_texture_aspect(SECTION_WEAPONS_EN_PATH).unwrap() - 340.0 / 86.0).abs()
                < 0.001
        );
    }

    #[test]
    fn add_sprite_assigns_world_or_effect_render_layer() {
        let mut world = World::new();
        let enemy = world.spawn();
        world.add_component(enemy, Transform::default());
        add_sprite(&mut world, enemy, SurvivorSprite::Zombie);
        assert_eq!(
            world.get::<RenderLayer>(enemy).map(|layer| layer.0),
            Some(RENDER_LAYER_WORLD)
        );

        let effect = world.spawn();
        world.add_component(effect, Transform::default());
        add_sprite(&mut world, effect, SurvivorSprite::LightningStrike);
        assert_eq!(
            world.get::<RenderLayer>(effect).map(|layer| layer.0),
            Some(RENDER_LAYER_EFFECTS)
        );
    }
}
