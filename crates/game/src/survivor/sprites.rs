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
pub const TITLE_BACKDROP_PATH: &str = "assets/textures/survivor/title_backdrop.png";
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
            _ => None,
        }
    }
}

pub fn survivor_textured_sprite(world: &World, path: &str) -> Sprite {
    world
        .resource::<SurvivorTextureHandles>()
        .and_then(|textures| textures.handle_for(path))
        .cloned()
        .map(Sprite::with_handle)
        .unwrap_or_else(|| Sprite::textured(path))
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
        let u_offset = (self.base_x + self.x) / self.atlas_width;
        let v_top = (self.base_y + self.y) / self.atlas_height;
        let v_size = self.height / self.atlas_height;

        UvRect {
            u_offset,
            v_offset: v_top + v_size,
            u_size: self.width / self.atlas_width,
            v_size: -v_size,
        }
    }

    fn fit_scale(self, max_size: f32) -> glam::Vec2 {
        if self.width >= self.height {
            glam::Vec2::new(max_size, max_size * self.height / self.width)
        } else {
            glam::Vec2::new(max_size * self.width / self.height, max_size)
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
        self.frame().fit_scale(max_size)
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

pub fn vertically_flipped_full_uv() -> UvRect {
    let mut uv = UvRect::FULL;
    uv.v_offset += uv.v_size;
    uv.v_size = -uv.v_size;
    uv
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
    sprite_component.color = color;
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

    #[test]
    fn survivor_sprite_uvs_are_vertically_flipped_for_engine_quad() {
        let uv = SurvivorSprite::Hero.uv();

        assert!(uv.v_size < 0.0);
        assert!(uv.v_offset > 0.0);
        assert!(uv.v_offset <= 1.0 / ATLAS_ROWS as f32);
    }

    #[test]
    fn sprite_fit_scale_preserves_source_aspect() {
        let scale = SurvivorSprite::Hero.fit_scale(PLAYER_VISUAL_SIZE);

        assert_eq!(scale.y, PLAYER_VISUAL_SIZE);
        assert!((scale.x - 138.10976).abs() < 0.0001);
    }

    #[test]
    fn visual_size_constants_keep_characters_readable() {
        assert_eq!(PLAYER_VISUAL_SIZE, 150.0);
        assert!(40.0 * ENEMY_VISUAL_SCALE >= 150.0);
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
