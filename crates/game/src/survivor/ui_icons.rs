use super::icons::{weapon_icon, ICONS_PATH, ICON_COLS, ICON_ROWS};
use super::inventory::{WeaponInventory, WeaponKind};
use super::levelup::{CardKind, PendingLevelUp};
use super::meta::SurvivorMode;
use super::passive::{PassiveInventory, PassiveKind};
use super::player::Player;
use super::powerup::PowerUpKind;
use super::sprites::{survivor_textured_sprite, PASSIVES_PATH, POWERUPS_PATH, RENDER_LAYER_UI};
use engine::{
    Camera, Entity, GameState, RenderLayer, System, Transform, UvRect, ViewportSize, World,
};
use glam::Vec2;

const SLOT_COLS: usize = 6;
const SLOT_W: f32 = 152.0;
const SLOT_H: f32 = 38.0;
const SLOT_GAP: f32 = 6.0;
const XP_BAR_H: f32 = 14.0;
const ICON_SIZE: f32 = 28.0;
const LEVEL_ICON_SIZE: f32 = 34.0;
const SHOP_ICON_SIZE: f32 = 22.0;
#[cfg(test)]
const SLOT_TEXT_INSET: f32 = 42.0;
#[cfg(test)]
const SHOP_TEXT_X_OFFSET: f32 = -306.0;
#[cfg(test)]
const LEVELUP_TEXT_X_OFFSET: f32 = -178.0;

#[derive(Debug, Clone, Copy)]
pub struct UiIconSprite;

#[derive(Debug, Clone, Copy)]
struct HudIconLayout {
    ui_scale: f32,
    slot_x0: f32,
    slot_w: f32,
    slot_h: f32,
    slot_gap: f32,
    weapon_y: f32,
    passive_y: f32,
    icon_size: f32,
}

#[derive(Debug, Clone, Copy)]
enum IconSource {
    Icons { col: u32, row: u32 },
    Passive(PassiveKind),
    PowerUp(PowerUpKind),
}

#[derive(Default)]
pub struct UiIconSystem;

impl System for UiIconSystem {
    fn run(&mut self, world: &mut World, _dt: f32) {
        despawn_previous_icons(world);

        let Some(viewport) = world.resource::<ViewportSize>().copied() else {
            return;
        };
        let Some(camera) = world.resource::<Camera>().copied() else {
            return;
        };
        let mode = world
            .resource::<SurvivorMode>()
            .copied()
            .unwrap_or_default();
        let is_levelup = matches!(world.resource::<GameState>(), Some(GameState::Paused));

        spawn_hud_slot_icons(world, camera, viewport);
        if is_levelup {
            spawn_levelup_icons(world, camera, viewport);
        }
        if matches!(mode, SurvivorMode::Shop) {
            spawn_shop_icons(world, camera, viewport);
        }
    }
}

fn despawn_previous_icons(world: &mut World) {
    let entities: Vec<Entity> = world.query::<UiIconSprite>().map(|(e, _)| e).collect();
    for entity in entities {
        world.despawn(entity);
    }
}

fn spawn_hud_slot_icons(world: &mut World, camera: Camera, viewport: ViewportSize) {
    let weapon_slots: Vec<(WeaponKind, bool)> = world
        .query2::<Player, WeaponInventory>()
        .next()
        .map(|(_, _, inv)| {
            inv.slots
                .iter()
                .map(|s| (s.kind.clone(), s.evolved))
                .collect()
        })
        .unwrap_or_default();
    let passive_slots: Vec<PassiveKind> = world
        .query2::<Player, PassiveInventory>()
        .next()
        .map(|(_, _, inv)| inv.passives.iter().map(|s| s.kind).collect())
        .unwrap_or_default();

    let layout = hud_icon_layout(viewport.width, viewport.height);

    for (i, (kind, evolved)) in weapon_slots.iter().enumerate().take(SLOT_COLS) {
        let icon = if *evolved {
            weapon_evolution_icon(kind)
        } else {
            weapon_source(kind)
        };
        spawn_icon(
            world,
            camera,
            hud_icon_center(&layout, i, 0),
            layout.icon_size,
            icon,
            40.0,
        );
    }

    for (i, kind) in passive_slots.iter().enumerate().take(SLOT_COLS) {
        spawn_icon(
            world,
            camera,
            hud_icon_center(&layout, i, 1),
            layout.icon_size,
            IconSource::Passive(*kind),
            40.0,
        );
    }
}

fn spawn_levelup_icons(world: &mut World, camera: Camera, viewport: ViewportSize) {
    let Some(offered) = world
        .resource::<PendingLevelUp>()
        .map(|pending| pending.offered)
    else {
        return;
    };

    for (i, card) in offered.iter().enumerate() {
        spawn_icon(
            world,
            camera,
            levelup_icon_center(viewport.width, viewport.height, i),
            levelup_icon_size(viewport.width, viewport.height),
            card_icon(*card),
            45.0,
        );
    }
}

