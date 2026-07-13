use super::locale::{loc, Lang};
use super::meta::{MetaSave, SurvivorMode};
use super::player::PlayerStats;
/// Phase 8-B: PowerUp 매장 19종.
///
/// - `PowerUpKind` — 19종 영구 강화 종류 (Reroll/Skip/Banish 포함).
/// - `ShopCursor` — Shop 화면 커서 위치.
/// - `ShopInputSystem` — W/S/Enter/Esc 입력 처리.
/// - `try_purchase` — gold 차감 + level 증가.
/// - `apply_powerups` — StatRecalcSystem 에서 패시브 합산 *후* 호출.
use engine::{InputState, KeyCode, System, World};

// ─── PowerUpKind ──────────────────────────────────────────────────────────────

/// 영구 PowerUp 종류 (Phase 8-B). 기본 18종 + Banish = 19종.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PowerUpKind {
    Might,     // +1 might per lv
    Armor,     // +1 armor
    MaxHealth, // +20 max_health
    Recovery,  // +0.5/s
    Cooldown,  // *0.95 per lv
    Area,      // *1.05
    Speed,     // *1.05 (projectile_speed)
    Duration,  // *1.05
    Amount,    // +1
    MoveSpeed, // *1.05
    Magnet,    // *1.10
    Growth,    // *1.05
    Greed,     // *1.05
    Luck,      // *1.05
    Curse,     // 적 HP/속도/접촉 피해 강화
    Revival,   // +1
    Reroll,    // +1 (LevelUp 카드 reroll 횟수)
    Skip,      // +1 (LevelUp skip 횟수)
    Banish,    // +1 (LevelUp banish 횟수)
}

impl PowerUpKind {
    pub const ALL: &'static [PowerUpKind] = &[
        PowerUpKind::Might,
        PowerUpKind::Armor,
        PowerUpKind::MaxHealth,
        PowerUpKind::Recovery,
        PowerUpKind::Cooldown,
        PowerUpKind::Area,
        PowerUpKind::Speed,
        PowerUpKind::Duration,
        PowerUpKind::Amount,
        PowerUpKind::MoveSpeed,
        PowerUpKind::Magnet,
        PowerUpKind::Growth,
        PowerUpKind::Greed,
        PowerUpKind::Luck,
        PowerUpKind::Curse,
        PowerUpKind::Revival,
        PowerUpKind::Reroll,
        PowerUpKind::Skip,
        PowerUpKind::Banish,
    ];

    /// MetaSave.powerup_levels 에서 사용하는 문자열 키.
    pub fn key(self) -> &'static str {
        match self {
            PowerUpKind::Might => "Might",
            PowerUpKind::Armor => "Armor",
            PowerUpKind::MaxHealth => "MaxHealth",
            PowerUpKind::Recovery => "Recovery",
            PowerUpKind::Cooldown => "Cooldown",
            PowerUpKind::Area => "Area",
            PowerUpKind::Speed => "Speed",
            PowerUpKind::Duration => "Duration",
            PowerUpKind::Amount => "Amount",
            PowerUpKind::MoveSpeed => "MoveSpeed",
            PowerUpKind::Magnet => "Magnet",
            PowerUpKind::Growth => "Growth",
            PowerUpKind::Greed => "Greed",
            PowerUpKind::Luck => "Luck",
            PowerUpKind::Curse => "Curse",
            PowerUpKind::Revival => "Revival",
            PowerUpKind::Reroll => "Reroll",
            PowerUpKind::Skip => "Skip",
            PowerUpKind::Banish => "Banish",
        }
    }

    /// 최대 레벨 — Reroll/Skip/Banish 는 3, 나머지는 5.
    pub fn max_level(self) -> u8 {
        match self {
            PowerUpKind::Reroll | PowerUpKind::Skip | PowerUpKind::Banish => 3,
            _ => 5,
        }
    }

    /// 표시 이름 (한/영).
    pub fn label(self, lang: Lang) -> &'static str {
        match self {
            PowerUpKind::Might => loc(lang, "공격력", "Might"),
            PowerUpKind::Armor => loc(lang, "방어력", "Armor"),
            PowerUpKind::MaxHealth => loc(lang, "최대 HP", "Max Health"),
            PowerUpKind::Recovery => loc(lang, "회복", "Recovery"),
            PowerUpKind::Cooldown => loc(lang, "쿨다운", "Cooldown"),
            PowerUpKind::Area => loc(lang, "범위", "Area"),
            PowerUpKind::Speed => loc(lang, "투사체 속도", "Proj Speed"),
            PowerUpKind::Duration => loc(lang, "지속 시간", "Duration"),
            PowerUpKind::Amount => loc(lang, "수량", "Amount"),
            PowerUpKind::MoveSpeed => loc(lang, "이동 속도", "Move Speed"),
            PowerUpKind::Magnet => loc(lang, "자력", "Magnet"),
            PowerUpKind::Growth => loc(lang, "성장", "Growth"),
            PowerUpKind::Greed => loc(lang, "탐욕", "Greed"),
            PowerUpKind::Luck => loc(lang, "운", "Luck"),
            PowerUpKind::Curse => loc(lang, "저주", "Curse"),
            PowerUpKind::Revival => loc(lang, "부활", "Revival"),
            PowerUpKind::Reroll => loc(lang, "재추첨", "Reroll"),
            PowerUpKind::Skip => loc(lang, "건너뛰기", "Skip"),
            PowerUpKind::Banish => loc(lang, "추방", "Banish"),
        }
    }

    /// 다음 레벨 구매 비용 — (current_lv + 1) * 100.
    pub fn cost(self, current_lv: u8) -> u32 {
        (current_lv as u32 + 1) * 100
    }
}

