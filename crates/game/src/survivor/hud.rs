use super::achievement::{achievement_completed, AchievementKind};
use super::boss::Boss;
use super::character::{CharacterCursor, CharacterKind};
use super::damage_number::DamageNumber;
use super::debug_input::DebugOverlay;
use super::health::Health;
use super::inventory::{WeaponInventory, WeaponKind};
use super::levelup::PendingLevelUp;
use super::locale::{loc, text, Lang, UiText};
use super::meta::{
    AchievementCursor, HudDetail, MetaSave, PauseMenuCursor, ResolutionPreset, SettingsCursor,
    SurvivorMode, ACHIEVEMENTS_PER_PAGE, PAUSE_MENU_ITEMS, SETTINGS_ITEMS,
};
use super::passive::{PassiveInventory, PassiveKind};
use super::pickup::GoldWallet;
use super::player::{Player, PlayerStats};
use super::powerup::{PowerUpKind, ShopCursor};
use super::sprites::{
    survivor_texture_aspect, survivor_texture_handle, GAMEOVER_TITLE_EN_PATH,
    GAMEOVER_TITLE_KO_PATH, INGAME_LABEL_GOLD_EN_PATH, INGAME_LABEL_GOLD_KO_PATH,
    INGAME_LABEL_HP_EN_PATH, INGAME_LABEL_HP_KO_PATH, INGAME_LABEL_KILLS_EN_PATH,
    INGAME_LABEL_KILLS_KO_PATH, INGAME_LABEL_LV_EN_PATH, INGAME_LABEL_LV_KO_PATH,
    INGAME_LABEL_PASSIVES_EN_PATH, INGAME_LABEL_PASSIVES_KO_PATH, INGAME_LABEL_XP_EN_PATH,
    INGAME_LABEL_XP_KO_PATH, LEVELUP_TITLE_EN_PATH, LEVELUP_TITLE_KO_PATH, RESTART_HINT_EN_PATH,
    RESTART_HINT_KO_PATH, SECTION_PASSIVES_EN_PATH, SECTION_PASSIVES_KO_PATH,
    SECTION_WEAPONS_EN_PATH, SECTION_WEAPONS_KO_PATH, UI_MODAL_PANEL_PATH, UI_SLOT_FRAME_PATH,
};
use super::stage::{StageCursor, StageKind};
use super::xp::XpAccumulator;
use engine::renderer::text::{DrawText, TextQueue};
use engine::{
    Camera, DrawImage, DrawRect, GameState, System, Transform, UiImageQueue, UiQueue, ViewportSize,
    World,
};
use glam::Vec2;

// ── 바 크기 상수 ─────────────────────────────────────────────────────────────
const HP_BAR_W: f32 = 280.0;
const HP_BAR_H: f32 = 24.0;
const XP_BAR_H: f32 = 14.0;
const BOSS_BAR_W: f32 = 460.0;
const BOSS_BAR_H: f32 = 22.0;
const SLOT_COLS: usize = 6;
const SLOT_W: f32 = 152.0;
const SLOT_H: f32 = 38.0;
const SLOT_GAP: f32 = 6.0;
const HUD_X: f32 = 16.0;
const HUD_Y: f32 = 14.0;
const BOSS_LABEL_HEIGHT: f32 = 24.0;
const BOSS_LABEL_PADDING: f32 = 12.0;
const LOCKED_TEXT_COLOR: [u8; 4] = [165, 165, 175, 255];
const HELP_TEXT_COLOR: [u8; 4] = [190, 190, 195, 255];
const UI_PANEL_IMAGE_Z: f32 = 24.0;
const UI_ROW_IMAGE_Z: f32 = 34.0;
const HUD_LABEL_BASE_H: f32 = 24.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IngameLabelKind {
    Lv,
    Hp,
    Xp,
    Gold,
    Kills,
    Passives,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IngameSectionKind {
    Weapons,
    Passives,
}

#[derive(Debug, Clone, PartialEq)]
struct HudLabelValue {
    label: IngameLabelKind,
    value: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct HudLabelLayout {
    label_x: f32,
    value_x: f32,
    width: f32,
}

fn ingame_label_path(kind: IngameLabelKind, lang: Lang) -> &'static str {
    match (kind, lang) {
        (IngameLabelKind::Lv, Lang::Ko) => INGAME_LABEL_LV_KO_PATH,
        (IngameLabelKind::Lv, Lang::En) => INGAME_LABEL_LV_EN_PATH,
        (IngameLabelKind::Hp, Lang::Ko) => INGAME_LABEL_HP_KO_PATH,
        (IngameLabelKind::Hp, Lang::En) => INGAME_LABEL_HP_EN_PATH,
        (IngameLabelKind::Xp, Lang::Ko) => INGAME_LABEL_XP_KO_PATH,
        (IngameLabelKind::Xp, Lang::En) => INGAME_LABEL_XP_EN_PATH,
        (IngameLabelKind::Gold, Lang::Ko) => INGAME_LABEL_GOLD_KO_PATH,
        (IngameLabelKind::Gold, Lang::En) => INGAME_LABEL_GOLD_EN_PATH,
        (IngameLabelKind::Kills, Lang::Ko) => INGAME_LABEL_KILLS_KO_PATH,
        (IngameLabelKind::Kills, Lang::En) => INGAME_LABEL_KILLS_EN_PATH,
        (IngameLabelKind::Passives, Lang::Ko) => INGAME_LABEL_PASSIVES_KO_PATH,
        (IngameLabelKind::Passives, Lang::En) => INGAME_LABEL_PASSIVES_EN_PATH,
    }
}

fn ingame_label_width(_kind: IngameLabelKind, _lang: Lang, scale: f32) -> f32 {
    let aspect = survivor_texture_aspect(INGAME_LABEL_LV_KO_PATH).unwrap_or(240.0 / 78.0);
    HUD_LABEL_BASE_H * aspect * scale
}

fn estimated_text_width(value: &str, font_size: f32) -> f32 {
    value.chars().count() as f32 * font_size * 0.54
}

fn hud_label_layouts(
    start_x: f32,
    lang: Lang,
    scale: f32,
    font_size: f32,
    values: &[HudLabelValue],
) -> Vec<HudLabelLayout> {
    let gap = 12.0 * scale;
    let label_value_gap = 5.0 * scale;
    let mut x = start_x;
    values
        .iter()
        .map(|item| {
            let label_w = ingame_label_width(item.label, lang, scale);
            let value_w = estimated_text_width(&item.value, font_size);
            let width = label_w + label_value_gap + value_w + gap;
            let layout = HudLabelLayout {
                label_x: x,
                value_x: x + label_w + label_value_gap,
                width,
            };
            x += width;
            layout
        })
        .collect()
}

#[cfg(test)]
fn hud_label_row_width(lang: Lang, scale: f32, font_size: f32, values: &[HudLabelValue]) -> f32 {
    hud_label_layouts(0.0, lang, scale, font_size, values)
        .last()
        .map(|layout| layout.label_x + layout.width)
        .unwrap_or(0.0)
}

fn queue_ingame_ui_image(world: &mut World, rect: (f32, f32, f32, f32), path: &str, z: f32) {
    queue_ui_texture(world, rect.0, rect.1, rect.2, rect.3, path, z);
}

fn aspect_fit_rect((x, y, w, h): (f32, f32, f32, f32), aspect: f32) -> (f32, f32, f32, f32) {
    let rect_aspect = w / h.max(1.0);
    if rect_aspect > aspect {
        let fitted_w = h * aspect;
        (x + (w - fitted_w) * 0.5, y, fitted_w, h)
    } else {
        let fitted_h = w / aspect.max(0.001);
        (x, y + (h - fitted_h) * 0.5, w, fitted_h)
    }
}

fn section_label_path(kind: IngameSectionKind, lang: Lang) -> &'static str {
    match (kind, lang) {
        (IngameSectionKind::Weapons, Lang::Ko) => SECTION_WEAPONS_KO_PATH,
        (IngameSectionKind::Weapons, Lang::En) => SECTION_WEAPONS_EN_PATH,
        (IngameSectionKind::Passives, Lang::Ko) => SECTION_PASSIVES_KO_PATH,
        (IngameSectionKind::Passives, Lang::En) => SECTION_PASSIVES_EN_PATH,
    }
}

fn levelup_title_path(lang: Lang) -> &'static str {
    match lang {
        Lang::Ko => LEVELUP_TITLE_KO_PATH,
        Lang::En => LEVELUP_TITLE_EN_PATH,
    }
}

