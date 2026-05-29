use super::icons::{weapon_icon, ICONS_PATH, ICON_COLS, ICON_ROWS};
use super::inventory::{WeaponInventory, WeaponKind};
use super::levelup::{CardKind, PendingLevelUp};
use super::meta::SurvivorMode;
use super::passive::{PassiveInventory, PassiveKind};
use super::player::Player;
use super::powerup::{PowerUpKind, ShopCursor};
use super::sprites::{
    survivor_texture_aspect, survivor_texture_handle, PASSIVES_PATH, POWERUPS_PATH,
    UI_MODAL_PANEL_PATH, UI_SLOT_FRAME_PATH,
};
use engine::{DrawImage, GameState, System, UiImageQueue, UvRect, ViewportSize, World};
use glam::Vec2;

const SLOT_COLS: usize = 6;
const SLOT_W: f32 = 152.0;
const SLOT_H: f32 = 38.0;
const SLOT_GAP: f32 = 6.0;
const XP_BAR_H: f32 = 14.0;
const ICON_SIZE: f32 = 28.0;
const LEVEL_ICON_SIZE: f32 = 34.0;
const SHOP_ICON_SIZE: f32 = 22.0;
const LEVELUP_ICON_X_OFFSET: f32 = -214.0;
const SHOP_ICON_X_OFFSET: f32 = -334.0;
const UI_BACKGROUND_Z: f32 = 30.0;
const UI_ROW_Z: f32 = 34.0;
const UI_ACCENT_Z: f32 = 36.0;
const UI_ICON_Z: f32 = 45.0;
#[cfg(test)]
const SLOT_TEXT_INSET: f32 = 46.0;
#[cfg(test)]
const SHOP_TEXT_X_OFFSET: f32 = -306.0;
#[cfg(test)]
const LEVELUP_TEXT_X_OFFSET: f32 = -178.0;

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
struct ScreenRect {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

impl ScreenRect {
    fn aspect_fit(self, aspect: f32) -> Self {
        let rect_aspect = self.w / self.h.max(1.0);
        if rect_aspect > aspect {
            let w = self.h * aspect;
            Self {
                x: self.x + (self.w - w) * 0.5,
                w,
                ..self
            }
        } else {
            let h = self.w / aspect.max(0.001);
            Self {
                y: self.y + (self.h - h) * 0.5,
                h,
                ..self
            }
        }
    }
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
        let Some(viewport) = world.resource::<ViewportSize>().copied() else {
            return;
        };
        let mode = world
            .resource::<SurvivorMode>()
            .copied()
            .unwrap_or_default();
        let is_levelup = matches!(world.resource::<GameState>(), Some(GameState::Paused));

        if matches!(mode, SurvivorMode::InGame) {
            queue_hud_slot_icons(world, viewport);
        }
        if is_levelup {
            queue_levelup_icons(world, viewport);
        }
        if matches!(mode, SurvivorMode::Shop) {
            queue_shop_icons(world, viewport);
        }
    }
}

fn queue_hud_slot_icons(world: &mut World, viewport: ViewportSize) {
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
    if weapon_slots.is_empty() && passive_slots.is_empty() {
        return;
    }

    queue_hud_slot_backgrounds(world, &layout, &weapon_slots, &passive_slots);

    for (i, (kind, evolved)) in weapon_slots.iter().enumerate().take(SLOT_COLS) {
        let icon = if *evolved {
            weapon_evolution_icon(kind)
        } else {
            weapon_source(kind)
        };
        queue_icon(
            world,
            hud_icon_center(&layout, i, 0),
            layout.icon_size,
            icon,
            UI_ICON_Z,
        );
    }

    for (i, kind) in passive_slots.iter().enumerate().take(SLOT_COLS) {
        queue_icon(
            world,
            hud_icon_center(&layout, i, 1),
            layout.icon_size,
            IconSource::Passive(*kind),
            UI_ICON_Z,
        );
    }
}