fn spawn_shop_icons(world: &mut World, camera: Camera, viewport: ViewportSize) {
    let mode = world
        .resource::<SurvivorMode>()
        .copied()
        .unwrap_or_default();
    if !matches!(mode, SurvivorMode::Shop) {
        return;
    }

    for (i, kind) in PowerUpKind::ALL.iter().enumerate() {
        spawn_icon(
            world,
            camera,
            shop_icon_center(viewport.width, viewport.height, i),
            shop_icon_size(viewport.width, viewport.height),
            IconSource::PowerUp(*kind),
            45.0,
        );
    }
}

fn spawn_icon(
    world: &mut World,
    camera: Camera,
    screen_center: Vec2,
    screen_size: f32,
    source: IconSource,
    z: f32,
) {
    let (texture, uv) = source.texture_and_uv();
    let world_center = camera.screen_to_world(screen_center);
    let world_size = screen_size / camera.zoom.max(0.01);
    let entity = world.spawn();
    world.add_component(
        entity,
        Transform {
            position: world_center,
            scale: Vec2::splat(world_size),
            rotation: 0.0,
            z,
        },
    );
    world.add_component(entity, survivor_textured_sprite(world, texture));
    world.add_component(entity, RenderLayer(RENDER_LAYER_UI));
    world.add_component(entity, uv);
    world.add_component(entity, UiIconSprite);
}

fn responsive_ui_scale(viewport_w: f32, viewport_h: f32) -> f32 {
    (viewport_w / 1280.0)
        .min(viewport_h / 720.0)
        .clamp(0.72, 1.5)
}

fn hud_icon_layout(viewport_w: f32, viewport_h: f32) -> HudIconLayout {
    let ui_scale = responsive_ui_scale(viewport_w, viewport_h);
    let slot_x0 = 16.0 * ui_scale;
    let slot_gap = SLOT_GAP * ui_scale;
    let slot_h = SLOT_H * ui_scale;
    let max_panel_w = (viewport_w - slot_x0 * 2.0).max(300.0);
    let slot_w = (SLOT_W * ui_scale)
        .min((max_panel_w - (SLOT_COLS - 1) as f32 * slot_gap) / SLOT_COLS as f32);
    let weapon_y = viewport_h - XP_BAR_H * ui_scale - slot_h * 2.0 - slot_gap - 16.0 * ui_scale;
    let passive_y = weapon_y + slot_h + slot_gap;
    HudIconLayout {
        ui_scale,
        slot_x0,
        slot_w,
        slot_h,
        slot_gap,
        weapon_y,
        passive_y,
        icon_size: (ICON_SIZE * ui_scale).min(32.0),
    }
}

fn hud_icon_center(layout: &HudIconLayout, slot_index: usize, row: usize) -> Vec2 {
    let sx = layout.slot_x0 + slot_index as f32 * (layout.slot_w + layout.slot_gap);
    let y = if row == 0 {
        layout.weapon_y
    } else {
        layout.passive_y
    };
    Vec2::new(sx + 22.0 * layout.ui_scale, y + layout.slot_h / 2.0)
}

#[cfg(test)]
fn hud_slot_text_x(layout: &HudIconLayout, slot_index: usize) -> f32 {
    let sx = layout.slot_x0 + slot_index as f32 * (layout.slot_w + layout.slot_gap);
    sx + SLOT_TEXT_INSET * layout.ui_scale
}

fn levelup_icon_center(viewport_w: f32, viewport_h: f32, card_index: usize) -> Vec2 {
    let cx = viewport_w / 2.0;
    let cy = viewport_h / 2.0;
    Vec2::new(cx - 206.0, cy - 35.0 + card_index as f32 * 56.0)
}

fn levelup_icon_size(viewport_w: f32, viewport_h: f32) -> f32 {
    (LEVEL_ICON_SIZE * responsive_ui_scale(viewport_w, viewport_h)).clamp(28.0, 36.0)
}

#[cfg(test)]
fn levelup_text_x(viewport_w: f32) -> f32 {
    viewport_w / 2.0 + LEVELUP_TEXT_X_OFFSET
}

fn shop_row_step(viewport_h: f32) -> f32 {
    if viewport_h <= 620.0 {
        26.0
    } else {
        30.0
    }
}

fn shop_icon_center(viewport_w: f32, viewport_h: f32, row_index: usize) -> Vec2 {
    let cx = viewport_w / 2.0;
    Vec2::new(
        cx - 326.0,
        90.0 + row_index as f32 * shop_row_step(viewport_h),
    )
}