fn gameover_title_path(lang: Lang) -> &'static str {
    match lang {
        Lang::Ko => GAMEOVER_TITLE_KO_PATH,
        Lang::En => GAMEOVER_TITLE_EN_PATH,
    }
}

fn restart_hint_path(lang: Lang) -> &'static str {
    match lang {
        Lang::Ko => RESTART_HINT_KO_PATH,
        Lang::En => RESTART_HINT_EN_PATH,
    }
}

fn responsive_ui_scale(viewport_w: f32, viewport_h: f32) -> f32 {
    (viewport_w / 1280.0)
        .min(viewport_h / 720.0)
        .clamp(0.72, 1.5)
}

fn queue_ui_texture(world: &mut World, x: f32, y: f32, w: f32, h: f32, path: &str, z: f32) {
    let (x, y, w, h) = survivor_texture_aspect(path)
        .map(|aspect| aspect_fit_rect((x, y, w, h), aspect))
        .unwrap_or((x, y, w, h));
    let handle = survivor_texture_handle(world, path);
    if let Some(queue) = world.resource_mut::<UiImageQueue>() {
        queue.push(DrawImage::textured_with_handle(x, y, w, h, path, handle).with_z(z));
    }
}

fn queue_ui_colored_image(
    world: &mut World,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    color: [f32; 4],
    z: f32,
) {
    if let Some(queue) = world.resource_mut::<UiImageQueue>() {
        queue.push(DrawImage::colored(x, y, w, h, color).with_z(z));
    }
}

fn queue_modal_panel(world: &mut World, x: f32, y: f32, w: f32, h: f32, z: f32) {
    let (x, y, w, h) = survivor_texture_aspect(UI_MODAL_PANEL_PATH)
        .map(|aspect| aspect_fit_rect((x, y, w, h), aspect))
        .unwrap_or((x, y, w, h));
    queue_ui_colored_image(world, x, y, w, h, [0.022, 0.019, 0.021, 1.0], z - 0.1);
    queue_ui_texture(world, x, y, w, h, UI_MODAL_PANEL_PATH, z);
}

fn queue_slot_frame(world: &mut World, x: f32, y: f32, w: f32, h: f32, z: f32) {
    let (x, y, w, h) = survivor_texture_aspect(UI_SLOT_FRAME_PATH)
        .map(|aspect| aspect_fit_rect((x, y, w, h), aspect))
        .unwrap_or((x, y, w, h));
    queue_ui_colored_image(world, x, y, w, h, [0.024, 0.022, 0.024, 1.0], z - 0.1);
    queue_ui_texture(world, x, y, w, h, UI_SLOT_FRAME_PATH, z);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn responsive_ui_scale_tracks_actual_viewport() {
        assert!((responsive_ui_scale(1280.0, 720.0) - 1.0).abs() < 0.001);
        assert!(responsive_ui_scale(800.0, 600.0) < 1.0);
        assert!(responsive_ui_scale(1920.0, 1080.0) > 1.0);
    }

    #[test]
    fn responsive_ui_scale_uses_smaller_axis_for_mismatched_aspect() {
        let wide_short = responsive_ui_scale(1920.0, 600.0);
        let baseline = responsive_ui_scale(1280.0, 720.0);

        assert!(
            wide_short < baseline,
            "short viewport height should shrink UI even when viewport is wide"
        );
    }

    #[test]
    fn queued_ui_texture_uses_default_full_uv() {
        let mut world = World::new();
        world.insert_resource(UiImageQueue::default());

        queue_ui_texture(
            &mut world,
            10.0,
            20.0,
            100.0,
            40.0,
            UI_MODAL_PANEL_PATH,
            3.0,
        );

        let queue = world.resource::<UiImageQueue>().unwrap();
        assert_eq!(queue.items.len(), 1);
        assert_eq!(queue.items[0].texture.as_deref(), Some(UI_MODAL_PANEL_PATH));
        assert_eq!(queue.items[0].uv, engine::UvRect::FULL);
    }

    #[test]
    fn queued_ui_texture_preserves_source_aspect() {
        let mut world = World::new();
        world.insert_resource(UiImageQueue::default());

        queue_ui_texture(
            &mut world,
            10.0,
            20.0,
            100.0,
            40.0,
            UI_MODAL_PANEL_PATH,
            3.0,
        );

        let queue = world.resource::<UiImageQueue>().unwrap();
        let image = &queue.items[0];
        let aspect = survivor_texture_aspect(UI_MODAL_PANEL_PATH).unwrap();
        assert!((image.w / image.h - aspect).abs() < 0.001);
    }

    #[test]
    fn ingame_static_ui_paths_follow_language() {
        assert_eq!(
            ingame_label_path(IngameLabelKind::Kills, Lang::Ko),
            INGAME_LABEL_KILLS_KO_PATH
        );
        assert_eq!(
            ingame_label_path(IngameLabelKind::Gold, Lang::En),
            INGAME_LABEL_GOLD_EN_PATH
        );
        assert_eq!(
            section_label_path(IngameSectionKind::Weapons, Lang::Ko),
            SECTION_WEAPONS_KO_PATH
        );
        assert_eq!(levelup_title_path(Lang::En), LEVELUP_TITLE_EN_PATH);
        assert_eq!(gameover_title_path(Lang::Ko), GAMEOVER_TITLE_KO_PATH);
        assert_eq!(restart_hint_path(Lang::En), RESTART_HINT_EN_PATH);
    }

    #[test]
    fn hud_label_rows_fit_supported_viewports() {
        let compact_scale = responsive_ui_scale(800.0, 600.0);
        let compact_values = vec![
            HudLabelValue {
                label: IngameLabelKind::Lv,
                value: "12".to_string(),
            },
            HudLabelValue {
                label: IngameLabelKind::Hp,
                value: "83/100".to_string(),
            },
            HudLabelValue {
                label: IngameLabelKind::Xp,
                value: "12/35".to_string(),
            },
            HudLabelValue {
                label: IngameLabelKind::Kills,
                value: "999".to_string(),
            },
        ];
        assert!(
            HUD_X * compact_scale
                + estimated_text_width("59:59", 20.0 * compact_scale)
                + 14.0 * compact_scale
                + hud_label_row_width(
                    Lang::En,
                    compact_scale,
                    20.0 * compact_scale,
                    &compact_values
                )
                < 800.0
        );

        let wide_scale = responsive_ui_scale(1280.0, 720.0);
        let wide_values = vec![
            HudLabelValue {
                label: IngameLabelKind::Lv,
                value: "99".to_string(),
            },
            HudLabelValue {
                label: IngameLabelKind::Hp,
                value: "999/999".to_string(),
            },
            HudLabelValue {
                label: IngameLabelKind::Xp,
                value: "99/999".to_string(),
            },
            HudLabelValue {
                label: IngameLabelKind::Gold,
                value: "99999".to_string(),
            },
            HudLabelValue {
                label: IngameLabelKind::Passives,
                value: "6".to_string(),
            },
            HudLabelValue {
                label: IngameLabelKind::Kills,
                value: "9999".to_string(),
            },
        ];
        assert!(
            HUD_X
                + estimated_text_width("59:59", 19.0 * wide_scale)
                + 14.0 * wide_scale
                + hud_label_row_width(Lang::En, wide_scale, 19.0 * wide_scale, &wide_values)
                < 1280.0
        );
    }

    #[test]
    fn title_hud_does_not_queue_text() {
        let mut world = World::new();
        world.insert_resource(TextQueue::default());

        draw_title_hud(&mut world, 800.0, 600.0, Lang::Ko, true);

        let queue = world.resource::<TextQueue>().unwrap();
        assert!(queue.is_empty());
    }

    #[test]
    fn achievements_hud_keeps_gold_out_of_summary() {
        let mut world = World::new();
        world.insert_resource(TextQueue::default());
        world.insert_resource(UiQueue::default());
        world.insert_resource(UiImageQueue::default());
        world.insert_resource(MetaSave {
            gold_total: 999,
            kills_total: 5,
            best_time: 61.0,
            ..Default::default()
        });

        draw_achievements_hud(&mut world, 800.0, 600.0, Lang::En);

        let queue = world.resource::<TextQueue>().unwrap();
        assert!(queue.iter().any(|item| item.text.contains("Best 01:01")));
        assert!(queue.iter().any(|item| item.text.contains("Kills 5")));
        assert!(!queue.iter().any(|item| item.text.contains("Gold")));
        assert!(!queue.iter().any(|item| item.text.contains("999")));
    }

    #[test]
    fn compact_boss_bar_stays_below_top_hud() {
        let scale = responsive_ui_scale(800.0, 600.0);
        let top_hud_bottom = top_hud_panel_bottom(true, HudDetail::Normal, scale, 800.0);
        let boss_label_y =
            boss_bar_y(true, HudDetail::Normal, scale, 800.0) - BOSS_LABEL_HEIGHT * scale;

        assert!(boss_label_y >= top_hud_bottom + BOSS_LABEL_PADDING * scale);
    }

    #[test]
    fn detailed_compact_boss_bar_stays_below_expanded_hud() {
        let scale = responsive_ui_scale(800.0, 600.0);
        let top_hud_bottom = top_hud_panel_bottom(true, HudDetail::Detailed, scale, 800.0);
        let boss_label_y =
            boss_bar_y(true, HudDetail::Detailed, scale, 800.0) - BOSS_LABEL_HEIGHT * scale;

        assert!(boss_label_y >= top_hud_bottom + BOSS_LABEL_PADDING * scale);
    }

    #[test]
    fn default_boss_bar_stays_below_top_hud() {
        let scale = responsive_ui_scale(1280.0, 720.0);
        let top_hud_bottom = top_hud_panel_bottom(false, HudDetail::Normal, scale, 1280.0);
        let boss_label_y =
            boss_bar_y(false, HudDetail::Normal, scale, 1280.0) - BOSS_LABEL_HEIGHT * scale;

        assert!(boss_label_y >= top_hud_bottom + BOSS_LABEL_PADDING * scale);
    }

    #[test]
    fn default_detailed_boss_bar_stays_below_top_hud() {
        let scale = responsive_ui_scale(1280.0, 720.0);
        let top_hud_bottom = top_hud_panel_bottom(false, HudDetail::Detailed, scale, 1280.0);
        let boss_label_y =
            boss_bar_y(false, HudDetail::Detailed, scale, 1280.0) - BOSS_LABEL_HEIGHT * scale;

        assert!(boss_label_y >= top_hud_bottom + BOSS_LABEL_PADDING * scale);
    }
}