// ─── ShopCursor ───────────────────────────────────────────────────────────────

/// Shop 화면 커서 위치 (선택된 PowerUp 인덱스).
#[derive(Debug, Default, Clone, Copy)]
pub struct ShopCursor {
    pub index: usize,
}

// ─── ShopInputSystem ──────────────────────────────────────────────────────────

/// Shop 입력 시스템 — SurvivorMode::Shop 일 때만 동작.
///
/// - W/↑ — 커서 위 이동
/// - S/↓ — 커서 아래 이동
/// - Enter — 선택 항목 구매 시도
/// - Esc  — Title 복귀 + 자동 저장
pub struct ShopInputSystem;

impl System for ShopInputSystem {
    fn run(&mut self, world: &mut World, _dt: f32) {
        let mode = world
            .resource::<SurvivorMode>()
            .copied()
            .unwrap_or(SurvivorMode::Title);
        if !matches!(mode, SurvivorMode::Shop) {
            return;
        }

        // 입력 캐시 (borrow 즉시 끊기)
        // 이유: InputState 를 읽은 뒤 resource_mut::<ShopCursor> 와 충돌하지 않도록.
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

        // 커서 이동
        if up || down {
            if let Some(cursor) = world.resource_mut::<ShopCursor>() {
                if up && cursor.index > 0 {
                    cursor.index -= 1;
                }
                if down && cursor.index < PowerUpKind::ALL.len() - 1 {
                    cursor.index += 1;
                }
            }
        }

        // 구매
        if enter {
            let idx = world.resource::<ShopCursor>().map(|c| c.index).unwrap_or(0);
            let kind = PowerUpKind::ALL[idx];
            try_purchase(world, kind);
        }

        // Title 복귀
        if esc {
            // 메타 저장 (borrow 분리)
            if let Some(meta) = world.resource::<MetaSave>() {
                meta.save_to_disk();
            }
            if let Some(m) = world.resource_mut::<SurvivorMode>() {
                *m = SurvivorMode::Title;
            }
        }
    }
}

// ─── try_purchase ─────────────────────────────────────────────────────────────

/// 구매 시도.
///
/// gold 충분 + 미최대 레벨이면 구매 성공 (gold 차감, level +1, 즉시 저장).
/// 실패 시 `false` 반환.
pub fn try_purchase(world: &mut World, kind: PowerUpKind) -> bool {
    let key = kind.key().to_string();
    let mut purchased = false;

    if let Some(meta) = world.resource_mut::<MetaSave>() {
        let current = *meta.powerup_levels.get(&key).unwrap_or(&0);
        if current >= kind.max_level() {
            println!("PowerUp {key} already maxed (lv {current})");
            return false;
        }
        let cost = kind.cost(current);
        if meta.gold_total < cost {
            println!(
                "Not enough gold for {} (need {}, have {})",
                key, cost, meta.gold_total
            );
            return false;
        }
        meta.gold_total -= cost;
        meta.powerup_levels.insert(key.clone(), current + 1);
        purchased = true;
        println!("Purchased {} Lv {} (cost {})", key, current + 1, cost);
    }

    if purchased {
        // borrow 분리 후 저장
        if let Some(meta) = world.resource::<MetaSave>() {
            meta.save_to_disk();
        }
    }
    purchased
}

