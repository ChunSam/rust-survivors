/// Phase 9: 캐릭터 6종 + CharacterSelect 화면 + 해금 시스템.
///
/// - `CharacterKind` — 6종 캐릭터 (Antonio/Imelda/Pasqualina/Gennaro/Arca/Porta).
/// - `SelectedCharacter` — 현재 선택된 캐릭터 리소스.
/// - `CharacterCursor` — CharacterSelect 화면 커서 리소스.
/// - `CharacterSelectSystem` — W/S/Enter/Esc 입력 처리.
use engine::{InputState, System, World};
use serde::{Deserialize, Serialize};
use winit::keyboard::KeyCode;

use super::inventory::{WeaponInventory, WeaponKind, WeaponSlot};
use super::locale::{loc, Lang};
use super::meta::{MetaSave, SurvivorMode};
use super::player::PlayerStats;

// ─── CharacterKind ────────────────────────────────────────────────────────────

/// 캐릭터 6종 (Vampire Survivors 풍).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CharacterKind {
    #[default]
    Antonio, // 시작 Whip — 가장 기본
    Imelda,     // 시작 MagicWand
    Pasqualina, // 시작 Knife — amount +1
    Gennaro,    // 시작 Axe — might +1
    Arca,       // 시작 FireWand — might +2
    Porta,      // 시작 LightningRing — luck *1.2
}

impl CharacterKind {
    pub const ALL: &'static [CharacterKind] = &[
        CharacterKind::Antonio,
        CharacterKind::Imelda,
        CharacterKind::Pasqualina,
        CharacterKind::Gennaro,
        CharacterKind::Arca,
        CharacterKind::Porta,
    ];

    pub fn label(self, lang: Lang) -> &'static str {
        match self {
            CharacterKind::Antonio => loc(lang, "안토니오 (채찍)", "Antonio (Whip)"),
            CharacterKind::Imelda => loc(lang, "이멜다 (마법 지팡이)", "Imelda (MagicWand)"),
            CharacterKind::Pasqualina => loc(lang, "파스콸리나 (나이프)", "Pasqualina (Knife)"),
            CharacterKind::Gennaro => loc(lang, "젠나로 (도끼)", "Gennaro (Axe)"),
            CharacterKind::Arca => loc(lang, "아르카 (불 지팡이)", "Arca (FireWand)"),
            CharacterKind::Porta => loc(lang, "포르타 (번개)", "Porta (Lightning)"),
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            CharacterKind::Antonio => "Antonio",
            CharacterKind::Imelda => "Imelda",
            CharacterKind::Pasqualina => "Pasqualina",
            CharacterKind::Gennaro => "Gennaro",
            CharacterKind::Arca => "Arca",
            CharacterKind::Porta => "Porta",
        }
    }

    /// 해금 조건 — gold_total 누적치. Antonio 는 항상 해금.
    pub fn unlock_gold(self) -> u32 {
        match self {
            CharacterKind::Antonio => 0,
            CharacterKind::Imelda => 100,
            CharacterKind::Pasqualina => 300,
            CharacterKind::Gennaro => 600,
            CharacterKind::Arca => 1000,
            CharacterKind::Porta => 2000,
        }
    }

    pub fn is_unlocked(self, meta: &MetaSave) -> bool {
        self == CharacterKind::Antonio
            || meta.unlocked_chars.iter().any(|c| c == self.key())
            || meta.gold_total >= self.unlock_gold()
    }

    /// 캐릭터별 PlayerStats 보정 (default 위에 추가 적용).
    ///
    /// `StatRecalcSystem` 이 default + passive + powerup 합산 후 이 함수를 호출한다.
    pub fn apply_stats(self, stats: &mut PlayerStats) {
        match self {
            CharacterKind::Antonio => {} // 기본 — 보정 없음
            CharacterKind::Imelda => stats.growth *= 1.10,
            CharacterKind::Pasqualina => stats.amount += 1,
            CharacterKind::Gennaro => stats.might += 1.0,
            CharacterKind::Arca => stats.might += 2.0,
            CharacterKind::Porta => stats.luck *= 1.20,
        }
    }

    /// 캐릭터별 시작 인벤토리. 모든 캐릭터가 1개 무기로 시작한다.
    pub fn starter_inventory(self) -> WeaponInventory {
        let mut inv = WeaponInventory { slots: Vec::new() };
        let slot = match self {
            CharacterKind::Antonio => WeaponSlot {
                kind: WeaponKind::Whip {
                    damage: 10.0,
                    area_width: 120.0,
                    area_height: 60.0,
                },
                level: 1,
                cooldown: 1.0,
                elapsed: 0.0,
                evolved: false,
            },
            CharacterKind::Imelda => WeaponSlot {
                kind: WeaponKind::MagicWand {
                    damage: 8.0,
                    projectile_speed: 300.0,
                    lifetime: 1.5,
                    pierce: 0,
                },
                level: 1,
                cooldown: 1.2,
                elapsed: 0.0,
                evolved: false,
            },
            CharacterKind::Pasqualina => WeaponSlot {
                kind: WeaponKind::Knife {
                    damage: 6.0,
                    projectile_speed: 400.0,
                    lifetime: 1.0,
                    pierce: 0,
                    amount: 1,
                    spread_radians: 0.3,
                },
                level: 1,
                cooldown: 0.8,
                elapsed: 0.0,
                evolved: false,
            },
            CharacterKind::Gennaro => WeaponSlot {
                kind: WeaponKind::Axe {
                    damage: 12.0,
                    initial_speed: 250.0,
                    gravity: 600.0,
                    lifetime: 1.5,
                    pierce: 2,
                    amount: 1,
                },
                level: 1,
                cooldown: 1.5,
                elapsed: 0.0,
                evolved: false,
            },
            CharacterKind::Arca => WeaponSlot {
                kind: WeaponKind::FireWand {
                    damage: 25.0,
                    projectile_speed: 250.0,
                    lifetime: 1.5,
                    pierce: 0,
                },
                level: 1,
                cooldown: 2.5,
                elapsed: 0.0,
                evolved: false,
            },
            CharacterKind::Porta => WeaponSlot {
                kind: WeaponKind::LightningRing {
                    damage: 30.0,
                    strike_count: 1,
                    hit_radius: 40.0,
                },
                level: 1,
                cooldown: 4.0,
                elapsed: 0.0,
                evolved: false,
            },
        };
        inv.slots.push(slot);
        inv
    }
}