fn compact_stats_line(lang: Lang, stats: &PlayerStats) -> String {
    format!(
        "{} {:.0}%  {} {:.0}%  {} {:.0}%  {} +{}  {} {:.0}%",
        loc(lang, "공", "Mgt"),
        stats.might * 100.0,
        loc(lang, "쿨", "Cd"),
        stats.cooldown * 100.0,
        loc(lang, "범", "Area"),
        stats.area * 100.0,
        loc(lang, "수", "Amt"),
        stats.amount,
        loc(lang, "속", "Spd"),
        stats.move_speed * 100.0
    )
}

fn compact_stats_lines(lang: Lang, stats: &PlayerStats) -> (String, String) {
    (
        format!(
            "{} {:.0}%  {} {:.0}%",
            loc(lang, "공", "Mgt"),
            stats.might * 100.0,
            loc(lang, "쿨", "Cd"),
            stats.cooldown * 100.0
        ),
        format!(
            "{} {:.0}%  {} +{}  {} {:.0}%",
            loc(lang, "범", "Area"),
            stats.area * 100.0,
            loc(lang, "수", "Amt"),
            stats.amount,
            loc(lang, "속", "Spd"),
            stats.move_speed * 100.0
        ),
    )
}

/// 게임 진행 통계. 매 프레임 누적/조회.
#[derive(Debug, Default)]
pub struct GameStats {
    pub elapsed: f32, // 누적 게임 시간(초). Playing 중에만 증가.
    pub kills: u32,   // 누적 처치 수.
}

// ── 시각적 피드백 헬퍼 ───────────────────────────────────────────────────────

/// HP 비율(0.0~1.0)에 따라 초록→노랑→빨강 색상 반환
fn hp_color(ratio: f32) -> [f32; 4] {
    let r = ratio.clamp(0.0, 1.0);
    if r > 0.5 {
        let t = 1.0 - (r - 0.5) / 0.5; // 0=초록, 1=노랑
        [0.1 + t * 0.8, 0.8, 0.1, 1.0]
    } else {
        let t = 1.0 - r / 0.5; // 0=노랑, 1=빨강
        [0.9, 0.8 - t * 0.7, 0.1, 1.0]
    }
}

fn draw_title_hud(_world: &mut World, _vw: f32, _vh: f32, _lang: Lang, _compact_resolution: bool) {}

fn draw_character_select_hud(world: &mut World, vw: f32, vh: f32, lang: Lang) {
    let cx = vw / 2.0;
    let cursor_idx = world
        .resource::<CharacterCursor>()
        .map(|c| c.index)
        .unwrap_or(0);
    let meta_snapshot = world.resource::<MetaSave>().cloned().unwrap_or_default();
    let gold = meta_snapshot.gold_total;
    let panel_h = (CharacterKind::ALL.len() as f32 * 38.0 + 132.0).min(vh - 34.0);
    queue_modal_panel(world, cx - 386.0, 12.0, 772.0, panel_h, UI_PANEL_IMAGE_Z);
    queue_slot_frame(
        world,
        cx - 348.0,
        80.0 + cursor_idx as f32 * 38.0,
        696.0,
        50.0,
        UI_ROW_IMAGE_Z,
    );
    if let Some(uq) = world.resource_mut::<UiQueue>() {
        let hy = 88.0 + cursor_idx as f32 * 38.0;
        uq.push(DrawRect::new(cx - 330.0, hy, 660.0, 34.0, [0.25, 0.22, 0.05, 0.75]).with_z(0.2));
    }
    if let Some(q) = world.resource_mut::<TextQueue>() {
        q.push(DrawText::new(
            text(lang, UiText::CharacterSelect).to_string(),
            Vec2::new(cx - 180.0, 26.0),
            40.0,
            [255, 220, 80, 255],
        ));
        q.push(DrawText::new(
            format!("{} {}", text(lang, UiText::Gold), gold),
            Vec2::new(cx + 180.0, 32.0),
            24.0,
            [255, 255, 100, 255],
        ));
        for (i, kind) in CharacterKind::ALL.iter().enumerate() {
            let need = kind.unlock_gold();
            let unlocked = kind.is_unlocked(&meta_snapshot);
            let prefix = if i == cursor_idx { ">" } else { " " };
            let lock_str = if unlocked {
                String::new()
            } else {
                format!("[{} {}]", text(lang, UiText::Locked), need)
            };
            let color = if i == cursor_idx {
                [255, 255, 80, 255]
            } else if unlocked {
                [200, 200, 200, 255]
            } else {
                LOCKED_TEXT_COLOR
            };
            q.push(DrawText::new(
                format!("{} {} {}", prefix, kind.label(lang), lock_str),
                Vec2::new(cx - 318.0, 96.0 + i as f32 * 38.0),
                22.0,
                color,
            ));
        }
        q.push(DrawText::new(
            text(lang, UiText::NavigateSelectBack).to_string(),
            Vec2::new(cx - 290.0, vh * 0.74),
            20.0,
            HELP_TEXT_COLOR,
        ));
    }
}

fn draw_stage_select_hud(world: &mut World, vw: f32, vh: f32, lang: Lang) {
    let cx = vw / 2.0;
    let cursor_idx = world
        .resource::<StageCursor>()
        .map(|c| c.index)
        .unwrap_or(0);
    let unlocked: Vec<String> = world
        .resource::<MetaSave>()
        .map(|m| m.unlocked_stages.clone())
        .unwrap_or_else(|| vec!["MadForest".to_string()]);
    let panel_h = (StageKind::ALL.len() as f32 * 48.0 + 138.0).min(vh - 34.0);
    queue_modal_panel(world, cx - 386.0, 12.0, 772.0, panel_h, UI_PANEL_IMAGE_Z);
    queue_slot_frame(
        world,
        cx - 348.0,
        78.0 + cursor_idx as f32 * 48.0,
        696.0,
        56.0,
        UI_ROW_IMAGE_Z,
    );
    if let Some(uq) = world.resource_mut::<UiQueue>() {
        let hy = 88.0 + cursor_idx as f32 * 48.0;
        uq.push(DrawRect::new(cx - 330.0, hy, 660.0, 40.0, [0.25, 0.22, 0.05, 0.75]).with_z(0.2));
    }
    if let Some(q) = world.resource_mut::<TextQueue>() {
        q.push(DrawText::new(
            text(lang, UiText::StageSelect).to_string(),
            Vec2::new(cx - 150.0, 26.0),
            40.0,
            [255, 220, 80, 255],
        ));
        for (i, stage) in StageKind::ALL.iter().enumerate() {
            let is_unlocked = stage.prerequisite().is_none()
                || stage
                    .prerequisite()
                    .map(|p| unlocked.iter().any(|s| s == p.key()))
                    .unwrap_or(false);
            let prefix = if i == cursor_idx { ">" } else { " " };
            let lock_str = if is_unlocked {
                String::new()
            } else {
                format!("[{}]", text(lang, UiText::Locked))
            };
            let color = if i == cursor_idx {
                [255, 255, 80, 255]
            } else if is_unlocked {
                [200, 200, 200, 255]
            } else {
                LOCKED_TEXT_COLOR
            };
            q.push(DrawText::new(
                format!("{} {} {}", prefix, stage.label(lang), lock_str),
                Vec2::new(cx - 318.0, 96.0 + i as f32 * 48.0),
                26.0,
                color,
            ));
        }
        q.push(DrawText::new(
            text(lang, UiText::NavigateSelectBack).to_string(),
            Vec2::new(cx - 290.0, vh * 0.74),
            20.0,
            HELP_TEXT_COLOR,
        ));
    }
}