fn queue_levelup_icons(world: &mut World, viewport: ViewportSize) {
    let Some(offered) = world
        .resource::<PendingLevelUp>()
        .map(|pending| pending.offered)
    else {
        return;
    };

    queue_levelup_backgrounds(world, viewport);

    for (i, card) in offered.iter().enumerate() {
        queue_icon(
            world,
            levelup_icon_center(viewport.width, viewport.height, i),
            levelup_icon_size(viewport.width, viewport.height),
            card_icon(*card),
            UI_ICON_Z,
        );
    }
}

fn queue_shop_icons(world: &mut World, viewport: ViewportSize) {
    let mode = world
        .resource::<SurvivorMode>()
        .copied()
        .unwrap_or_default();
    if !matches!(mode, SurvivorMode::Shop) {
        return;
    }

    let cursor_idx = world.resource::<ShopCursor>().map(|c| c.index).unwrap_or(0);
    queue_screen_colored_rect(
        world,
        shop_selection_rect(viewport.width, viewport.height, cursor_idx),
        [0.25, 0.22, 0.05, 0.75],
        UI_ROW_Z,
    );
    queue_screen_texture(
        world,
        shop_selection_skin_rect(viewport.width, viewport.height, cursor_idx),
        UI_SLOT_FRAME_PATH,
        UI_ROW_Z + 1.0,
    );

    for (i, kind) in PowerUpKind::ALL.iter().enumerate() {
        queue_icon(
            world,
            shop_icon_center(viewport.width, viewport.height, i),
            shop_icon_size(viewport.width, viewport.height),
            IconSource::PowerUp(*kind),
            UI_ICON_Z,
        );
    }
}

fn queue_screen_colored_rect(world: &mut World, rect: ScreenRect, color: [f32; 4], z: f32) {
    if let Some(queue) = world.resource_mut::<UiImageQueue>() {
        queue.push(DrawImage::colored(rect.x, rect.y, rect.w, rect.h, color).with_z(z));
    }
}

fn queue_screen_texture(world: &mut World, rect: ScreenRect, path: &str, z: f32) {
    let image_rect = survivor_texture_aspect(path)
        .map(|aspect| rect.aspect_fit(aspect))
        .unwrap_or(rect);
    let handle = survivor_texture_handle(world, path);
    if let Some(queue) = world.resource_mut::<UiImageQueue>() {
        let backing = if path == UI_MODAL_PANEL_PATH {
            [0.022, 0.019, 0.021, 1.0]
        } else {
            [0.024, 0.022, 0.024, 1.0]
        };
        queue.push(
            DrawImage::colored(
                image_rect.x,
                image_rect.y,
                image_rect.w,
                image_rect.h,
                backing,
            )
            .with_z(z - 0.1),
        );
        queue.push(
            DrawImage::textured_with_handle(
                image_rect.x,
                image_rect.y,
                image_rect.w,
                image_rect.h,
                path,
                handle,
            )
            .with_z(z),
        );
    }
}

fn queue_hud_slot_backgrounds(
    world: &mut World,
    layout: &HudIconLayout,
    weapon_slots: &[(WeaponKind, bool)],
    passive_slots: &[PassiveKind],
) {
    queue_screen_colored_rect(
        world,
        hud_panel_rect(layout),
        [0.015, 0.015, 0.025, 0.84],
        UI_BACKGROUND_Z,
    );

    for row in 0..2 {
        for i in 0..SLOT_COLS {
            queue_screen_texture(
                world,
                hud_slot_skin_rect(layout, i, row),
                UI_SLOT_FRAME_PATH,
                UI_ROW_Z,
            );
            queue_screen_colored_rect(
                world,
                hud_stripe_rect(layout, i, row),
                hud_stripe_color(row, i, weapon_slots, passive_slots),
                UI_ACCENT_Z,
            );
        }
    }
}

fn queue_levelup_backgrounds(world: &mut World, viewport: ViewportSize) {
    let viewport_w = viewport.width;
    let viewport_h = viewport.height;
    queue_screen_colored_rect(
        world,
        ScreenRect {
            x: 0.0,
            y: 0.0,
            w: viewport_w,
            h: viewport_h,
        },
        [0.0, 0.0, 0.0, 0.55],
        UI_BACKGROUND_Z,
    );
    queue_screen_texture(
        world,
        levelup_panel_skin_rect(viewport_w, viewport_h),
        UI_MODAL_PANEL_PATH,
        UI_BACKGROUND_Z,
    );

    for card_index in 0..3 {
        queue_screen_texture(
            world,
            levelup_card_skin_rect(viewport_w, viewport_h, card_index),
            UI_SLOT_FRAME_PATH,
            UI_ROW_Z,
        );
    }
}

