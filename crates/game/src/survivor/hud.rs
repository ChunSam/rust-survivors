use super::achievement::{achievement_completed, AchievementKind};
use super::boss::Boss;
use super::character::{CharacterCursor, CharacterKind};
use super::damage_number::DamageNumber;
use super::health::Health;
use super::inventory::{WeaponInventory, WeaponKind};
use super::levelup::PendingLevelUp;
use super::locale::{loc, text, Lang, UiText};
use super::meta::{
    HudDetail, MetaSave, PauseMenuCursor, ResolutionPreset, SettingsCursor, SurvivorMode,
    PAUSE_MENU_ITEMS, SETTINGS_ITEMS,
};
use super::passive::{PassiveInventory, PassiveKind};
use super::pickup::GoldWallet;
use super::player::{Player, PlayerStats};
use super::powerup::{PowerUpKind, ShopCursor};
use super::stage::{StageCursor, StageKind};
use super::xp::XpAccumulator;
use engine::components::GameState;
use engine::renderer::text::{DrawText, TextQueue};
use engine::{Camera, DrawRect, System, Transform, UiQueue, ViewportSize, World};
use glam::Vec2;

// ── 바 크기 상수 ─────────────────────────────────────────────────────────────
const HP_BAR_W: f32 = 180.0;
const HP_BAR_H: f32 = 14.0;
const XP_BAR_H: f32 = 6.0;
const BOSS_BAR_W: f32 = 360.0;
const BOSS_BAR_H: f32 = 16.0;
const SLOT_COLS: usize = 6;
const SLOT_W: f32 = 112.0;
const SLOT_H: f32 = 24.0;
const SLOT_GAP: f32 = 4.0;

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

/// 무기 종류별 고유 색상
fn weapon_kind_color(kind: &WeaponKind) -> [f32; 4] {
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

fn weapon_kind_name(kind: &WeaponKind, lang: Lang) -> &'static str {
    match kind {
        WeaponKind::Whip { .. } => loc(lang, "채찍", "Whip"),
        WeaponKind::MagicWand { .. } => loc(lang, "마법봉", "Wand"),
        WeaponKind::Knife { .. } => loc(lang, "칼", "Knife"),
        WeaponKind::Axe { .. } => loc(lang, "도끼", "Axe"),
        WeaponKind::Cross { .. } => loc(lang, "십자가", "Cross"),
        WeaponKind::FireWand { .. } => loc(lang, "화염봉", "Fire"),
        WeaponKind::Garlic { .. } => loc(lang, "마늘", "Garlic"),
        WeaponKind::HolyWater { .. } => loc(lang, "성수", "Water"),
        WeaponKind::KingBible { .. } => loc(lang, "성서", "Bible"),
        WeaponKind::LightningRing { .. } => loc(lang, "번개반지", "Ring"),
    }
}

/// 패시브 종류별 고유 색상
fn passive_kind_color(kind: PassiveKind) -> [f32; 4] {
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
        PassiveKind::Crown => [1.0, 0.9, 0.1, 1.0],
        PassiveKind::StoneMask => [0.5, 0.5, 0.5, 1.0],
        PassiveKind::SkullOManiac => [0.4, 0.2, 0.4, 1.0],
        PassiveKind::Tiragisu => [1.0, 0.5, 0.5, 1.0],
    }
}