fn draw_stage_clear_hud(world: &mut World, vw: f32, vh: f32, lang: Lang) {
    let cx = vw / 2.0;
    let cy = vh / 2.0;
    let elapsed_stat = world
        .resource::<GameStats>()
        .map(|s| s.elapsed)
        .unwrap_or(0.0);
    let kills_stat = world.resource::<GameStats>().map(|s| s.kills).unwrap_or(0);
    let gold_stat = world
        .resource::<GoldWallet>()
        .map(|w| w.current)
        .unwrap_or(0);
    let lv_stat = world
        .query2::<Player, XpAccumulator>()
        .next()
        .map(|(_, _, a)| a.level)
        .unwrap_or(1);
    let mm = (elapsed_stat as u32) / 60;
    let ss = (elapsed_stat as u32) % 60;
    queue_modal_panel(
        world,
        cx - 286.0,
        cy - 200.0,
        572.0,
        400.0,
        UI_PANEL_IMAGE_Z,
    );
    if let Some(uq) = world.resource_mut::<UiQueue>() {
        uq.push(DrawRect::new(0.0, 0.0, vw, vh, [0.0, 0.05, 0.0, 0.5]).with_z(0.0));
    }
    if let Some(q) = world.resource_mut::<TextQueue>() {
        q.push(DrawText::new(
            text(lang, UiText::StageClear).to_string(),
            Vec2::new(cx - 205.0, cy - 150.0),
            56.0,
            [255, 220, 80, 255],
        ));
        q.push(DrawText::new(
            format!(
                "{} {:02}:{:02}  {} {}  {} {}  {} {}",
                text(lang, UiText::Time),
                mm,
                ss,
                text(lang, UiText::Lv),
                lv_stat,
                text(lang, UiText::Kills),
                kills_stat,
                text(lang, UiText::Gold),
                gold_stat
            ),
            Vec2::new(cx - 240.0, cy - 12.0),
            22.0,
            [200, 230, 200, 255],
        ));
        q.push(DrawText::new(
            text(lang, UiText::PressEnterReturn).to_string(),
            Vec2::new(cx - 130.0, cy + 112.0),
            28.0,
            [255, 255, 255, 255],
        ));
    }
}

fn draw_shop_hud(world: &mut World, vw: f32, vh: f32, lang: Lang) {
    let cx = vw / 2.0;
    let meta_clone = world.resource::<MetaSave>().cloned();
    let cursor_idx = world.resource::<ShopCursor>().map(|c| c.index).unwrap_or(0);
    let panel_w = if vw - 16.0 < 620.0 {
        vw - 16.0
    } else {
        (vw - 28.0).clamp(620.0, 980.0)
    };
    let panel_h = if vh - 20.0 < 420.0 {
        vh - 20.0
    } else {
        (vh - 36.0).clamp(420.0, 660.0)
    };
    queue_modal_panel(
        world,
        cx - panel_w / 2.0,
        10.0,
        panel_w,
        panel_h,
        UI_PANEL_IMAGE_Z,
    );
    if let Some(q) = world.resource_mut::<TextQueue>() {
        q.push(DrawText::new(
            text(lang, UiText::PowerupShop).to_string(),
            Vec2::new(cx - 300.0, 26.0),
            40.0,
            [255, 220, 80, 255],
        ));
        if let Some(meta) = &meta_clone {
            q.push(DrawText::new(
                format!("{} {}", text(lang, UiText::Gold), meta.gold_total),
                Vec2::new(cx + 170.0, 32.0),
                24.0,
                [255, 255, 100, 255],
            ));
        }
        let row_step = if vh <= 620.0 { 26.0 } else { 30.0 };
        for (i, kind) in PowerUpKind::ALL.iter().enumerate() {
            let lv = meta_clone
                .as_ref()
                .map(|m| *m.powerup_levels.get(kind.key()).unwrap_or(&0))
                .unwrap_or(0);
            let cost = kind.cost(lv);
            let prefix = if i == cursor_idx { ">" } else { " " };
            let color = if i == cursor_idx {
                [255, 255, 80, 255]
            } else {
                [200, 200, 200, 255]
            };
            q.push(DrawText::new(
                format!(
                    "{} {:<10} Lv {}/{}  {}{}",
                    prefix,
                    kind.label(lang),
                    lv,
                    kind.max_level(),
                    text(lang, UiText::CostPrefix),
                    cost,
                ),
                Vec2::new(cx - 306.0, 80.0 + i as f32 * row_step),
                if vh <= 620.0 { 17.0 } else { 19.0 },
                color,
            ));
        }
        q.push(DrawText::new(
            text(lang, UiText::NavigateBuyBack).to_string(),
            Vec2::new(
                cx - 300.0,
                80.0 + PowerUpKind::ALL.len() as f32 * row_step + 18.0,
            ),
            18.0,
            HELP_TEXT_COLOR,
        ));
    }
}

fn draw_achievements_hud(world: &mut World, vw: f32, vh: f32, lang: Lang) {
    let cx = vw / 2.0;
    let cursor = world
        .resource::<AchievementCursor>()
        .copied()
        .unwrap_or_default();
    let page_count =
        (AchievementKind::ALL.len() + ACHIEVEMENTS_PER_PAGE - 1) / ACHIEVEMENTS_PER_PAGE;
    let page = cursor.page.min(page_count.saturating_sub(1));
    let start = page * ACHIEVEMENTS_PER_PAGE;
    let page_items = AchievementKind::ALL
        .len()
        .saturating_sub(start)
        .min(ACHIEVEMENTS_PER_PAGE);
    let selected = cursor.index.min(page_items.saturating_sub(1));
    let meta_snapshot = world.resource::<MetaSave>().cloned().unwrap_or_default();
    let done = AchievementKind::ALL
        .iter()
        .filter(|&&a| achievement_completed(&meta_snapshot, a))
        .count();
    let best = meta_snapshot.best_time as u32;
    let row_step = if vh <= 620.0 { 34.0 } else { 40.0 };
    let row_y = if vh <= 620.0 { 116.0 } else { 124.0 };
    let panel_w = if vw - 16.0 < 720.0 {
        vw - 16.0
    } else {
        (vw - 36.0).clamp(720.0, 980.0)
    };
    let panel_h = (vh - 30.0).clamp(520.0, 680.0).min(vh - 20.0);
    let panel_x = cx - panel_w / 2.0;

    queue_modal_panel(world, panel_x, 10.0, panel_w, panel_h, UI_PANEL_IMAGE_Z);
    queue_slot_frame(
        world,
        panel_x + 32.0,
        row_y - 10.0 + selected as f32 * row_step,
        panel_w - 64.0,
        row_step + 8.0,
        UI_ROW_IMAGE_Z,
    );
    if let Some(uq) = world.resource_mut::<UiQueue>() {
        uq.push(
            DrawRect::new(
                panel_x + 46.0,
                row_y - 2.0 + selected as f32 * row_step,
                panel_w - 92.0,
                row_step - 8.0,
                [0.25, 0.22, 0.05, 0.75],
            )
            .with_z(0.2),
        );
    }

    if let Some(q) = world.resource_mut::<TextQueue>() {
        q.push(DrawText::new(
            text(lang, UiText::Achievements).to_string(),
            Vec2::new(panel_x + 42.0, 28.0),
            38.0,
            [255, 220, 80, 255],
        ));
        q.push(DrawText::new(
            format!(
                "{} {:02}:{:02}  {} {}  {}/{}  {} {}/{}",
                text(lang, UiText::Best),
                best / 60,
                best % 60,
                text(lang, UiText::Kills),
                meta_snapshot.kills_total,
                done,
                AchievementKind::ALL.len(),
                loc(lang, "페이지", "Page"),
                page + 1,
                page_count
            ),
            Vec2::new(panel_x + 42.0, 76.0),
            if vh <= 620.0 { 17.0 } else { 20.0 },
            [220, 220, 225, 255],
        ));

        for (i, &achievement) in AchievementKind::ALL
            .iter()
            .skip(start)
            .take(ACHIEVEMENTS_PER_PAGE)
            .enumerate()
        {
            let completed = achievement_completed(&meta_snapshot, achievement);
            let prefix = if i == selected { ">" } else { " " };
            let mark = if completed { "[x]" } else { "[ ]" };
            let color = if i == selected {
                [255, 255, 90, 255]
            } else if completed {
                [210, 240, 190, 255]
            } else {
                LOCKED_TEXT_COLOR
            };
            q.push(DrawText::new(
                format!(
                    "{} {} {} - {}",
                    prefix,
                    mark,
                    achievement.title(lang),
                    achievement.reward(lang)
                ),
                Vec2::new(panel_x + 56.0, row_y + i as f32 * row_step),
                if vh <= 620.0 { 15.0 } else { 17.0 },
                color,
            ));
        }

        q.push(DrawText::new(
            loc(
                lang,
                "W/S = 이동  A/D = 페이지  ESC = 뒤로",
                "W/S = navigate  A/D = page  ESC = back",
            )
            .to_string(),
            Vec2::new(panel_x + 42.0, 10.0 + panel_h - 34.0),
            18.0,
            HELP_TEXT_COLOR,
        ));
    }
}