fn queue_icon(
    world: &mut World,
    screen_center: Vec2,
    screen_size: f32,
    source: IconSource,
    z: f32,
) {
    let (texture, uv) = source.texture_and_uv();
    let handle = survivor_texture_handle(world, texture);
    if let Some(queue) = world.resource_mut::<UiImageQueue>() {
        queue.push(
            DrawImage::textured_with_handle(
                screen_center.x - screen_size * 0.5,
                screen_center.y - screen_size * 0.5,
                screen_size,
                screen_size,
                texture,
                handle,
            )
            .with_uv(uv)
            .with_z(z),
        );
    }
}

fn hud_panel_width(layout: &HudIconLayout) -> f32 {
    SLOT_COLS as f32 * layout.slot_w + (SLOT_COLS - 1) as f32 * layout.slot_gap
}

fn hud_panel_rect(layout: &HudIconLayout) -> ScreenRect {
    ScreenRect {
        x: layout.slot_x0 - 4.0,
        y: layout.weapon_y - 26.0 * layout.ui_scale,
        w: hud_panel_width(layout) + 8.0,
        h: layout.slot_h * 2.0 + layout.slot_gap + 36.0 * layout.ui_scale,
    }
}

fn hud_slot_rect(layout: &HudIconLayout, slot_index: usize, row: usize) -> ScreenRect {
    let sx = layout.slot_x0 + slot_index as f32 * (layout.slot_w + layout.slot_gap);
    ScreenRect {
        x: sx,
        y: if row == 0 {
            layout.weapon_y
        } else {
            layout.passive_y
        },
        w: layout.slot_w,
        h: layout.slot_h,
    }
}

fn hud_slot_skin_rect(layout: &HudIconLayout, slot_index: usize, row: usize) -> ScreenRect {
    let slot = hud_slot_rect(layout, slot_index, row);
    ScreenRect {
        x: slot.x - 4.0 * layout.ui_scale,
        y: slot.y - 5.0 * layout.ui_scale,
        w: slot.w + 8.0 * layout.ui_scale,
        h: slot.h + 10.0 * layout.ui_scale,
    }
}

fn hud_stripe_rect(layout: &HudIconLayout, slot_index: usize, row: usize) -> ScreenRect {
    let slot = hud_slot_rect(layout, slot_index, row);
    ScreenRect {
        x: slot.x,
        y: slot.y,
        w: 7.0 * layout.ui_scale,
        h: slot.h,
    }
}

fn hud_stripe_color(
    row: usize,
    slot_index: usize,
    weapon_slots: &[(WeaponKind, bool)],
    passive_slots: &[PassiveKind],
) -> [f32; 4] {
    if row == 0 {
        weapon_slots
            .get(slot_index)
            .map(|(kind, _)| weapon_icon_color(kind))
            .unwrap_or([0.18, 0.18, 0.22, 1.0])
    } else {
        passive_slots
            .get(slot_index)
            .map(|kind| passive_icon_color(*kind))
            .unwrap_or([0.18, 0.18, 0.22, 1.0])
    }
}

fn weapon_icon_color(kind: &WeaponKind) -> [f32; 4] {
    match kind {
        WeaponKind::Whip { .. } => [0.5, 0.9, 0.3, 1.0],
        WeaponKind::MagicWand { .. } => [0.3, 0.5, 1.0, 1.0],
        WeaponKind::Knife { .. } => [0.7, 0.7, 0.7, 1.0],
        WeaponKind::Axe { .. } => [0.8, 0.5, 0.2, 1.0],
        WeaponKind::Cross { .. } => [1.0, 1.0, 0.8, 1.0],
        WeaponKind::FireWand { .. } => [1.0, 0.4, 0.1, 1.0],
        WeaponKind::Garlic { .. } => [0.9, 0.9, 0.4, 1.0],
        WeaponKind::HolyWater { .. } => [0.4, 0.7, 1.0, 1.0],
        WeaponKind::KingBible { .. } => [0.9, 0.7, 0.2, 1.0],
        WeaponKind::LightningRing { .. } => [1.0, 1.0, 0.3, 1.0],
    }
}

