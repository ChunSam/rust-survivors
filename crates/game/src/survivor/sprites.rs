use engine::{AnimationClip, AnimationPlayer, Entity, Sprite, UvRect, World};

/// Shared generated sprite atlas for the survivor mode.
pub const ATLAS_PATH: &str = "assets/textures/survivor/survivor_atlas.png";
pub const TITLE_BACKDROP_PATH: &str = "assets/textures/survivor/title_backdrop.png";
pub const PLAYER_VISUAL_SIZE: f32 = 96.0;
pub const ENEMY_VISUAL_SCALE: f32 = 1.8;
pub const BOSS_VISUAL_SCALE: f32 = 1.3;

const ATLAS_COLS: u32 = 6;
const ATLAS_ROWS: u32 = 6;

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
}

impl SurvivorSprite {
    pub fn uv(self) -> UvRect {
        let (col, row) = match self {
            SurvivorSprite::Hero => (0, 0),
            SurvivorSprite::Zombie => (1, 0),
            SurvivorSprite::Bat => (2, 0),
            SurvivorSprite::Ghost => (3, 0),
            SurvivorSprite::Skeleton => (4, 0),
            SurvivorSprite::Mage => (5, 0),
            SurvivorSprite::Mantis => (0, 1),
            SurvivorSprite::Plant => (1, 1),
            SurvivorSprite::Slime => (2, 1),
            SurvivorSprite::Mummy => (3, 1),
            SurvivorSprite::Knight => (4, 1),
            SurvivorSprite::GiantSlime => (5, 1),
            SurvivorSprite::GhostKing => (0, 2),
            SurvivorSprite::Death => (1, 2),
            SurvivorSprite::XpGem => (2, 2),
            SurvivorSprite::Coin => (3, 2),
            SurvivorSprite::Chicken => (4, 2),
            SurvivorSprite::Vacuum => (5, 2),
            SurvivorSprite::Bomb => (0, 3),
            SurvivorSprite::Rosary => (1, 3),
            SurvivorSprite::Chest => (2, 3),
            SurvivorSprite::MagicBolt => (3, 3),
            SurvivorSprite::Knife => (4, 3),
            SurvivorSprite::Axe => (5, 3),
        };
        UvRect::from_grid(col, row, ATLAS_COLS, ATLAS_ROWS)
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
    let uv = sprite.uv();
    world.add_component(
        entity,
        Sprite {
            texture: Some(ATLAS_PATH.to_string()),
            color,
            image_handle: None,
        },
    );
    world.add_component(entity, uv);
    world.add_component(entity, single_frame(sprite));
}

fn single_frame(sprite: SurvivorSprite) -> AnimationPlayer {
    AnimationPlayer::new(vec![AnimationClip {
        frames: vec![sprite.uv()],
        fps: 1.0,
        looping: false,
    }])
}