fn shop_icon_size(viewport_w: f32, viewport_h: f32) -> f32 {
    (SHOP_ICON_SIZE * responsive_ui_scale(viewport_w, viewport_h)).clamp(18.0, 24.0)
}

#[cfg(test)]
fn shop_text_x(viewport_w: f32) -> f32 {
    viewport_w / 2.0 + SHOP_TEXT_X_OFFSET
}

fn weapon_source(kind: &WeaponKind) -> IconSource {
    let frame = weapon_icon(kind).frame();
    IconSource::Icons {
        col: frame.col,
        row: frame.row,
    }
}

fn weapon_evolution_icon(kind: &WeaponKind) -> IconSource {
    let index = match kind {
        WeaponKind::Whip { .. } => 26,
        WeaponKind::MagicWand { .. } => 27,
        WeaponKind::Knife { .. } => 28,
        WeaponKind::Axe { .. } => 29,
        WeaponKind::Cross { .. } => 30,
        WeaponKind::FireWand { .. } => 31,
        WeaponKind::Garlic { .. } => 32,
        WeaponKind::HolyWater { .. } => 33,
        WeaponKind::KingBible { .. } | WeaponKind::LightningRing { .. } => 35,
    };
    IconSource::Icons {
        col: index % ICON_COLS,
        row: index / ICON_COLS,
    }
}

fn card_icon(card: CardKind) -> IconSource {
    if let Some(passive) = passive_from_card(card) {
        return IconSource::Passive(passive);
    }

    let weapon = match card {
        CardKind::WhipDamage | CardKind::WhipArea | CardKind::WhipCooldown => 0,
        CardKind::MagicWandDamage | CardKind::MagicWandSpeed | CardKind::MagicWandCooldown => 1,
        CardKind::KnifeDamage | CardKind::KnifeAmount | CardKind::KnifeCooldown => 2,
        CardKind::AxeDamage | CardKind::AxePierce | CardKind::AxeCooldown => 3,
        CardKind::CrossDamage | CardKind::CrossReturnAt | CardKind::CrossCooldown => 4,
        CardKind::FireWandDamage | CardKind::FireWandCooldown => 5,
        CardKind::GarlicDamage | CardKind::GarlicRadius | CardKind::GarlicCooldown => 6,
        CardKind::HolyWaterDamage | CardKind::HolyWaterDropCount | CardKind::HolyWaterCooldown => 7,
        CardKind::KingBibleDamage | CardKind::KingBibleBookCount | CardKind::KingBibleCooldown => 8,
        CardKind::LightningDamage
        | CardKind::LightningStrikeCount
        | CardKind::LightningCooldown => 9,
        _ => 35,
    };
    IconSource::Icons {
        col: weapon % ICON_COLS,
        row: weapon / ICON_COLS,
    }
}

fn passive_from_card(card: CardKind) -> Option<PassiveKind> {
    Some(match card {
        CardKind::PassiveSpinach => PassiveKind::Spinach,
        CardKind::PassiveArmor => PassiveKind::Armor,
        CardKind::PassiveHollowHeart => PassiveKind::HollowHeart,
        CardKind::PassivePummarola => PassiveKind::Pummarola,
        CardKind::PassiveEmptyTome => PassiveKind::EmptyTome,
        CardKind::PassiveCandelabrador => PassiveKind::Candelabrador,
        CardKind::PassiveBracer => PassiveKind::Bracer,
        CardKind::PassiveSpellbinder => PassiveKind::Spellbinder,
        CardKind::PassiveDuplicator => PassiveKind::Duplicator,
        CardKind::PassiveWings => PassiveKind::Wings,
        CardKind::PassiveAttractorb => PassiveKind::Attractorb,
        CardKind::PassiveClover => PassiveKind::Clover,
        CardKind::PassiveCrown => PassiveKind::Crown,
        CardKind::PassiveStoneMask => PassiveKind::StoneMask,
        CardKind::PassiveSkullOManiac => PassiveKind::SkullOManiac,
        CardKind::PassiveTiragisu => PassiveKind::Tiragisu,
        _ => return None,
    })
}

impl IconSource {
    fn texture_and_uv(self) -> (&'static str, UvRect) {
        match self {
            IconSource::Icons { col, row } => {
                (ICONS_PATH, flipped_grid_uv(col, row, ICON_COLS, ICON_ROWS))
            }
            IconSource::Passive(kind) => {
                let index = passive_index(kind);
                (PASSIVES_PATH, flipped_grid_uv(index % 8, index / 8, 8, 2))
            }
            IconSource::PowerUp(kind) => {
                let index = powerup_index(kind);
                (
                    POWERUPS_PATH,
                    flipped_grid_uv(index % 10, index / 10, 10, 2),
                )
            }
        }
    }
}