// ─── apply_powerups ───────────────────────────────────────────────────────────

/// PlayerStats 에 PowerUp 효과 합산.
///
/// StatRecalcSystem 이 PassiveInventory 합산 *후* 이 함수를 호출해야 한다.
/// Reroll/Skip/Banish 는 LevelUp 카드 시스템에서 직접 읽으므로 여기서는 no-op.
pub fn apply_powerups(stats: &mut PlayerStats, meta: &MetaSave) {
    for kind in PowerUpKind::ALL {
        let key = kind.key();
        let lv = *meta.powerup_levels.get(key).unwrap_or(&0);
        if lv == 0 {
            continue;
        }
        let lvf = lv as f32;
        match kind {
            PowerUpKind::Might => stats.might += lvf,
            PowerUpKind::Armor => stats.armor += lvf,
            PowerUpKind::MaxHealth => stats.max_health += 20.0 * lvf,
            PowerUpKind::Recovery => stats.recovery += 0.5 * lvf,
            PowerUpKind::Cooldown => stats.cooldown *= 0.95_f32.powf(lvf),
            PowerUpKind::Area => stats.area *= 1.05_f32.powf(lvf),
            PowerUpKind::Speed => stats.projectile_speed *= 1.05_f32.powf(lvf),
            PowerUpKind::Duration => stats.duration *= 1.05_f32.powf(lvf),
            PowerUpKind::Amount => stats.amount += lv as i32,
            PowerUpKind::MoveSpeed => stats.move_speed *= 1.05_f32.powf(lvf),
            PowerUpKind::Magnet => stats.magnet *= 1.10_f32.powf(lvf),
            PowerUpKind::Growth => stats.growth *= 1.05_f32.powf(lvf),
            PowerUpKind::Greed => stats.greed *= 1.05_f32.powf(lvf),
            PowerUpKind::Luck => stats.luck *= 1.05_f32.powf(lvf),
            PowerUpKind::Curse => stats.curse *= 1.05_f32.powf(lvf),
            PowerUpKind::Revival => stats.revival += lv as u32,
            // LevelUp 카드 시스템에서 직접 소비 — placeholder
            PowerUpKind::Reroll | PowerUpKind::Skip | PowerUpKind::Banish => {}
        }
    }
}

// ─── 테스트 ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use engine::World;
    use std::collections::HashMap;

    #[test]
    fn powerup_count_is_nineteen() {
        assert_eq!(
            PowerUpKind::ALL.len(),
            19,
            "PowerUpKind::ALL 은 Reroll/Skip/Banish 포함 19종이어야 함"
        );
    }

    #[test]
    fn try_purchase_succeeds_with_enough_gold() {
        let mut world = World::new();
        let meta = MetaSave {
            gold_total: 200,
            powerup_levels: HashMap::new(),
            ..MetaSave::default()
        };
        world.insert_resource(meta);
        world.insert_resource(ShopCursor::default());

        let ok = try_purchase(&mut world, PowerUpKind::Might);
        assert!(ok, "gold 200 으로 Might Lv1(cost 100) 구매 성공해야 함");

        let gold_left = world
            .resource::<MetaSave>()
            .map(|m| m.gold_total)
            .unwrap_or(0);
        assert_eq!(gold_left, 100, "구매 후 gold 100 남아야 함");

        let lv = world
            .resource::<MetaSave>()
            .and_then(|m| m.powerup_levels.get("Might").copied())
            .unwrap_or(0);
        assert_eq!(lv, 1, "Might 레벨이 1 이어야 함");
    }

    #[test]
    fn apply_powerups_increases_might() {
        let mut stats = PlayerStats::default();
        let mut meta = MetaSave::default();
        meta.powerup_levels.insert("Might".to_string(), 3);

        apply_powerups(&mut stats, &meta);

        assert!(
            (stats.might - 3.0).abs() < 0.001,
            "Might Lv3 적용 후 might == 3.0 이어야 함 (현재 {})",
            stats.might
        );
    }
}