fn passive_kind_name(kind: PassiveKind, lang: Lang) -> &'static str {
    match kind {
        PassiveKind::Spinach => loc(lang, "시금치", "Spinach"),
        PassiveKind::Armor => loc(lang, "방어구", "Armor"),
        PassiveKind::HollowHeart => loc(lang, "빈심장", "Heart"),
        PassiveKind::Pummarola => loc(lang, "포마롤라", "Regen"),
        PassiveKind::EmptyTome => loc(lang, "빈서적", "Tome"),
        PassiveKind::Candelabrador => loc(lang, "촛대", "Candle"),
        PassiveKind::Bracer => loc(lang, "팔찌", "Bracer"),
        PassiveKind::Spellbinder => loc(lang, "결속자", "Binder"),
        PassiveKind::Duplicator => loc(lang, "복제기", "Dupe"),
        PassiveKind::Wings => loc(lang, "날개", "Wings"),
        PassiveKind::Attractorb => loc(lang, "인력구", "Magnet"),
        PassiveKind::Clover => loc(lang, "클로버", "Clover"),
        PassiveKind::Crown => loc(lang, "왕관", "Crown"),
        PassiveKind::StoneMask => loc(lang, "석가면", "Mask"),
        PassiveKind::SkullOManiac => loc(lang, "해골광", "Skull"),
        PassiveKind::Tiragisu => loc(lang, "티라지수", "Revive"),
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

        match mode {
            SurvivorMode::Title => {
                let cx = vw / 2.0;
                let cy = vh / 2.0;
                if let Some(uq) = world.resource_mut::<UiQueue>() {
                    uq.push(DrawRect::new(0.0, 0.0, vw, vh, [0.0, 0.0, 0.05, 0.35]).with_z(0.0));
                    uq.push(
                        DrawRect::new(
                            cx - 340.0,
                            cy - 160.0,
                            680.0,
                            250.0,
                            [0.02, 0.015, 0.03, 0.70],
                        )
                        .with_z(0.08),
                    );
                    uq.push(
                        DrawRect::new(
                            cx - 340.0,
                            cy + 118.0,
                            680.0,
                            30.0,
                            [0.02, 0.015, 0.03, 0.64],
                        )
                        .with_z(0.08),
                    );
                }
                if let Some(q) = world.resource_mut::<TextQueue>() {
                    q.push(DrawText {
                        text: text(lang, UiText::GameTitle).to_string(),
                        position: Vec2::new(cx - 227.0, cy - 127.0),
                        size: 56.0,
                        color: [20, 12, 8, 230],
                    });
                    q.push(DrawText {
                        text: text(lang, UiText::GameTitle).to_string(),
                        position: Vec2::new(cx - 230.0, cy - 130.0),
                        size: 56.0,
                        color: [255, 220, 80, 255],
                    });
                    q.push(DrawText {
                        text: text(lang, UiText::PressEnterStart).to_string(),
                        position: Vec2::new(cx - 130.0, cy - 20.0),
                        size: 22.0,
                        color: [255, 255, 255, 255],
                    });
                    q.push(DrawText {
                        text: text(lang, UiText::TitleMenuHelp).to_string(),
                        position: Vec2::new(cx - 240.0, cy + 40.0),
                        size: 18.0,
                        color: [200, 200, 255, 255],
                    });
                }
                let meta_info = world
                    .resource::<MetaSave>()
                    .map(|m| (m.gold_total, m.best_time, m.kills_total));
                if let Some((gold, best, kills)) = meta_info {
                    if let Some(q) = world.resource_mut::<TextQueue>() {
                        q.push(DrawText {
                            text: if lang == Lang::Ko {
                                format!(
                                    "골드 {}  최고 {:02}:{:02}  처치 {}",
                                    gold,
                                    (best as u32) / 60,
                                    (best as u32) % 60,
                                    kills
                                )
                            } else {
                                format!(
                                    "Gold {}  Best {:02}:{:02}  Kills {}",
                                    gold,
                                    (best as u32) / 60,
                                    (best as u32) % 60,
                                    kills
                                )
                            },
                            position: Vec2::new(cx - 230.0, cy + 130.0),
                            size: 16.0,
                            color: [200, 200, 200, 255],
                        });
                    }
                }
                return;
            }
            SurvivorMode::CharacterSelect => {
                let cx = vw / 2.0;
                let cursor_idx = world
                    .resource::<CharacterCursor>()
                    .map(|c| c.index)
                    .unwrap_or(0);
                let meta_snapshot = world.resource::<MetaSave>().cloned().unwrap_or_default();
                let gold = meta_snapshot.gold_total;
                if let Some(uq) = world.resource_mut::<UiQueue>() {
                    let hy = 90.0 + cursor_idx as f32 * 32.0;
                    uq.push(
                        DrawRect::new(cx - 290.0, hy, 580.0, 28.0, [0.25, 0.22, 0.05, 0.75])
                            .with_z(0.2),
                    );
                }
                if let Some(q) = world.resource_mut::<TextQueue>() {
                    q.push(DrawText {
                        text: text(lang, UiText::CharacterSelect).to_string(),
                        position: Vec2::new(cx - 150.0, 30.0),
                        size: 32.0,
                        color: [255, 220, 80, 255],
                    });
                    q.push(DrawText {
                        text: format!("{} {}", text(lang, UiText::Gold), gold),
                        position: Vec2::new(cx + 150.0, 30.0),
                        size: 20.0,
                        color: [255, 255, 100, 255],
                    });
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
                            [120, 120, 120, 255]
                        };
                        q.push(DrawText {
                            text: format!("{} {} {}", prefix, kind.label(lang), lock_str),
                            position: Vec2::new(cx - 280.0, 96.0 + i as f32 * 32.0),
                            size: 18.0,
                            color,
                        });
                    }
                    q.push(DrawText {
                        text: text(lang, UiText::NavigateSelectBack).to_string(),
                        position: Vec2::new(cx - 250.0, vh * 0.70),
                        size: 14.0,
                        color: [180, 180, 180, 255],
                    });
                }
                return;
            }
            SurvivorMode::StageSelect => {
                let cx = vw / 2.0;
                let cursor_idx = world
                    .resource::<StageCursor>()
                    .map(|c| c.index)
                    .unwrap_or(0);
                let unlocked: Vec<String> = world
                    .resource::<MetaSave>()
                    .map(|m| m.unlocked_stages.clone())
                    .unwrap_or_else(|| vec!["MadForest".to_string()]);
                if let Some(uq) = world.resource_mut::<UiQueue>() {
                    let hy = 90.0 + cursor_idx as f32 * 40.0;
                    uq.push(
                        DrawRect::new(cx - 290.0, hy, 580.0, 34.0, [0.25, 0.22, 0.05, 0.75])
                            .with_z(0.2),
                    );
                }
                if let Some(q) = world.resource_mut::<TextQueue>() {
                    q.push(DrawText {
                        text: text(lang, UiText::StageSelect).to_string(),
                        position: Vec2::new(cx - 120.0, 30.0),
                        size: 32.0,
                        color: [255, 220, 80, 255],
                    });
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
                            [120, 120, 120, 255]
                        };
                        q.push(DrawText {
                            text: format!("{} {} {}", prefix, stage.label(lang), lock_str),
                            position: Vec2::new(cx - 280.0, 96.0 + i as f32 * 40.0),
                            size: 22.0,
                            color,
                        });
                    }
                    q.push(DrawText {
                        text: text(lang, UiText::NavigateSelectBack).to_string(),
                        position: Vec2::new(cx - 250.0, vh * 0.70),
                        size: 14.0,
                        color: [180, 180, 180, 255],
                    });
                }
                return;
            }
            SurvivorMode::StageClear => {
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
                if let Some(uq) = world.resource_mut::<UiQueue>() {
                    uq.push(DrawRect::new(0.0, 0.0, vw, vh, [0.0, 0.05, 0.0, 0.5]).with_z(0.0));
                    uq.push(
                        DrawRect::new(cx - 204.0, cy - 154.0, 408.0, 308.0, [0.2, 0.25, 0.0, 1.0])
                            .with_z(0.1),
                    );
                    uq.push(
                        DrawRect::new(
                            cx - 200.0,
                            cy - 150.0,
                            400.0,
                            300.0,
                            [0.03, 0.06, 0.01, 0.96],
                        )
                        .with_z(0.11),
                    );
                }
                if let Some(q) = world.resource_mut::<TextQueue>() {
                    q.push(DrawText {
                        text: text(lang, UiText::StageClear).to_string(),
                        position: Vec2::new(cx - 160.0, cy - 138.0),
                        size: 44.0,
                        color: [255, 220, 80, 255],
                    });
                    q.push(DrawText {
                        text: if lang == Lang::Ko {
                            format!(
                                "시간: {:02}:{:02}  레벨: {}  처치: {}  골드: {}",
                                mm, ss, lv_stat, kills_stat, gold_stat
                            )
                        } else {
                            format!(
                                "Time: {:02}:{:02}  Lv: {}  Kills: {}  Gold: {}",
                                mm, ss, lv_stat, kills_stat, gold_stat
                            )
                        },
                        position: Vec2::new(cx - 185.0, cy - 10.0),
                        size: 18.0,
                        color: [200, 230, 200, 255],
                    });
                    q.push(DrawText {
                        text: text(lang, UiText::PressEnterReturn).to_string(),
                        position: Vec2::new(cx - 100.0, cy + 100.0),
                        size: 22.0,
                        color: [255, 255, 255, 255],
                    });
                }
                return;
            }
            SurvivorMode::Shop => {
                let cx = vw / 2.0;
                let meta_clone = world.resource::<MetaSave>().cloned();
                let cursor_idx = world.resource::<ShopCursor>().map(|c| c.index).unwrap_or(0);
                if let Some(uq) = world.resource_mut::<UiQueue>() {
                    let hy = 70.0 + cursor_idx as f32 * 26.0;
                    uq.push(
                        DrawRect::new(cx - 330.0, hy, 660.0, 22.0, [0.25, 0.22, 0.05, 0.75])
                            .with_z(0.2),
                    );
                }
                if let Some(q) = world.resource_mut::<TextQueue>() {
                    q.push(DrawText {
                        text: text(lang, UiText::PowerupShop).to_string(),
                        position: Vec2::new(cx - 250.0, 30.0),
                        size: 32.0,
                        color: [255, 220, 80, 255],
                    });
                    if let Some(meta) = &meta_clone {
                        q.push(DrawText {
                            text: format!("{} {}", text(lang, UiText::Gold), meta.gold_total),
                            position: Vec2::new(cx + 150.0, 30.0),
                            size: 20.0,
                            color: [255, 255, 100, 255],
                        });
                    }
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
                        q.push(DrawText {
                            text: format!(
                                "{} {:<10} Lv {}/{}  {}{}",
                                prefix,
                                kind.label(lang),
                                lv,
                                kind.max_level(),
                                text(lang, UiText::CostPrefix),
                                cost,
                            ),
                            position: Vec2::new(cx - 320.0, 76.0 + i as f32 * 26.0),
                            size: 16.0,
                            color,
                        });
                    }
                    q.push(DrawText {
                        text: text(lang, UiText::NavigateBuyBack).to_string(),
                        position: Vec2::new(
                            cx - 250.0,
                            76.0 + PowerUpKind::ALL.len() as f32 * 26.0 + 20.0,
                        ),
                        size: 14.0,
                        color: [180, 180, 180, 255],
                    });

                    if let Some(meta) = &meta_clone {
                        let done = AchievementKind::ALL
                            .iter()
                            .filter(|&&a| achievement_completed(meta, a))
                            .count();
                        let ax = (cx + 150.0).min(vw - 270.0);
                        q.push(DrawText {
                            text: format!(
                                "{} {}/{}",
                                text(lang, UiText::Achievements),
                                done,
                                AchievementKind::ALL.len()
                            ),
                            position: Vec2::new(ax, 76.0),
                            size: 18.0,
                            color: [255, 220, 80, 255],
                        });
                        for (i, &achievement) in AchievementKind::ALL.iter().take(8).enumerate() {
                            let completed = achievement_completed(meta, achievement);
                            let mark = if completed { "[x]" } else { "[ ]" };
                            let color = if completed {
                                [210, 240, 190, 255]
                            } else {
                                [135, 135, 145, 255]
                            };
                            q.push(DrawText {
                                text: format!(
                                    "{} {} - {}",
                                    mark,
                                    achievement.title(lang),
                                    achievement.reward(lang)
                                ),
                                position: Vec2::new(ax, 104.0 + i as f32 * 24.0),
                                size: 12.0,
                                color,
                            });
                        }
                    }
                }
                return;
            }
            SurvivorMode::PauseMenu => {
                let cursor_idx = world
                    .resource::<PauseMenuCursor>()
                    .map(|c| c.index)
                    .unwrap_or(0);
                let cx = vw / 2.0;
                let cy = vh / 2.0;
                let panel_w = 340.0_f32;
                let panel_h = 270.0_f32;
                let panel_x = cx - panel_w / 2.0;
                let panel_y = cy - panel_h / 2.0;
                const ITEM_H: f32 = 50.0;
                let item_y0 = panel_y + 70.0;

                // 어두운 전체 오버레이
                if let Some(uq) = world.resource_mut::<UiQueue>() {
                    uq.push(DrawRect::new(0.0, 0.0, vw, vh, [0.0, 0.0, 0.0, 0.62]).with_z(0.0));
                    // 패널 테두리
                    uq.push(
                        DrawRect::new(
                            panel_x - 2.0,
                            panel_y - 2.0,
                            panel_w + 4.0,
                            panel_h + 4.0,
                            [0.3, 0.25, 0.1, 1.0],
                        )
                        .with_z(0.1),
                    );
                    // 패널 본체
                    uq.push(
                        DrawRect::new(panel_x, panel_y, panel_w, panel_h, [0.06, 0.05, 0.02, 0.97])
                            .with_z(0.11),
                    );
                    // 선택 항목 하이라이트
                    let hy = item_y0 + cursor_idx as f32 * ITEM_H - 4.0;
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
                    q.push(DrawText {
                        text: text(lang, UiText::Paused).to_string(),
                        position: Vec2::new(panel_x + panel_w / 2.0 - 50.0, panel_y + 14.0),
                        size: 34.0,
                        color: [255, 220, 80, 255],
                    });
                    for (i, (label, _)) in labels.iter().enumerate().take(PAUSE_MENU_ITEMS) {
                        let color = if i == cursor_idx {
                            [255, 255, 80, 255]
                        } else {
                            [200, 200, 200, 255]
                        };
                        q.push(DrawText {
                            text: label.to_string(),
                            position: Vec2::new(panel_x + 20.0, item_y0 + i as f32 * ITEM_H),
                            size: 22.0,
                            color,
                        });
                    }
                    q.push(DrawText {
                        text: text(lang, UiText::PauseHelp).to_string(),
                        position: Vec2::new(panel_x + 10.0, panel_y + panel_h - 22.0),
                        size: 12.0,
                        color: [160, 160, 160, 255],
                    });
                }
                return;
            }
            SurvivorMode::Settings => {
                let cx = vw / 2.0;
                let cy = vh / 2.0;
                let panel_w = 620.0_f32;
                let panel_h = 360.0_f32;
                let px = cx - panel_w / 2.0;
                let py = cy - panel_h / 2.0;
                let cursor_idx = world
                    .resource::<SettingsCursor>()
                    .map(|c| c.index)
                    .unwrap_or(0);
                let meta = world.resource::<MetaSave>().cloned().unwrap_or_default();
                let selected_resolution = ResolutionPreset::from_key(&meta.resolution_key);

                if let Some(uq) = world.resource_mut::<UiQueue>() {
                    uq.push(DrawRect::new(0.0, 0.0, vw, vh, [0.0, 0.0, 0.0, 0.5]).with_z(0.0));
                    uq.push(
                        DrawRect::new(
                            px - 2.0,
                            py - 2.0,
                            panel_w + 4.0,
                            panel_h + 4.0,
                            [0.3, 0.25, 0.1, 1.0],
                        )
                        .with_z(0.1),
                    );
                    uq.push(
                        DrawRect::new(px, py, panel_w, panel_h, [0.06, 0.05, 0.02, 0.97])
                            .with_z(0.11),
                    );
                    // 선택 항목 하이라이트
                    let hy = py + 72.0 + cursor_idx as f32 * 42.0 - 4.0;
                    uq.push(
                        DrawRect::new(
                            px + 10.0,
                            hy,
                            panel_w - 20.0,
                            40.0,
                            [0.28, 0.22, 0.06, 0.85],
                        )
                        .with_z(0.2),
                    );
                }
                if let Some(q) = world.resource_mut::<TextQueue>() {
                    q.push(DrawText {
                        text: text(lang, UiText::Settings).to_string(),
                        position: Vec2::new(cx - 40.0, py + 12.0),
                        size: 30.0,
                        color: [255, 220, 80, 255],
                    });
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
                    for (i, (label, value)) in rows.iter().enumerate().take(SETTINGS_ITEMS) {
                        let is_sel = i == cursor_idx;
                        let color = if is_sel {
                            [255, 255, 80, 255]
                        } else {
                            [200, 200, 200, 255]
                        };
                        q.push(DrawText {
                            text: format!("{:<14}  < {} >", label, value),
                            position: Vec2::new(px + 20.0, py + 74.0 + i as f32 * 42.0),
                            size: 19.0,
                            color,
                        });
                    }
                    q.push(DrawText {
                        text: text(lang, UiText::NavigateChangeApplyBack).to_string(),
                        position: Vec2::new(px + 10.0, py + panel_h - 22.0),
                        size: 13.0,
                        color: [160, 160, 160, 255],
                    });
                }
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
            let hp_ratio = (hp / hp_max.max(1.0)).clamp(0.0, 1.0);
            let xp_ratio = if xp_max > 0 {
                (xp as f32 / xp_max as f32).clamp(0.0, 1.0)
            } else {
                0.0
            };

            if let Some(uq) = world.resource_mut::<UiQueue>() {
                // HP 바 배경
                uq.push(
                    DrawRect::new(
                        9.0,
                        7.0,
                        HP_BAR_W + 2.0,
                        HP_BAR_H + 2.0,
                        [0.1, 0.0, 0.0, 0.9],
                    )
                    .with_z(0.2),
                );
                // HP 바 fill
                if hp_ratio > 0.0 {
                    uq.push(
                        DrawRect::new(10.0, 8.0, HP_BAR_W * hp_ratio, HP_BAR_H, hp_color(hp_ratio))
                            .with_z(0.3),
                    );
                }
                // XP 바 배경
                let xp_y = vh - XP_BAR_H;
                uq.push(DrawRect::new(0.0, xp_y, vw, XP_BAR_H, [0.1, 0.1, 0.2, 0.85]).with_z(0.2));
                // XP 바 fill
                if xp_ratio > 0.0 {
                    uq.push(
                        DrawRect::new(0.0, xp_y, vw * xp_ratio, XP_BAR_H, [0.3, 0.7, 1.0, 1.0])
                            .with_z(0.3),
                    );
                }
            }

            // HP 수치 텍스트 (바 우측)
            let hp_text = format!("{:.0}/{:.0}", hp.max(0.0), hp_max);
            let xp_text = format!("{}/{}", xp, xp_max);
            let mut stat_line: Option<String> = None;
            let info_line = match (hud_detail, lang) {
                (HudDetail::Minimal, Lang::Ko) => {
                    format!(
                        "{:02}:{:02}  Lv {}  HP {}  XP {}  처치 {}",
                        mm, ss, lv, hp_text, xp_text, kills
                    )
                }
                (HudDetail::Minimal, Lang::En) => {
                    format!(
                        "{:02}:{:02}  Lv {}  HP {}  XP {}  Kills {}",
                        mm, ss, lv, hp_text, xp_text, kills
                    )
                }
                (HudDetail::Detailed, Lang::Ko) => {
                    if let Some(stats) = player_stats {
                        stat_line = Some(format!(
                            "공격 {:.0}%  쿨다운 {:.0}%  범위 {:.0}%  투사체 +{}  이동 {:.0}%",
                            stats.might * 100.0,
                            stats.cooldown * 100.0,
                            stats.area * 100.0,
                            stats.amount,
                            stats.move_speed * 100.0
                        ));
                    }
                    format!(
                        "{:02}:{:02}  Lv {}  HP {}  XP {}  골드 {}  처치 {}",
                        mm, ss, lv, hp_text, xp_text, gold, kills
                    )
                }
                (HudDetail::Detailed, Lang::En) => {
                    if let Some(stats) = player_stats {
                        stat_line = Some(format!(
                            "Might {:.0}%  Cooldown {:.0}%  Area {:.0}%  Amount +{}  Move {:.0}%",
                            stats.might * 100.0,
                            stats.cooldown * 100.0,
                            stats.area * 100.0,
                            stats.amount,
                            stats.move_speed * 100.0
                        ));
                    }
                    format!(
                        "{:02}:{:02}  Lv {}  HP {}  XP {}  Gold {}  Kills {}",
                        mm, ss, lv, hp_text, xp_text, gold, kills
                    )
                }
                (HudDetail::Normal, Lang::Ko) => {
                    format!(
                        "{:02}:{:02}  Lv {}  HP {}  XP {}  골드 {}  패시브 {}  처치 {}",
                        mm, ss, lv, hp_text, xp_text, gold, passive_count, kills
                    )
                }
                (HudDetail::Normal, Lang::En) => {
                    format!(
                        "{:02}:{:02}  Lv {}  HP {}  XP {}  Gold {}  Passives {}  Kills {}",
                        mm, ss, lv, hp_text, xp_text, gold, passive_count, kills
                    )
                }
            };
            if let Some(q) = world.resource_mut::<TextQueue>() {
                q.push(DrawText {
                    text: hp_text,
                    position: Vec2::new(10.0 + HP_BAR_W + 4.0, 8.0),
                    size: 13.0,
                    color: [255, 255, 255, 255],
                });
                q.push(DrawText {
                    text: info_line,
                    position: Vec2::new(10.0, 26.0),
                    size: 14.0,
                    color: [210, 210, 210, 255],
                });
                if let Some(stat_line) = stat_line {
                    q.push(DrawText {
                        text: stat_line,
                        position: Vec2::new(10.0, 42.0),
                        size: 12.0,
                        color: [185, 205, 220, 245],
                    });
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

        let slot_x0 = 10.0;
        let weapon_y = vh - XP_BAR_H - SLOT_H * 2.0 - SLOT_GAP - 8.0;
        let passive_y = weapon_y + SLOT_H + SLOT_GAP;
        let panel_w = SLOT_COLS as f32 * SLOT_W + (SLOT_COLS - 1) as f32 * SLOT_GAP;

        if let Some(uq) = world.resource_mut::<UiQueue>() {
            uq.push(
                DrawRect::new(
                    slot_x0 - 4.0,
                    weapon_y - 18.0,
                    panel_w + 8.0,
                    SLOT_H * 2.0 + SLOT_GAP + 24.0,
                    [0.02, 0.02, 0.03, 0.72],
                )
                .with_z(0.35),
            );
            for row in 0..2 {
                let y = if row == 0 { weapon_y } else { passive_y };
                for i in 0..SLOT_COLS {
                    let sx = slot_x0 + i as f32 * (SLOT_W + SLOT_GAP);
                    uq.push(
                        DrawRect::new(sx, y, SLOT_W, SLOT_H, [0.08, 0.08, 0.12, 0.88]).with_z(0.4),
                    );
                    let stripe = if row == 0 {
                        weapon_slots
                            .get(i)
                            .map(|(kind, _, _)| weapon_kind_color(kind))
                            .unwrap_or([0.18, 0.18, 0.22, 1.0])
                    } else {
                        passive_slots
                            .get(i)
                            .map(|(kind, _)| passive_kind_color(*kind))
                            .unwrap_or([0.18, 0.18, 0.22, 1.0])
                    };
                    uq.push(DrawRect::new(sx, y, 5.0, SLOT_H, stripe).with_z(0.5));
                }
            }
        }

        if let Some(q) = world.resource_mut::<TextQueue>() {
            q.push(DrawText {
                text: text(lang, UiText::Weapons).to_string(),
                position: Vec2::new(slot_x0, weapon_y - 16.0),
                size: 11.0,
                color: [180, 190, 220, 255],
            });
            q.push(DrawText {
                text: text(lang, UiText::Passives).to_string(),
                position: Vec2::new(slot_x0 + 52.0, weapon_y - 16.0),
                size: 11.0,
                color: [180, 190, 220, 255],
            });

            for i in 0..SLOT_COLS {
                let sx = slot_x0 + i as f32 * (SLOT_W + SLOT_GAP);
                let text = weapon_slots
                    .get(i)
                    .map(|(kind, level, evolved)| {
                        if *evolved {
                            format!("{} {} E", weapon_kind_name(kind, lang), level)
                        } else {
                            format!("{} {}", weapon_kind_name(kind, lang), level)
                        }
                    })
                    .unwrap_or_else(|| text(lang, UiText::EmptySlot).to_string());
                q.push(DrawText {
                    text,
                    position: Vec2::new(sx + 10.0, weapon_y + 5.0),
                    size: 12.0,
                    color: [235, 235, 240, 235],
                });
            }

            for i in 0..SLOT_COLS {
                let sx = slot_x0 + i as f32 * (SLOT_W + SLOT_GAP);
                let text = passive_slots
                    .get(i)
                    .map(|(kind, level)| format!("{} {}", passive_kind_name(*kind, lang), level))
                    .unwrap_or_else(|| text(lang, UiText::EmptySlot).to_string());
                q.push(DrawText {
                    text,
                    position: Vec2::new(sx + 10.0, passive_y + 5.0),
                    size: 12.0,
                    color: [235, 235, 240, 235],
                });
            }
        }

        // 5) Paused + PendingLevelUp: 반투명 오버레이 + 카드 패널
        if matches!(state, GameState::Paused) {
            if let Some(p) = world.resource::<PendingLevelUp>() {
                if !p.consumed {
                    let offered = p.offered;
                    let cx = vw / 2.0;
                    let cy = vh / 2.0;
                    if let Some(uq) = world.resource_mut::<UiQueue>() {
                        uq.push(DrawRect::new(0.0, 0.0, vw, vh, [0.0, 0.0, 0.0, 0.55]).with_z(0.0));
                        uq.push(
                            DrawRect::new(
                                cx - 204.0,
                                cy - 129.0,
                                408.0,
                                258.0,
                                [0.25, 0.18, 0.0, 1.0],
                            )
                            .with_z(0.1),
                        );
                        uq.push(
                            DrawRect::new(
                                cx - 200.0,
                                cy - 125.0,
                                400.0,
                                250.0,
                                [0.07, 0.05, 0.01, 0.96],
                            )
                            .with_z(0.11),
                        );
                        for ci in 0u32..3 {
                            let oy = cy - 50.0 + ci as f32 * 47.0;
                            uq.push(
                                DrawRect::new(
                                    cx - 190.0,
                                    oy,
                                    380.0,
                                    38.0,
                                    [0.15, 0.12, 0.03, 0.85],
                                )
                                .with_z(0.2),
                            );
                        }
                    }
                    if let Some(q) = world.resource_mut::<TextQueue>() {
                        q.push(DrawText {
                            text: text(lang, UiText::LevelUp).to_string(),
                            position: Vec2::new(cx - 60.0, cy - 117.0),
                            size: 38.0,
                            color: [255, 220, 80, 255],
                        });
                        q.push(DrawText {
                            text: format!("1.  {}", offered[0].label(lang)),
                            position: Vec2::new(cx - 180.0, cy - 42.0),
                            size: 19.0,
                            color: [255, 255, 255, 255],
                        });
                        q.push(DrawText {
                            text: format!("2.  {}", offered[1].label(lang)),
                            position: Vec2::new(cx - 180.0, cy + 5.0),
                            size: 19.0,
                            color: [255, 255, 255, 255],
                        });
                        q.push(DrawText {
                            text: format!("3.  {}", offered[2].label(lang)),
                            position: Vec2::new(cx - 180.0, cy + 52.0),
                            size: 19.0,
                            color: [255, 255, 255, 255],
                        });
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
            if let Some(uq) = world.resource_mut::<UiQueue>() {
                uq.push(DrawRect::new(0.0, 0.0, vw, vh, [0.0, 0.0, 0.0, 0.62]).with_z(0.0));
                uq.push(
                    DrawRect::new(cx - 252.0, cy - 172.0, 504.0, 344.0, [0.2, 0.0, 0.0, 1.0])
                        .with_z(0.1),
                );
                uq.push(
                    DrawRect::new(cx - 248.0, cy - 168.0, 496.0, 336.0, [0.04, 0.0, 0.0, 0.96])
                        .with_z(0.11),
                );
            }
            if let Some(q) = world.resource_mut::<TextQueue>() {
                q.push(DrawText {
                    text: text(lang, UiText::GameOver).to_string(),
                    position: Vec2::new(cx - 80.0, cy - 158.0),
                    size: 52.0,
                    color: [255, 60, 60, 255],
                });
                q.push(DrawText {
                    text: if lang == Lang::Ko {
                        format!(
                            "시간 {:02}:{:02}  레벨 {}  처치 {}  골드 {}",
                            mm2, ss2, lv_stat, kills, gold
                        )
                    } else {
                        format!(
                            "Time {:02}:{:02}  Lv {}  Kills {}  Gold {}",
                            mm2, ss2, lv_stat, kills, gold
                        )
                    },
                    position: Vec2::new(cx - 220.0, cy - 20.0),
                    size: 18.0,
                    color: [220, 180, 180, 255],
                });
                q.push(DrawText {
                    text: text(lang, UiText::RestartHint).to_string(),
                    position: Vec2::new(cx - 90.0, cy + 130.0),
                    size: 22.0,
                    color: [255, 255, 255, 255],
                });
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
            let boss_bar_x = (vw - BOSS_BAR_W) / 2.0;
            const BOSS_BAR_Y: f32 = 50.0;
            if let Some(uq) = world.resource_mut::<UiQueue>() {
                uq.push(
                    DrawRect::new(
                        boss_bar_x - 2.0,
                        BOSS_BAR_Y - 2.0,
                        BOSS_BAR_W + 4.0,
                        BOSS_BAR_H + 4.0,
                        [0.0, 0.0, 0.0, 0.9],
                    )
                    .with_z(0.2),
                );
                if hp_ratio > 0.0 {
                    uq.push(
                        DrawRect::new(
                            boss_bar_x,
                            BOSS_BAR_Y,
                            BOSS_BAR_W * hp_ratio,
                            BOSS_BAR_H,
                            phase_color,
                        )
                        .with_z(0.3),
                    );
                }
            }
            if let Some(q) = world.resource_mut::<TextQueue>() {
                q.push(DrawText {
                    text: format!("{}  {:.0}/{:.0}", kind.label(lang), hp.max(0.0), max),
                    position: Vec2::new(boss_bar_x, BOSS_BAR_Y - 18.0),
                    size: 14.0,
                    color: [255, 120, 120, 255],
                });
            }
        }

        // 8) 데미지 숫자(floating combat text) — 월드→화면 좌표 변환 후 TextQueue 에 push
        let cam_pos = world
            .resource::<Camera>()
            .map(|c| c.position)
            .unwrap_or(Vec2::ZERO);
        let damage_items: Vec<(Vec2, f32, u8)> = world
            .query2::<DamageNumber, Transform>()
            .filter_map(|(_, dn, t)| {
                let screen = t.position - cam_pos;
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
                    q.push(DrawText {
                        text: format!("{:.0}", value),
                        position: pos,
                        size: 16.0,
                        color: [255, 230, 120, alpha],
                    });
                }
            }
        }
    }
}
