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
use super::ui_layout::{HudSlotLayout, LevelUpLayout, ScreenRect, ShopLayout};
use engine::{DrawImage, GameState, System, UiImageQueue, UvRect, ViewportSize, World};
use glam::Vec2;

const SLOT_COLS: usize = 6;
const UI_BACKGROUND_Z: f32 = 30.0;
const UI_ROW_Z: f32 = 34.0;
const UI_ACCENT_Z: f32 = 36.0;
const UI_ICON_Z: f32 = 45.0;

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

    let layout = HudSlotLayout::new(viewport.width, viewport.height);
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
            layout.icon_center(i, 0),
            layout.icon_size,
            icon,
            UI_ICON_Z,
        );
    }

    for (i, kind) in passive_slots.iter().enumerate().take(SLOT_COLS) {
        queue_icon(
            world,
            layout.icon_center(i, 1),
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
            LevelUpLayout::new(viewport.width, viewport.height).icon_center(i),
            LevelUpLayout::new(viewport.width, viewport.height).icon_size(),
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
    let layout = ShopLayout::new(viewport.width, viewport.height);
    let visible_start = layout.visible_start(cursor_idx, PowerUpKind::ALL.len());
    let visible_cursor = cursor_idx.saturating_sub(visible_start);
    queue_screen_colored_rect(
        world,
        layout.selection_rect(visible_cursor),
        [0.25, 0.22, 0.05, 0.75],
        UI_ROW_Z,
    );

    for (visible_i, kind) in PowerUpKind::ALL
        .iter()
        .skip(visible_start)
        .take(layout.visible_rows())
        .enumerate()
    {
        queue_icon(
            world,
            layout.icon_center(visible_i),
            layout.icon_size(),
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
    layout: &HudSlotLayout,
    weapon_slots: &[(WeaponKind, bool)],
    passive_slots: &[PassiveKind],
) {
    queue_screen_colored_rect(
        world,
        layout.panel_rect(),
        [0.015, 0.015, 0.025, 0.84],
        UI_BACKGROUND_Z,
    );

    for row in 0..2 {
        for i in 0..SLOT_COLS {
            queue_screen_texture(
                world,
                layout.slot_skin_rect(i, row),
                UI_SLOT_FRAME_PATH,
                UI_ROW_Z,
            );
            queue_screen_colored_rect(
                world,
                layout.stripe_rect(i, row),
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
        ScreenRect::new(0.0, 0.0, viewport_w, viewport_h),
        [0.0, 0.0, 0.0, 0.55],
        UI_BACKGROUND_Z,
    );
    queue_screen_texture(
        world,
        LevelUpLayout::new(viewport_w, viewport_h).panel_skin_rect(),
        UI_MODAL_PANEL_PATH,
        UI_BACKGROUND_Z,
    );

    for card_index in 0..3 {
        queue_screen_colored_rect(
            world,
            LevelUpLayout::new(viewport_w, viewport_h).card_rect(card_index),
            [0.045, 0.043, 0.047, 0.82],
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
        assert!(image.x >= 12.0);
        assert!(image.y >= 24.0);
        assert!(image.x + image.w <= 60.0);
        assert!(image.y + image.h <= 40.0);
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
        let layout = HudSlotLayout::new(800.0, 600.0);

        for slot in 0..SLOT_COLS {
            let center = layout.icon_center(slot, 0);
            let icon_right = center.x + layout.icon_size / 2.0;
            let text_x = layout.text_x(slot);
            let slot_rect = layout.slot_rect(slot, 0);
            let slot_right = slot_rect.x + slot_rect.w;

            assert!(center.x > slot_rect.x);
            assert!(center.y > slot_rect.y);
            assert!(center.y < slot_rect.y + slot_rect.h);
            assert!(icon_right + 4.0 <= text_x);
            assert!(text_x < slot_right);
            assert!(center.y + layout.icon_size / 2.0 <= layout.slot_rect(slot, 1).y);
        }

        assert_ui_z_order();
    }

    #[test]
    fn levelup_icons_stay_inside_card_text_margin_at_common_resolutions() {
        for (viewport_w, viewport_h) in [(800.0, 600.0), (3840.0, 2160.0)] {
            let layout = LevelUpLayout::new(viewport_w, viewport_h);
            let icon_size = layout.icon_size();
            let text_x = layout.text_x();
            let panel_skin = layout.panel_skin_rect();

            assert!(panel_skin.x >= 0.0);
            assert!(panel_skin.x + panel_skin.w <= viewport_w);

            for card in 0..3 {
                let center = layout.icon_center(card);
                let row_rect = layout.card_rect(card);

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
            let layout = ShopLayout::new(viewport_w, viewport_h);
            let icon_size = layout.icon_size();
            let text_x = layout.text_x();
            let first_selection_skin = layout.selection_skin_rect(0);

            assert!(first_selection_skin.x >= 0.0);
            assert!(first_selection_skin.x + first_selection_skin.w <= viewport_w);

            for row in 0..layout.visible_rows() {
                let center = layout.icon_center(row);
                let selection_rect = layout.selection_rect(row);

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
        assert_eq!(HudSlotLayout::new(3840.0, 2160.0).icon_size, 32.0);
        assert_eq!(LevelUpLayout::new(3840.0, 2160.0).icon_size(), 36.0);
        assert_eq!(ShopLayout::new(3840.0, 2160.0).icon_size(), 24.0);
    }
}