fn passive_icon_color(kind: PassiveKind) -> [f32; 4] {
    match kind {
        PassiveKind::Spinach => [0.3, 0.8, 0.3, 1.0],
        PassiveKind::Armor => [0.6, 0.6, 0.7, 1.0],
        PassiveKind::HollowHeart => [0.9, 0.3, 0.3, 1.0],
        PassiveKind::Pummarola => [1.0, 0.4, 0.6, 1.0],
        PassiveKind::EmptyTome => [0.7, 0.5, 0.9, 1.0],
        PassiveKind::Candelabrador => [1.0, 0.8, 0.2, 1.0],
        PassiveKind::Bracer => [0.8, 0.6, 0.4, 1.0],
        PassiveKind::Spellbinder => [0.5, 0.5, 0.9, 1.0],
        PassiveKind::Duplicator => [0.6, 0.9, 0.6, 1.0],
        PassiveKind::Wings => [0.9, 0.7, 1.0, 1.0],
        PassiveKind::Attractorb => [0.3, 0.9, 0.9, 1.0],
        PassiveKind::Clover => [0.2, 0.8, 0.4, 1.0],
        PassiveKind::Crown => [1.0, 0.8, 0.1, 1.0],
        PassiveKind::StoneMask => [0.5, 0.5, 0.5, 1.0],
        PassiveKind::SkullOManiac => [0.4, 0.2, 0.4, 1.0],
        PassiveKind::Tiragisu => [1.0, 0.5, 0.5, 1.0],
    }
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
    Vec2::new(
        cx + LEVELUP_ICON_X_OFFSET,
        cy - 35.0 + card_index as f32 * 56.0,
    )
}

fn levelup_panel_rect(viewport_w: f32, viewport_h: f32) -> ScreenRect {
    let cx = viewport_w / 2.0;
    let cy = viewport_h / 2.0;
    ScreenRect {
        x: cx - 250.0,
        y: cy - 150.0,
        w: 500.0,
        h: 300.0,
    }
}

fn levelup_panel_skin_rect(viewport_w: f32, viewport_h: f32) -> ScreenRect {
    let panel = levelup_panel_rect(viewport_w, viewport_h);
    ScreenRect {
        x: panel.x - 24.0,
        y: panel.y - 20.0,
        w: panel.w + 48.0,
        h: panel.h + 40.0,
    }
}

fn levelup_card_rect(viewport_w: f32, viewport_h: f32, card_index: usize) -> ScreenRect {
    let cx = viewport_w / 2.0;
    let cy = viewport_h / 2.0;
    ScreenRect {
        x: cx - 235.0,
        y: cy - 58.0 + card_index as f32 * 56.0,
        w: 470.0,
        h: 46.0,
    }
}

fn levelup_card_skin_rect(viewport_w: f32, viewport_h: f32, card_index: usize) -> ScreenRect {
    let card = levelup_card_rect(viewport_w, viewport_h, card_index);
    ScreenRect {
        x: card.x - 16.0,
        y: card.y - 10.0,
        w: card.w + 32.0,
        h: card.h + 20.0,
    }
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
        cx + SHOP_ICON_X_OFFSET,
        90.0 + row_index as f32 * shop_row_step(viewport_h),
    )
}

fn shop_icon_size(viewport_w: f32, viewport_h: f32) -> f32 {
    (SHOP_ICON_SIZE * responsive_ui_scale(viewport_w, viewport_h)).clamp(18.0, 24.0)
}

fn shop_selection_rect(viewport_w: f32, viewport_h: f32, row_index: usize) -> ScreenRect {
    let cx = viewport_w / 2.0;
    let row_step = shop_row_step(viewport_h);
    ScreenRect {
        x: cx - 360.0,
        y: 74.0 + row_index as f32 * row_step,
        w: 720.0,
        h: 26.0,
    }
}

