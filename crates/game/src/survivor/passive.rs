/// Vampire Survivors 패시브 아이템 16종 + 인벤토리.
///
/// Phase 3-B 에서 도입. StatRecalcSystem 이 PassiveInventory 를 읽어
/// PlayerStats 를 매 프레임 재계산한다.
use super::locale::{loc, Lang};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PassiveKind {
    Spinach,       // +1 might per level
    Armor,         // +1 armor per level
    HollowHeart,   // +20 max_health per level
    Pummarola,     // +0.5 recovery per level
    EmptyTome,     // -8% cooldown (cooldown *= 0.92 per level)
    Candelabrador, // +10% area (area *= 1.10 per level)
    Bracer,        // +10% projectile_speed (곱)
    Spellbinder,   // +10% duration (곱)
    Duplicator,    // +1 amount per level
    Wings,         // +10% move_speed (move_speed *= 1.10 per level)
    Attractorb,    // +25% magnet (magnet *= 1.25 per level)
    Clover,        // +10% luck (placeholder, 곱)
    Crown,         // +8% growth (곱)
    StoneMask,     // +10% greed (placeholder, 곱)
    SkullOManiac,  // +10% curse (placeholder, 곱)
    Tiragisu,      // +1 revival per level
}

impl PassiveKind {
    /// 카드 풀 라벨용. 한 줄.
    pub fn label(self, lang: Lang) -> &'static str {
        match self {
            PassiveKind::Spinach => loc(lang, "시금치: 공격력 +1", "Spinach: MIGHT +1"),
            PassiveKind::Armor => loc(lang, "방어구: 방어력 +1", "Armor: ARMOR +1"),
            PassiveKind::HollowHeart => {
                loc(lang, "빈 심장: 최대 HP +20", "Hollow Heart: MAX HP +20")
            }
            PassiveKind::Pummarola => {
                loc(lang, "포마롤라: 회복 +0.5/초", "Pummarola: REGEN +0.5/s")
            }
            PassiveKind::EmptyTome => loc(lang, "빈 서적: 쿨다운 -8%", "Empty Tome: CD -8%"),
            PassiveKind::Candelabrador => loc(lang, "촛대: 범위 +10%", "Candelabrador: AREA +10%"),
            PassiveKind::Bracer => loc(lang, "팔찌: 투사체 속도 +10%", "Bracer: PROJ SPEED +10%"),
            PassiveKind::Spellbinder => {
                loc(lang, "마법 결속자: 지속 +10%", "Spellbinder: DURATION +10%")
            }
            PassiveKind::Duplicator => loc(lang, "복제기: 수량 +1", "Duplicator: AMOUNT +1"),
            PassiveKind::Wings => loc(lang, "날개: 이동 +10%", "Wings: MOVE +10%"),
            PassiveKind::Attractorb => loc(lang, "인력구: 자력 +25%", "Attractorb: MAGNET +25%"),
            PassiveKind::Clover => loc(lang, "클로버: 운 +10%", "Clover: LUCK +10%"),
            PassiveKind::Crown => loc(lang, "왕관: XP +8%", "Crown: XP +8%"),
            PassiveKind::StoneMask => loc(lang, "석가면: 탐욕 +10%", "Stone Mask: GREED +10%"),
            PassiveKind::SkullOManiac => {
                loc(lang, "해골 광: 저주 +10%", "Skull O'Maniac: CURSE +10%")
            }
            PassiveKind::Tiragisu => loc(lang, "티라지수: 부활 +1", "Tiragisu: REVIVAL +1"),
        }
    }

    /// 최대 레벨 (Vampire Survivors 원작: 5).
    pub const MAX_LEVEL: u8 = 5;
}

#[derive(Debug, Clone)]
pub struct PassiveSlot {
    pub kind: PassiveKind,
    pub level: u8, // 1 ~ MAX_LEVEL
}

/// Player 에 부착되는 패시브 인벤토리.
#[derive(Debug, Clone, Default)]
pub struct PassiveInventory {
    pub passives: Vec<PassiveSlot>,
}

impl PassiveInventory {
    /// 해당 패시브가 이미 있으면 +1 (max 까지). 없으면 신규 추가 (level 1).
    /// 반환값: 적용 후 level. max 도달 후 호출 시 변화 없음(현재 level 반환).
    pub fn add_or_levelup(&mut self, kind: PassiveKind) -> u8 {
        if let Some(slot) = self.passives.iter_mut().find(|s| s.kind == kind) {
            if slot.level < PassiveKind::MAX_LEVEL {
                slot.level += 1;
            }
            slot.level
        } else {
            self.passives.push(PassiveSlot { kind, level: 1 });
            1
        }
    }

    /// 특정 패시브 레벨 조회 (없으면 0).
    pub fn level_of(&self, kind: PassiveKind) -> u8 {
        self.passives
            .iter()
            .find(|s| s.kind == kind)
            .map(|s| s.level)
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passive_inventory_add_or_levelup() {
        let mut inv = PassiveInventory::default();

        // 처음 추가 → level 1
        assert_eq!(inv.add_or_levelup(PassiveKind::Spinach), 1);
        // 두 번째 → level 2
        assert_eq!(inv.add_or_levelup(PassiveKind::Spinach), 2);
        // 4번 더 → MAX_LEVEL(5) 도달
        inv.add_or_levelup(PassiveKind::Spinach);
        inv.add_or_levelup(PassiveKind::Spinach);
        inv.add_or_levelup(PassiveKind::Spinach);
        assert_eq!(inv.add_or_levelup(PassiveKind::Spinach), 5);
        // max 초과 호출 → 여전히 5
        assert_eq!(inv.add_or_levelup(PassiveKind::Spinach), 5);

        // level_of 확인
        assert_eq!(inv.level_of(PassiveKind::Spinach), 5);
        // 없는 패시브 → 0
        assert_eq!(inv.level_of(PassiveKind::Armor), 0);
    }
}