fn draw_pause_menu_hud(world: &mut World, vw: f32, vh: f32, lang: Lang) {
    let cursor_idx = world
        .resource::<PauseMenuCursor>()
        .map(|c| c.index)
        .unwrap_or(0);
    let cx = vw / 2.0;
    let cy = vh / 2.0;
    let panel_w = 420.0_f32;
    let panel_h = 320.0_f32;
    let panel_x = cx - panel_w / 2.0;
    let panel_y = cy - panel_h / 2.0;
    const ITEM_H: f32 = 60.0;
    let item_y0 = panel_y + 82.0;
    let hy = item_y0 + cursor_idx as f32 * ITEM_H - 4.0;
    queue_modal_panel(
        world,
        panel_x - 34.0,
        panel_y - 26.0,
        panel_w + 68.0,
        panel_h + 52.0,
        UI_PANEL_IMAGE_Z,
    );
    queue_slot_frame(
        world,
        panel_x - 4.0,
        hy - 10.0,
        panel_w + 8.0,
        ITEM_H + 12.0,
        UI_ROW_IMAGE_Z,
    );

    // 어두운 전체 오버레이
    if let Some(uq) = world.resource_mut::<UiQueue>() {
        uq.push(DrawRect::new(0.0, 0.0, vw, vh, [0.0, 0.0, 0.0, 0.62]).with_z(0.0));
        // 선택 항목 하이라이트
        uq.push(
            DrawRect::new(
                panel_x + 10.0,
                hy,
                panel_w - 20.0,
                ITEM_H - 6.0,
                [0.28, 0.22, 0.06, 0.85],
            )
            .with_z(0.2),
        );
    }

    let labels = [
        (text(lang, UiText::Resume), text(lang, UiText::Resume)),
        (
            text(lang, UiText::ReturnToTitle),
            text(lang, UiText::ReturnToTitle),
        ),
        (text(lang, UiText::QuitGame), text(lang, UiText::QuitGame)),
    ];

    if let Some(q) = world.resource_mut::<TextQueue>() {
        q.push(DrawText::new(
            text(lang, UiText::Paused).to_string(),
            Vec2::new(panel_x + panel_w / 2.0 - 64.0, panel_y + 16.0),
            42.0,
            [255, 220, 80, 255],
        ));
        for (i, (label, _)) in labels.iter().enumerate().take(PAUSE_MENU_ITEMS) {
            let color = if i == cursor_idx {
                [255, 255, 80, 255]
            } else {
                [200, 200, 200, 255]
            };
            q.push(DrawText::new(
                label.to_string(),
                Vec2::new(panel_x + 20.0, item_y0 + i as f32 * ITEM_H),
                28.0,
                color,
            ));
        }
        q.push(DrawText::new(
            text(lang, UiText::PauseHelp).to_string(),
            Vec2::new(panel_x + 10.0, panel_y + panel_h - 36.0),
            18.0,
            HELP_TEXT_COLOR,
        ));
    }
}

fn draw_settings_hud(world: &mut World, vw: f32, vh: f32, lang: Lang) {
    let cx = vw / 2.0;
    let cy = vh / 2.0;
    let panel_w = 720.0_f32.min(vw - 40.0);
    let panel_h = 420.0_f32.min(vh - 40.0);
    let px = cx - panel_w / 2.0;
    let py = cy - panel_h / 2.0;
    let cursor_idx = world
        .resource::<SettingsCursor>()
        .map(|c| c.index)
        .unwrap_or(0);
    let meta = world.resource::<MetaSave>().cloned().unwrap_or_default();
    let selected_resolution = ResolutionPreset::from_key(&meta.resolution_key);
    let row_step = if panel_h <= 380.0 { 42.0 } else { 50.0 };
    let hy = py + 82.0 + cursor_idx as f32 * row_step - 4.0;
    queue_modal_panel(
        world,
        px - 36.0,
        py - 28.0,
        panel_w + 72.0,
        panel_h + 56.0,
        UI_PANEL_IMAGE_Z,
    );
    queue_slot_frame(
        world,
        px - 2.0,
        hy - 8.0,
        panel_w + 4.0,
        row_step + 10.0,
        UI_ROW_IMAGE_Z,
    );

    if let Some(uq) = world.resource_mut::<UiQueue>() {
        uq.push(DrawRect::new(0.0, 0.0, vw, vh, [0.0, 0.0, 0.0, 0.5]).with_z(0.0));
        // 선택 항목 하이라이트
        uq.push(
            DrawRect::new(
                px + 10.0,
                hy,
                panel_w - 20.0,
                row_step - 4.0,
                [0.28, 0.22, 0.06, 0.85],
            )
            .with_z(0.2),
        );
    }
    if let Some(q) = world.resource_mut::<TextQueue>() {
        q.push(DrawText::new(
            text(lang, UiText::Settings).to_string(),
            Vec2::new(cx - 55.0, py + 14.0),
            38.0,
            [255, 220, 80, 255],
        ));
        let rows = [
            (
                text(lang, UiText::Language).to_string(),
                meta.language_setting.label(lang).to_string(),
            ),
            (
                text(lang, UiText::HudDetail).to_string(),
                meta.hud_detail.label(lang).to_string(),
            ),
            (
                text(lang, UiText::BgmVolume).to_string(),
                format!("{:.0}%", meta.bgm_volume * 100.0),
            ),
            (
                text(lang, UiText::SfxVolume).to_string(),
                format!("{:.0}%", meta.sfx_volume * 100.0),
            ),
            (
                text(lang, UiText::Resolution).to_string(),
                selected_resolution.label(lang).to_string(),
            ),
        ];
        let row_step = if panel_h <= 380.0 { 42.0 } else { 50.0 };
        for (i, (label, value)) in rows.iter().enumerate().take(SETTINGS_ITEMS) {
            let is_sel = i == cursor_idx;
            let color = if is_sel {
                [255, 255, 80, 255]
            } else {
                [200, 200, 200, 255]
            };
            q.push(DrawText::new(
                format!("{:<14}  < {} >", label, value),
                Vec2::new(px + 24.0, py + 86.0 + i as f32 * row_step),
                if panel_h <= 380.0 { 20.0 } else { 24.0 },
                color,
            ));
        }
        q.push(DrawText::new(
            text(lang, UiText::NavigateChangeApplyBack).to_string(),
            Vec2::new(px + 36.0, py + panel_h + 40.0),
            16.0,
            HELP_TEXT_COLOR,
        ));
    }
}

pub struct HudSystem;