fn shop_selection_skin_rect(viewport_w: f32, viewport_h: f32, row_index: usize) -> ScreenRect {
    let row = shop_selection_rect(viewport_w, viewport_h, row_index);
    ScreenRect {
        x: row.x - 12.0,
        y: row.y - 7.0,
        w: row.w + 24.0,
        h: row.h + 14.0,
    }
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
            IconSource::Icons { col, row } => (
                ICONS_PATH,
                UvRect::from_grid(col, row, ICON_COLS, ICON_ROWS),
            ),
            IconSource::Passive(kind) => {
                let index = passive_index(kind);
                (PASSIVES_PATH, UvRect::from_grid(index % 8, index / 8, 8, 2))
            }
            IconSource::PowerUp(kind) => {
                let index = powerup_index(kind);
                (
                    POWERUPS_PATH,
                    UvRect::from_grid(index % 10, index / 10, 10, 2),
                )
            }
        }
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

    fn assert_ui_z_order() {
        let background = std::hint::black_box(UI_BACKGROUND_Z);
        let row = std::hint::black_box(UI_ROW_Z);
        let accent = std::hint::black_box(UI_ACCENT_Z);
        let icon = std::hint::black_box(UI_ICON_Z);

        assert!(icon > row);
        assert!(accent > row);
        assert!(row > background);
    }

    #[test]
    fn icon_uv_points_inside_grid_with_top_left_origin() {
        let uv = UvRect::from_grid(5, 1, 6, 6);

        assert!(uv.u_offset >= 0.0);
        assert!(uv.u_offset < 1.0);
        assert!(uv.v_offset > 0.0);
        assert!(uv.v_offset <= 1.0);
        assert!(uv.v_size > 0.0);
        assert!((uv.u_offset - 5.0 / 6.0).abs() < f32::EPSILON);
        assert!((uv.v_offset - 1.0 / 6.0).abs() < f32::EPSILON);
        assert!((uv.u_size - 1.0 / 6.0).abs() < f32::EPSILON);
        assert!((uv.v_size - 1.0 / 6.0).abs() < f32::EPSILON);
    }

    #[test]
    fn queue_icon_pushes_screen_space_draw_image() {
        let mut world = World::new();
        world.insert_resource(UiImageQueue::default());

        queue_icon(
            &mut world,
            Vec2::new(100.0, 80.0),
            32.0,
            IconSource::Icons { col: 0, row: 0 },
            UI_ICON_Z,
        );

        let queue = world.resource::<UiImageQueue>().unwrap();
        assert_eq!(queue.items.len(), 1);
        let image = &queue.items[0];
        assert_eq!(image.x, 84.0);
        assert_eq!(image.y, 64.0);
        assert_eq!(image.w, 32.0);
        assert_eq!(image.h, 32.0);
        assert_eq!(image.z, UI_ICON_Z);
        assert_eq!(image.texture.as_deref(), Some(ICONS_PATH));
        assert!(image.image_handle.is_none());
        assert!(image.uv.v_size > 0.0);
    }

    #[test]
    fn queue_colored_rect_pushes_ui_image_background() {
        let mut world = World::new();
        world.insert_resource(UiImageQueue::default());

        queue_screen_colored_rect(
            &mut world,
            ScreenRect {
                x: 12.0,
                y: 24.0,
                w: 48.0,
                h: 16.0,
            },
            [0.1, 0.2, 0.3, 0.4],
            UI_ROW_Z,
        );

        let queue = world.resource::<UiImageQueue>().unwrap();
        assert_eq!(queue.items.len(), 1);
        let image = &queue.items[0];
        assert_eq!(image.x, 12.0);
        assert_eq!(image.y, 24.0);
        assert_eq!(image.w, 48.0);
        assert_eq!(image.h, 16.0);
        assert_eq!(image.color, [0.1, 0.2, 0.3, 0.4]);
        assert_eq!(image.z, UI_ROW_Z);
        assert!(image.texture.is_none());
    }

    #[test]
    fn queued_screen_texture_uses_default_full_uv() {
        let mut world = World::new();
        world.insert_resource(UiImageQueue::default());

        queue_screen_texture(
            &mut world,
            ScreenRect {
                x: 12.0,
                y: 24.0,
                w: 48.0,
                h: 16.0,
            },
            UI_SLOT_FRAME_PATH,
            UI_ROW_Z,
        );

        let queue = world.resource::<UiImageQueue>().unwrap();
        assert_eq!(queue.items.len(), 2);
        let image = &queue.items[1];
        assert_eq!(image.texture.as_deref(), Some(UI_SLOT_FRAME_PATH));
        assert_eq!(image.uv, UvRect::FULL);
    }

    #[test]
    fn queued_screen_texture_preserves_source_aspect() {
        let mut world = World::new();
        world.insert_resource(UiImageQueue::default());

        queue_screen_texture(
            &mut world,
            ScreenRect {
                x: 12.0,
                y: 24.0,
                w: 48.0,
                h: 16.0,
            },
            UI_SLOT_FRAME_PATH,
            UI_ROW_Z,
        );

        let queue = world.resource::<UiImageQueue>().unwrap();
        let image = &queue.items[1];
        let aspect = survivor_texture_aspect(UI_SLOT_FRAME_PATH).unwrap();
        assert!((image.w / image.h - aspect).abs() < 0.001);
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
            let slot_rect = hud_slot_rect(&layout, slot, 0);
            let slot_right = slot_rect.x + slot_rect.w;

            assert!(center.x > slot_rect.x);
            assert!(center.y > slot_rect.y);
            assert!(center.y < slot_rect.y + slot_rect.h);
            assert!(icon_right + 4.0 <= text_x);
            assert!(text_x < slot_right);
            assert!(center.y + layout.icon_size / 2.0 <= layout.passive_y);
        }

        assert_ui_z_order();
    }

    #[test]
    fn levelup_icons_stay_inside_card_text_margin_at_common_resolutions() {
        for (viewport_w, viewport_h) in [(800.0, 600.0), (3840.0, 2160.0)] {
            let icon_size = levelup_icon_size(viewport_w, viewport_h);
            let text_x = levelup_text_x(viewport_w);
            let panel_skin = levelup_panel_skin_rect(viewport_w, viewport_h);

            assert!(panel_skin.x >= 0.0);
            assert!(panel_skin.x + panel_skin.w <= viewport_w);

            for card in 0..3 {
                let center = levelup_icon_center(viewport_w, viewport_h, card);
                let row_rect = levelup_card_rect(viewport_w, viewport_h, card);

                assert!(center.x > row_rect.x);
                assert!(center.y > row_rect.y);
                assert!(center.y < row_rect.y + row_rect.h);
                assert!(center.x - icon_size / 2.0 >= row_rect.x);
                assert!(center.x + icon_size / 2.0 + 10.0 <= text_x);
            }
        }

        assert_ui_z_order();
    }

    #[test]
    fn shop_icons_do_not_overlap_powerup_text_at_common_resolutions() {
        for (viewport_w, viewport_h) in [(800.0, 600.0), (3840.0, 2160.0)] {
            let icon_size = shop_icon_size(viewport_w, viewport_h);
            let text_x = shop_text_x(viewport_w);
            let first_selection_skin = shop_selection_skin_rect(viewport_w, viewport_h, 0);

            assert!(first_selection_skin.x >= 0.0);
            assert!(first_selection_skin.x + first_selection_skin.w <= viewport_w);

            for row in 0..PowerUpKind::ALL.len() {
                let center = shop_icon_center(viewport_w, viewport_h, row);
                let selection_rect = shop_selection_rect(viewport_w, viewport_h, row);

                assert!(center.x > selection_rect.x);
                assert!(center.y > selection_rect.y);
                assert!(center.y < selection_rect.y + selection_rect.h);
                assert!(center.x - icon_size / 2.0 >= 0.0);
                assert!(center.x + icon_size / 2.0 + 10.0 <= text_x);
                assert!(center.y + icon_size / 2.0 <= viewport_h - 8.0);
            }
        }

        assert_ui_z_order();
    }

    #[test]
    fn icon_sizes_are_capped_for_high_resolution_ui() {
        assert_eq!(hud_icon_layout(3840.0, 2160.0).icon_size, 32.0);
        assert_eq!(levelup_icon_size(3840.0, 2160.0), 36.0);
        assert_eq!(shop_icon_size(3840.0, 2160.0), 24.0);
    }
}