fn flipped_grid_uv(col: u32, row: u32, cols: u32, rows: u32) -> UvRect {
    let u_size = 1.0 / cols as f32;
    let v_size = 1.0 / rows as f32;
    UvRect {
        u_offset: col as f32 * u_size,
        v_offset: (row as f32 + 1.0) * v_size,
        u_size,
        v_size: -v_size,
    }
}

fn passive_index(kind: PassiveKind) -> u32 {
    match kind {
        PassiveKind::Spinach => 0,
        PassiveKind::Armor => 1,
        PassiveKind::HollowHeart => 2,
        PassiveKind::Pummarola => 3,
        PassiveKind::EmptyTome => 4,
        PassiveKind::Candelabrador => 5,
        PassiveKind::Bracer => 6,
        PassiveKind::Spellbinder => 7,
        PassiveKind::Duplicator => 8,
        PassiveKind::Wings => 9,
        PassiveKind::Attractorb => 10,
        PassiveKind::Clover => 11,
        PassiveKind::Crown => 12,
        PassiveKind::StoneMask => 13,
        PassiveKind::SkullOManiac => 14,
        PassiveKind::Tiragisu => 15,
    }
}

fn powerup_index(kind: PowerUpKind) -> u32 {
    PowerUpKind::ALL
        .iter()
        .position(|candidate| *candidate == kind)
        .unwrap_or(0) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flipped_icon_uv_points_inside_grid() {
        let uv = flipped_grid_uv(5, 1, 6, 6);

        assert!(uv.u_offset >= 0.0);
        assert!(uv.u_offset < 1.0);
        assert!(uv.v_offset > 0.0);
        assert!(uv.v_offset <= 1.0);
        assert!(uv.v_size < 0.0);
    }

    #[test]
    fn passive_and_powerup_indices_cover_generated_sheets() {
        assert_eq!(passive_index(PassiveKind::Spinach), 0);
        assert_eq!(passive_index(PassiveKind::Tiragisu), 15);
        assert_eq!(powerup_index(PowerUpKind::Might), 0);
        assert_eq!(powerup_index(PowerUpKind::Banish), 18);
    }

    #[test]
    fn levelup_passive_cards_use_passive_sheet() {
        let (texture, _) = card_icon(CardKind::PassiveSpinach).texture_and_uv();

        assert_eq!(texture, PASSIVES_PATH);
    }

    #[test]
    fn hud_icons_leave_text_room_at_800x600() {
        let layout = hud_icon_layout(800.0, 600.0);

        for slot in 0..SLOT_COLS {
            let center = hud_icon_center(&layout, slot, 0);
            let icon_right = center.x + layout.icon_size / 2.0;
            let text_x = hud_slot_text_x(&layout, slot);
            let slot_right =
                layout.slot_x0 + slot as f32 * (layout.slot_w + layout.slot_gap) + layout.slot_w;

            assert!(icon_right + 4.0 <= text_x);
            assert!(text_x < slot_right);
            assert!(center.y + layout.icon_size / 2.0 <= layout.passive_y);
        }
    }

    #[test]
    fn levelup_icons_stay_inside_card_text_margin() {
        let viewport_w = 800.0;
        let viewport_h = 600.0;
        let icon_size = levelup_icon_size(viewport_w, viewport_h);
        let text_x = levelup_text_x(viewport_w);

        for card in 0..3 {
            let center = levelup_icon_center(viewport_w, viewport_h, card);

            assert!(center.x - icon_size / 2.0 >= viewport_w / 2.0 - 235.0);
            assert!(center.x + icon_size / 2.0 + 8.0 <= text_x);
        }
    }

    #[test]
    fn shop_icons_do_not_overlap_powerup_text_at_800x600() {
        let viewport_w = 800.0;
        let viewport_h = 600.0;
        let icon_size = shop_icon_size(viewport_w, viewport_h);
        let text_x = shop_text_x(viewport_w);

        for row in 0..PowerUpKind::ALL.len() {
            let center = shop_icon_center(viewport_w, viewport_h, row);

            assert!(center.x - icon_size / 2.0 >= 0.0);
            assert!(center.x + icon_size / 2.0 + 6.0 <= text_x);
            assert!(center.y + icon_size / 2.0 <= viewport_h - 8.0);
        }
    }

    #[test]
    fn icon_sizes_are_capped_for_high_resolution_ui() {
        assert_eq!(hud_icon_layout(3840.0, 2160.0).icon_size, 32.0);
        assert_eq!(levelup_icon_size(3840.0, 2160.0), 36.0);
        assert_eq!(shop_icon_size(3840.0, 2160.0), 24.0);
    }
}