impl System for HudSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        // 실제 뷰포트 크기 (해상도 변경 즉시 반영)
        let vw = world
            .resource::<ViewportSize>()
            .map(|v| v.width)
            .unwrap_or(1280.0);
        let vh = world
            .resource::<ViewportSize>()
            .map(|v| v.height)
            .unwrap_or(720.0);

        let mode = world
            .resource::<SurvivorMode>()
            .copied()
            .unwrap_or(SurvivorMode::InGame);
        let lang = world
            .resource::<MetaSave>()
            .map(|m| m.effective_lang())
            .unwrap_or(Lang::Ko);
        let hud_detail = world
            .resource::<MetaSave>()
            .map(|m| m.hud_detail)
            .unwrap_or_default();
        let compact_resolution = vw <= 900.0 || vh <= 640.0;
        let ui_scale = responsive_ui_scale(vw, vh);

        match mode {
            SurvivorMode::Title => {
                draw_title_hud(world, vw, vh, lang, compact_resolution);
                return;
            }
            SurvivorMode::CharacterSelect => {
                draw_character_select_hud(world, vw, vh, lang);
                return;
            }
            SurvivorMode::StageSelect => {
                draw_stage_select_hud(world, vw, vh, lang);
                return;
            }
            SurvivorMode::StageClear => {
                draw_stage_clear_hud(world, vw, vh, lang);
                return;
            }
            SurvivorMode::Shop => {
                draw_shop_hud(world, vw, vh, lang);
                return;
            }
            SurvivorMode::Achievements => {
                draw_achievements_hud(world, vw, vh, lang);
                return;
            }
            SurvivorMode::PauseMenu => {
                draw_pause_menu_hud(world, vw, vh, lang);
                return;
            }
            SurvivorMode::Settings => {
                draw_settings_hud(world, vw, vh, lang);
                return;
            }
            SurvivorMode::InGame => {
                // 아래 인게임 HUD 코드 실행
            }
        }

        // ─── InGame HUD ───────────────────────────────────────────────────────

        // 1) Playing 중에만 timer 누적
        let state = world
            .resource::<GameState>()
            .cloned()
            .unwrap_or(GameState::Playing);
        if matches!(state, GameState::Playing) {
            if let Some(stats) = world.resource_mut::<GameStats>() {
                stats.elapsed += dt;
            }
        }

        // 2) Player 상태 캐시 (borrow 즉시 종료 — 이후 resource_mut 와 충돌 방지)
        let player_info = world
            .query2::<Player, Health>()
            .next()
            .map(|(_, _, h)| (h.current, h.max));
        let xp_info = world
            .query2::<Player, XpAccumulator>()
            .next()
            .map(|(_, _, acc)| (acc.current, acc.level, acc.next_threshold));
        let passive_count = world
            .query2::<Player, PassiveInventory>()
            .next()
            .map(|(_, _, inv)| inv.passives.len())
            .unwrap_or(0);

        let elapsed = world
            .resource::<GameStats>()
            .map(|s| s.elapsed)
            .unwrap_or(0.0);
        let kills = world.resource::<GameStats>().map(|s| s.kills).unwrap_or(0);
        let gold = world
            .resource::<GoldWallet>()
            .map(|w| w.current)
            .unwrap_or(0);
        let player_stats = world
            .query2::<Player, PlayerStats>()
            .next()
            .map(|(_, _, stats)| stats.clone());
        let mm = (elapsed as u32) / 60;
        let ss = (elapsed as u32) % 60;

        // 3) 좌상단 HP 바 + 정보 텍스트 + 하단 XP 바
        if let (Some((hp, hp_max)), Some((xp, lv, xp_max))) = (player_info, xp_info) {
            let hud_x = HUD_X * ui_scale;
            let hud_y = HUD_Y * ui_scale;
            let hp_bar_w = HP_BAR_W * ui_scale;
            let hp_bar_h = HP_BAR_H * ui_scale;
            let xp_bar_h = XP_BAR_H * ui_scale;
            let hp_ratio = (hp / hp_max.max(1.0)).clamp(0.0, 1.0);
            let xp_ratio = if xp_max > 0 {
                (xp as f32 / xp_max as f32).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let compact_hud = vw <= 900.0 || compact_resolution;
            let compact_detailed_hud = compact_hud && matches!(hud_detail, HudDetail::Detailed);
            let panel_h = if compact_detailed_hud { 120.0 } else { 64.0 };
            let panel_w = (if compact_detailed_hud {
                390.0_f32
            } else if compact_hud {
                560.0_f32
            } else {
                720.0_f32
            } * ui_scale)
                .min(vw - 16.0 * ui_scale);
            queue_slot_frame(
                world,
                hud_x - 20.0 * ui_scale,
                hud_y - 20.0 * ui_scale,
                panel_w + 28.0 * ui_scale,
                panel_h * ui_scale + 26.0 * ui_scale,
                UI_PANEL_IMAGE_Z,
            );
            queue_ui_colored_image(
                world,
                hud_x - ui_scale,
                hud_y - ui_scale,
                hp_bar_w + 2.0 * ui_scale,
                hp_bar_h + 2.0 * ui_scale,
                [0.04, 0.0, 0.0, 0.96],
                UI_ROW_IMAGE_Z,
            );
            if hp_ratio > 0.0 {
                queue_ui_colored_image(
                    world,
                    hud_x,
                    hud_y,
                    hp_bar_w * hp_ratio,
                    hp_bar_h,
                    hp_color(hp_ratio),
                    UI_ROW_IMAGE_Z + 1.0,
                );
            }
            let xp_y = vh - xp_bar_h;
            queue_ui_colored_image(
                world,
                0.0,
                xp_y,
                vw,
                xp_bar_h,
                [0.06, 0.07, 0.13, 0.92],
                UI_ROW_IMAGE_Z,
            );
            if xp_ratio > 0.0 {
                queue_ui_colored_image(
                    world,
                    0.0,
                    xp_y,
                    vw * xp_ratio,
                    xp_bar_h,
                    [0.25, 0.68, 1.0, 1.0],
                    UI_ROW_IMAGE_Z + 1.0,
                );
            }

            // HP 수치 텍스트 (바 우측)
            let hp_text = format!("{:.0}/{:.0}", hp.max(0.0), hp_max);
            let xp_text = format!("{}/{}", xp, xp_max);
            let mut stat_lines: Vec<String> = Vec::new();
            let info_size = if compact_detailed_hud {
                21.0
            } else if compact_hud {
                20.0
            } else {
                19.0
            };
            let stat_size = if compact_detailed_hud {
                19.0
            } else if compact_hud {
                18.0
            } else {
                17.0
            };
            let time_text = format!("{:02}:{:02}", mm, ss);
            let mut primary_values = Vec::new();
            let mut secondary_values = Vec::new();
            match hud_detail {
                HudDetail::Minimal => {
                    primary_values.push(HudLabelValue {
                        label: IngameLabelKind::Lv,
                        value: lv.to_string(),
                    });
                    primary_values.push(HudLabelValue {
                        label: IngameLabelKind::Hp,
                        value: hp_text.clone(),
                    });
                    primary_values.push(HudLabelValue {
                        label: IngameLabelKind::Xp,
                        value: xp_text.clone(),
                    });
                    primary_values.push(HudLabelValue {
                        label: IngameLabelKind::Kills,
                        value: kills.to_string(),
                    });
                }
                HudDetail::Detailed => {
                    if let Some(stats) = player_stats {
                        if compact_hud {
                            let (line_a, line_b) = compact_stats_lines(lang, &stats);
                            stat_lines.push(line_a);
                            stat_lines.push(line_b);
                        } else {
                            stat_lines.push(compact_stats_line(lang, &stats));
                        }
                    }
                    primary_values.push(HudLabelValue {
                        label: IngameLabelKind::Lv,
                        value: lv.to_string(),
                    });
                    primary_values.push(HudLabelValue {
                        label: IngameLabelKind::Hp,
                        value: hp_text.clone(),
                    });
                    if compact_hud {
                        secondary_values.push(HudLabelValue {
                            label: IngameLabelKind::Xp,
                            value: xp_text.clone(),
                        });
                        secondary_values.push(HudLabelValue {
                            label: IngameLabelKind::Kills,
                            value: kills.to_string(),
                        });
                    } else {
                        primary_values.push(HudLabelValue {
                            label: IngameLabelKind::Xp,
                            value: xp_text.clone(),
                        });
                        primary_values.push(HudLabelValue {
                            label: IngameLabelKind::Gold,
                            value: gold.to_string(),
                        });
                        primary_values.push(HudLabelValue {
                            label: IngameLabelKind::Kills,
                            value: kills.to_string(),
                        });
                    }
                }
                HudDetail::Normal => {
                    primary_values.push(HudLabelValue {
                        label: IngameLabelKind::Lv,
                        value: lv.to_string(),
                    });
                    primary_values.push(HudLabelValue {
                        label: IngameLabelKind::Hp,
                        value: hp_text.clone(),
                    });
                    primary_values.push(HudLabelValue {
                        label: IngameLabelKind::Xp,
                        value: xp_text.clone(),
                    });
                    if !compact_hud {
                        primary_values.push(HudLabelValue {
                            label: IngameLabelKind::Gold,
                            value: gold.to_string(),
                        });
                        primary_values.push(HudLabelValue {
                            label: IngameLabelKind::Passives,
                            value: passive_count.to_string(),
                        });
                    }
                    primary_values.push(HudLabelValue {
                        label: IngameLabelKind::Kills,
                        value: kills.to_string(),
                    });
                }
            };
            let primary_y = hud_y + 30.0 * ui_scale;
            let secondary_y = hud_y + 54.0 * ui_scale;
            let info_font_size = info_size * ui_scale;
            let time_width = estimated_text_width(&time_text, info_font_size) + 14.0 * ui_scale;
            let primary_layouts = hud_label_layouts(
                hud_x + time_width,
                lang,
                ui_scale,
                info_font_size,
                &primary_values,
            );
            let secondary_layouts =
                hud_label_layouts(hud_x, lang, ui_scale, info_font_size, &secondary_values);
            let label_h = HUD_LABEL_BASE_H * ui_scale;
            for (item, layout) in primary_values.iter().zip(primary_layouts.iter()) {
                queue_ingame_ui_image(
                    world,
                    (
                        layout.label_x,
                        primary_y + 2.0 * ui_scale,
                        ingame_label_width(item.label, lang, ui_scale),
                        label_h,
                    ),
                    ingame_label_path(item.label, lang),
                    UI_ROW_IMAGE_Z + 3.0,
                );
            }
            for (item, layout) in secondary_values.iter().zip(secondary_layouts.iter()) {
                queue_ingame_ui_image(
                    world,
                    (
                        layout.label_x,
                        secondary_y + 2.0 * ui_scale,
                        ingame_label_width(item.label, lang, ui_scale),
                        label_h,
                    ),
                    ingame_label_path(item.label, lang),
                    UI_ROW_IMAGE_Z + 3.0,
                );
            }
            if let Some(q) = world.resource_mut::<TextQueue>() {
                q.push(DrawText::new(
                    hp_text,
                    Vec2::new(hud_x + hp_bar_w + 8.0 * ui_scale, hud_y - ui_scale),
                    if compact_detailed_hud {
                        22.0
                    } else if compact_hud {
                        20.0
                    } else {
                        19.0
                    } * ui_scale,
                    [255, 255, 255, 255],
                ));
                q.push(DrawText::new(
                    time_text,
                    Vec2::new(hud_x, primary_y),
                    info_font_size,
                    [210, 210, 210, 255],
                ));
                for (item, layout) in primary_values.iter().zip(primary_layouts.iter()) {
                    q.push(DrawText::new(
                        item.value.clone(),
                        Vec2::new(layout.value_x, primary_y),
                        info_font_size,
                        [210, 210, 210, 255],
                    ));
                }
                if !secondary_values.is_empty() {
                    for (item, layout) in secondary_values.iter().zip(secondary_layouts.iter()) {
                        q.push(DrawText::new(
                            item.value.clone(),
                            Vec2::new(layout.value_x, secondary_y),
                            info_font_size,
                            [210, 210, 210, 255],
                        ));
                    }
                }
                for (i, stat_line) in stat_lines.into_iter().enumerate() {
                    let y = if compact_hud {
                        if compact_detailed_hud {
                            hud_y + 82.0 * ui_scale + i as f32 * 22.0 * ui_scale
                        } else {
                            hud_y + 68.0 * ui_scale + i as f32 * 22.0 * ui_scale
                        }
                    } else {
                        hud_y + 56.0 * ui_scale + i as f32 * 20.0 * ui_scale
                    };
                    q.push(DrawText::new(
                        stat_line,
                        Vec2::new(hud_x, y),
                        stat_size * ui_scale,
                        [205, 220, 235, 250],
                    ));
                }
            }

            if world
                .resource::<DebugOverlay>()
                .map(|overlay| overlay.visible)
                .unwrap_or(false)
            {
                let panel_w = (260.0 * ui_scale).min((vw - 24.0 * ui_scale).max(220.0));
                let panel_h = 112.0 * ui_scale;
                let panel_x = (vw - panel_w - 14.0 * ui_scale).max(12.0 * ui_scale);
                let panel_y = 14.0 * ui_scale;
                if let Some(uq) = world.resource_mut::<UiQueue>() {
                    uq.push(
                        DrawRect::new(
                            panel_x,
                            panel_y,
                            panel_w,
                            panel_h,
                            [0.02, 0.025, 0.035, 0.86],
                        )
                        .with_z(0.62),
                    );
                    uq.push(
                        DrawRect::new(
                            panel_x,
                            panel_y,
                            4.0 * ui_scale,
                            panel_h,
                            [0.45, 0.82, 1.0, 0.95],
                        )
                        .with_z(0.63),
                    );
                }
                if let Some(q) = world.resource_mut::<TextQueue>() {
                    let title = loc(lang, "테스트 모드", "Test Mode");
                    let hide = loc(lang, "H 숨김/표시", "H hide/show");
                    let bomb = loc(lang, "F5 폭탄", "F5 Bomb");
                    let rosary = loc(lang, "F6 로자리", "F6 Rosary");
                    let boss = loc(lang, "B 보스 소환", "B Spawn boss");
                    let lines = [title, hide, bomb, rosary, boss];
                    for (i, line) in lines.into_iter().enumerate() {
                        q.push(DrawText::new(
                            line.to_string(),
                            Vec2::new(
                                panel_x + 14.0 * ui_scale,
                                panel_y + (12.0 + i as f32 * 19.0) * ui_scale,
                            ),
                            if i == 0 { 17.0 } else { 15.0 } * ui_scale,
                            if i == 0 {
                                [205, 235, 255, 255]
                            } else {
                                [220, 220, 225, 245]
                            },
                        ));
                    }
                }
            }

            // 4) 무기/패시브 슬롯 (하단 좌측, XP 바 위). 800x600에서도 고정 6칸 x 2행 유지.
            let weapon_slots: Vec<(WeaponKind, u8, bool)> = world
                .query2::<Player, WeaponInventory>()
                .next()
                .map(|(_, _, inv)| {
                    inv.slots
                        .iter()
                        .map(|s| (s.kind.clone(), s.level, s.evolved))
                        .collect()
                })
                .unwrap_or_default();
            let passive_slots: Vec<(PassiveKind, u8)> = world
                .query2::<Player, PassiveInventory>()
                .next()
                .map(|(_, _, inv)| inv.passives.iter().map(|s| (s.kind, s.level)).collect())
                .unwrap_or_default();

            let slot_x0 = 16.0 * ui_scale;
            let slot_gap = SLOT_GAP * ui_scale;
            let slot_h = SLOT_H * ui_scale;
            let max_panel_w = (vw - slot_x0 * 2.0).max(300.0);
            let slot_w = (SLOT_W * ui_scale)
                .min((max_panel_w - (SLOT_COLS - 1) as f32 * slot_gap) / SLOT_COLS as f32);
            let weapon_y = vh - XP_BAR_H * ui_scale - slot_h * 2.0 - slot_gap - 16.0 * ui_scale;
            let passive_y = weapon_y + slot_h + slot_gap;

            queue_ingame_ui_image(
                world,
                (
                    slot_x0,
                    weapon_y - 32.0 * ui_scale,
                    124.0 * ui_scale,
                    31.0 * ui_scale,
                ),
                section_label_path(IngameSectionKind::Weapons, lang),
                UI_ROW_IMAGE_Z + 3.0,
            );
            queue_ingame_ui_image(
                world,
                (
                    slot_x0 + 132.0 * ui_scale,
                    weapon_y - 32.0 * ui_scale,
                    144.0 * ui_scale,
                    31.0 * ui_scale,
                ),
                section_label_path(IngameSectionKind::Passives, lang),
                UI_ROW_IMAGE_Z + 3.0,
            );

            if let Some(q) = world.resource_mut::<TextQueue>() {
                for i in 0..SLOT_COLS {
                    let sx = slot_x0 + i as f32 * (slot_w + slot_gap);
                    if let Some((_, level, evolved)) = weapon_slots.get(i) {
                        q.push(DrawText::new(
                            if *evolved {
                                format!("Lv {} E", level)
                            } else {
                                format!("Lv {}", level)
                            },
                            Vec2::new(sx + 46.0 * ui_scale, weapon_y + 8.0 * ui_scale),
                            if slot_w < 128.0 * ui_scale {
                                15.0
                            } else {
                                16.0
                            } * ui_scale,
                            [242, 242, 248, 245],
                        ));
                    }
                }

                for i in 0..SLOT_COLS {
                    let sx = slot_x0 + i as f32 * (slot_w + slot_gap);
                    if let Some((_, level)) = passive_slots.get(i) {
                        q.push(DrawText::new(
                            format!("Lv {}", level),
                            Vec2::new(sx + 46.0 * ui_scale, passive_y + 8.0 * ui_scale),
                            if slot_w < 128.0 * ui_scale {
                                15.0
                            } else {
                                16.0
                            } * ui_scale,
                            [242, 242, 248, 245],
                        ));
                    }
                }
            }

            // 5) Paused + PendingLevelUp: 반투명 오버레이 + 카드 패널
            if matches!(state, GameState::Paused) {
                if let Some(p) = world.resource::<PendingLevelUp>() {
                    let offered = p.offered;
                    let cx = vw / 2.0;
                    let cy = vh / 2.0;
                    queue_ingame_ui_image(
                        world,
                        (cx - 190.0, cy - 168.0, 380.0, 95.0),
                        levelup_title_path(lang),
                        UI_PANEL_IMAGE_Z + 8.0,
                    );
                    if let Some(q) = world.resource_mut::<TextQueue>() {
                        q.push(DrawText::new(
                            format!("1.  {}", offered[0].label(lang)),
                            Vec2::new(cx - 178.0, cy - 48.0),
                            24.0,
                            [255, 255, 255, 255],
                        ));
                        q.push(DrawText::new(
                            format!("2.  {}", offered[1].label(lang)),
                            Vec2::new(cx - 178.0, cy + 8.0),
                            24.0,
                            [255, 255, 255, 255],
                        ));
                        q.push(DrawText::new(
                            format!("3.  {}", offered[2].label(lang)),
                            Vec2::new(cx - 178.0, cy + 64.0),
                            24.0,
                            [255, 255, 255, 255],
                        ));
                    }
                }
            }
        }

        // 6) GameOver: 결과 패널 + 통계
        if matches!(state, GameState::GameOver) {
            let lv_stat = xp_info.map(|(_, lv, _)| lv).unwrap_or(1);
            let mm2 = mm;
            let ss2 = ss;
            let cx = vw / 2.0;
            let cy = vh / 2.0;
            queue_modal_panel(
                world,
                cx - 326.0,
                cy - 222.0,
                652.0,
                444.0,
                UI_PANEL_IMAGE_Z,
            );
            queue_ingame_ui_image(
                world,
                (cx - 205.0, cy - 194.0, 410.0, 102.0),
                gameover_title_path(lang),
                UI_PANEL_IMAGE_Z + 8.0,
            );
            queue_ingame_ui_image(
                world,
                (cx - 186.0, cy + 124.0, 372.0, 76.0),
                restart_hint_path(lang),
                UI_PANEL_IMAGE_Z + 8.0,
            );
            if let Some(uq) = world.resource_mut::<UiQueue>() {
                uq.push(DrawRect::new(0.0, 0.0, vw, vh, [0.0, 0.0, 0.0, 0.62]).with_z(0.0));
            }
            if let Some(q) = world.resource_mut::<TextQueue>() {
                q.push(DrawText::new(
                    format!(
                        "{} {:02}:{:02}  {} {}  {} {}  {} {}",
                        text(lang, UiText::Time),
                        mm2,
                        ss2,
                        text(lang, UiText::Lv),
                        lv_stat,
                        text(lang, UiText::Kills),
                        kills,
                        text(lang, UiText::Gold),
                        gold
                    ),
                    Vec2::new(cx - 270.0, cy - 20.0),
                    22.0,
                    [220, 180, 180, 255],
                ));
            }
        }

        // 7) 보스 HP 바 — 보스가 존재하면 화면 상단 중앙에 시각적 바로 표시
        let boss_info: Option<(super::boss::BossKind, f32, f32, u8)> = world
            .query2::<Boss, Health>()
            .next()
            .map(|(_, b, h)| (b.kind, h.current, h.max, b.phase));
        if let Some((kind, hp, max, phase)) = boss_info {
            let hp_ratio = (hp / max.max(1.0)).clamp(0.0, 1.0);
            let phase_color: [f32; 4] = match phase {
                2 => [1.0, 0.3, 0.0, 1.0],
                1 => [0.9, 0.1, 0.1, 1.0],
                _ => [0.7, 0.1, 0.1, 1.0],
            };
            let boss_bar_w = (BOSS_BAR_W * ui_scale).min(vw - 80.0 * ui_scale);
            let boss_bar_h = BOSS_BAR_H * ui_scale;
            let boss_bar_x = (vw - boss_bar_w) / 2.0;
            let boss_bar_y = boss_bar_y(compact_resolution, hud_detail, ui_scale, vw);
            queue_slot_frame(
                world,
                boss_bar_x - 26.0 * ui_scale,
                boss_bar_y - 12.0 * ui_scale,
                boss_bar_w + 52.0 * ui_scale,
                boss_bar_h + 24.0 * ui_scale,
                UI_PANEL_IMAGE_Z,
            );
            queue_ui_colored_image(
                world,
                boss_bar_x - 2.0,
                boss_bar_y - 2.0,
                boss_bar_w + 4.0,
                boss_bar_h + 4.0,
                [0.0, 0.0, 0.0, 0.9],
                UI_ROW_IMAGE_Z,
            );
            if hp_ratio > 0.0 {
                queue_ui_colored_image(
                    world,
                    boss_bar_x,
                    boss_bar_y,
                    boss_bar_w * hp_ratio,
                    boss_bar_h,
                    phase_color,
                    UI_ROW_IMAGE_Z + 1.0,
                );
            }
            if let Some(q) = world.resource_mut::<TextQueue>() {
                q.push(DrawText::new(
                    format!("{}  {:.0}/{:.0}", kind.label(lang), hp.max(0.0), max),
                    Vec2::new(boss_bar_x, boss_bar_y - BOSS_LABEL_HEIGHT * ui_scale),
                    18.0 * ui_scale,
                    [255, 120, 120, 255],
                ));
            }
        }

        // 8) 데미지 숫자(floating combat text) — 월드→화면 좌표 변환 후 TextQueue 에 push
        let camera = world.resource::<Camera>().copied().unwrap_or_default();
        let damage_items: Vec<(Vec2, f32, u8)> = world
            .query2::<DamageNumber, Transform>()
            .filter_map(|(_, dn, t)| {
                let screen = (t.position - camera.position) * camera.zoom;
                if screen.x < -40.0
                    || screen.x > vw + 40.0
                    || screen.y < -40.0
                    || screen.y > vh + 40.0
                {
                    return None;
                }
                let alpha = ((1.0 - dn.fade()) * 255.0).clamp(0.0, 255.0) as u8;
                Some((screen, dn.value, alpha))
            })
            .collect();

        if !damage_items.is_empty() {
            if let Some(q) = world.resource_mut::<TextQueue>() {
                for (pos, value, alpha) in damage_items {
                    q.push(DrawText::new(
                        format!("{:.0}", value),
                        pos,
                        20.0,
                        [255, 230, 120, alpha],
                    ));
                }
            }
        }
    }
}

fn top_hud_panel_height(compact_resolution: bool, hud_detail: HudDetail, viewport_w: f32) -> f32 {
    let compact_detailed_hud =
        (viewport_w <= 900.0 || compact_resolution) && matches!(hud_detail, HudDetail::Detailed);
    if compact_detailed_hud {
        120.0
    } else {
        64.0
    }
}

fn top_hud_panel_bottom(
    compact_resolution: bool,
    hud_detail: HudDetail,
    ui_scale: f32,
    viewport_w: f32,
) -> f32 {
    (HUD_Y - 8.0 + top_hud_panel_height(compact_resolution, hud_detail, viewport_w)) * ui_scale
}

fn boss_bar_y(
    compact_resolution: bool,
    hud_detail: HudDetail,
    ui_scale: f32,
    viewport_w: f32,
) -> f32 {
    top_hud_panel_bottom(compact_resolution, hud_detail, ui_scale, viewport_w)
        + (BOSS_LABEL_PADDING + BOSS_LABEL_HEIGHT) * ui_scale
}