// ─── 리소스 ──────────────────────────────────────────────────────────────────

/// 현재 선택된 캐릭터를 보관하는 리소스.
#[derive(Debug, Default, Clone, Copy)]
pub struct SelectedCharacter(pub CharacterKind);

/// Character Select 화면 커서.
#[derive(Debug, Default, Clone, Copy)]
pub struct CharacterCursor {
    pub index: usize,
}

// ─── CharacterSelectSystem ────────────────────────────────────────────────────

/// CharacterSelect 모드에서 W/S/Enter/Esc 입력을 처리.
///
/// - W/↑ : 커서 위
/// - S/↓ : 커서 아래
/// - Enter : 선택 (해금 검증 포함) → Title 복귀
/// - Esc  : 취소 → Title 복귀
pub struct CharacterSelectSystem;

impl System for CharacterSelectSystem {
    fn run(&mut self, world: &mut World, _dt: f32) {
        let mode = world
            .resource::<SurvivorMode>()
            .copied()
            .unwrap_or(SurvivorMode::Title);
        if !matches!(mode, SurvivorMode::CharacterSelect) {
            return;
        }

        // 입력 캐시 (borrow 분리)
        let (up, down, enter, esc) = {
            let i = match world.resource::<InputState>() {
                Some(i) => i,
                None => return,
            };
            (
                i.just_pressed(KeyCode::KeyW) || i.just_pressed(KeyCode::ArrowUp),
                i.just_pressed(KeyCode::KeyS) || i.just_pressed(KeyCode::ArrowDown),
                i.just_pressed(KeyCode::Enter),
                i.just_pressed(KeyCode::Escape),
            )
        };

        let meta_snapshot = world.resource::<MetaSave>().cloned().unwrap_or_default();

        // 커서 이동
        if up || down {
            if let Some(cursor) = world.resource_mut::<CharacterCursor>() {
                if up && cursor.index > 0 {
                    cursor.index -= 1;
                }
                if down && cursor.index < CharacterKind::ALL.len() - 1 {
                    cursor.index += 1;
                }
            }
        }

        // 선택 확정
        if enter {
            let idx = world
                .resource::<CharacterCursor>()
                .map(|c| c.index)
                .unwrap_or(0);
            let kind = CharacterKind::ALL[idx];
            if kind.is_unlocked(&meta_snapshot) {
                if let Some(sel) = world.resource_mut::<SelectedCharacter>() {
                    sel.0 = kind;
                }
                if let Some(m) = world.resource_mut::<SurvivorMode>() {
                    *m = SurvivorMode::Title;
                }
                println!("Selected: {}", kind.label(Lang::En));
            } else {
                println!(
                    "{} locked (need {} gold, have {})",
                    kind.label(Lang::En),
                    kind.unlock_gold(),
                    meta_snapshot.gold_total
                );
            }
        }

        // 취소
        if esc {
            if let Some(m) = world.resource_mut::<SurvivorMode>() {
                *m = SurvivorMode::Title;
            }
        }
    }
}

// ─── 테스트 ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn character_kind_count_is_six() {
        assert_eq!(CharacterKind::ALL.len(), 6, "캐릭터는 정확히 6종이어야 함");
    }

    #[test]
    fn antonio_starter_inventory_has_whip() {
        let inv = CharacterKind::Antonio.starter_inventory();
        assert_eq!(inv.slots.len(), 1, "Antonio 는 시작 무기 1개만 가져야 함");
        assert!(
            matches!(inv.slots[0].kind, WeaponKind::Whip { .. }),
            "Antonio 의 시작 무기는 Whip 이어야 함"
        );
    }

    #[test]
    fn apply_stats_arca_increases_might() {
        let mut stats = PlayerStats::default(); // might == 0.0
        CharacterKind::Arca.apply_stats(&mut stats);
        assert!(
            (stats.might - 2.0).abs() < 0.001,
            "Arca apply_stats 후 might == 2.0 이어야 함 (현재 {})",
            stats.might
        );
    }
}
