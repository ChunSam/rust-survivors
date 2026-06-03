use super::inventory::{WeaponInventory, WeaponKind};
use super::locale::{loc, Lang};
use super::meta::MetaSave;
use super::passive::{PassiveInventory, PassiveKind};
use super::player::Player;
use super::powerup::PowerUpKind;
use super::sfx::{SfxEvent, SfxQueue};
use super::xp::XpAccumulator;
use engine::{GameState, InputState, System, World};
use rand::seq::SliceRandom;
use winit::keyboard::KeyCode;

/// 카드 강화 종류 — 10 무기 × 2~3 카드씩 총 28 variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CardKind {
    // ── Whip (3)
    WhipDamage,   // damage += 5
    WhipArea,     // area_width += 20, area_height += 10
    WhipCooldown, // cooldown *= 0.85

    // ── MagicWand (3)
    MagicWandDamage,   // damage += 4
    MagicWandSpeed,    // projectile_speed += 60
    MagicWandCooldown, // cooldown *= 0.85

    // ── Knife (3)
    KnifeDamage,   // damage += 4
    KnifeAmount,   // amount += 1
    KnifeCooldown, // cooldown *= 0.85

    // ── Axe (3)
    AxeDamage,   // damage += 5
    AxePierce,   // pierce += 1
    AxeCooldown, // cooldown *= 0.85

    // ── Cross (3)
    CrossDamage,   // damage += 6
    CrossReturnAt, // return_at += 0.1 (부메랑 더 늦게 반전 → 더 멀리 감)
    CrossCooldown, // cooldown *= 0.85

    // ── FireWand (2)
    FireWandDamage,   // damage += 8
    FireWandCooldown, // cooldown *= 0.85

    // ── Garlic (3)
    GarlicDamage,   // damage += 2
    GarlicRadius,   // radius += 15
    GarlicCooldown, // cooldown *= 0.85

    // ── HolyWater (3)
    HolyWaterDamage,    // damage += 3
    HolyWaterDropCount, // drop_count += 1
    HolyWaterCooldown,  // cooldown *= 0.85

    // ── KingBible (3)
    KingBibleDamage,    // damage += 3
    KingBibleBookCount, // book_count += 1
    KingBibleCooldown,  // cooldown *= 0.85

    // ── LightningRing (3)
    LightningDamage,      // damage += 8
    LightningStrikeCount, // strike_count += 1
    LightningCooldown,    // cooldown *= 0.85

    // ── Passives (16) — 각각 add_or_levelup
    PassiveSpinach,
    PassiveArmor,
    PassiveHollowHeart,
    PassivePummarola,
    PassiveEmptyTome,
    PassiveCandelabrador,
    PassiveBracer,
    PassiveSpellbinder,
    PassiveDuplicator,
    PassiveWings,
    PassiveAttractorb,
    PassiveClover,
    PassiveCrown,
    PassiveStoneMask,
    PassiveSkullOManiac,
    PassiveTiragisu,
}

/// 풀 카드 풀 — ALL_CARDS.choose_multiple 로 3장 랜덤 추출.
const ALL_CARDS: &[CardKind] = &[
    CardKind::WhipDamage,
    CardKind::WhipArea,
    CardKind::WhipCooldown,
    CardKind::MagicWandDamage,
    CardKind::MagicWandSpeed,
    CardKind::MagicWandCooldown,
    CardKind::KnifeDamage,
    CardKind::KnifeAmount,
    CardKind::KnifeCooldown,
    CardKind::AxeDamage,
    CardKind::AxePierce,
    CardKind::AxeCooldown,
    CardKind::CrossDamage,
    CardKind::CrossReturnAt,
    CardKind::CrossCooldown,
    CardKind::FireWandDamage,
    CardKind::FireWandCooldown,
    CardKind::GarlicDamage,
    CardKind::GarlicRadius,
    CardKind::GarlicCooldown,
    CardKind::HolyWaterDamage,
    CardKind::HolyWaterDropCount,
    CardKind::HolyWaterCooldown,
    CardKind::KingBibleDamage,
    CardKind::KingBibleBookCount,
    CardKind::KingBibleCooldown,
    CardKind::LightningDamage,
    CardKind::LightningStrikeCount,
    CardKind::LightningCooldown,
    // Passives
    CardKind::PassiveSpinach,
    CardKind::PassiveArmor,
    CardKind::PassiveHollowHeart,
    CardKind::PassivePummarola,
    CardKind::PassiveEmptyTome,
    CardKind::PassiveCandelabrador,
    CardKind::PassiveBracer,
    CardKind::PassiveSpellbinder,
    CardKind::PassiveDuplicator,
    CardKind::PassiveWings,
    CardKind::PassiveAttractorb,
    CardKind::PassiveClover,
    CardKind::PassiveCrown,
    CardKind::PassiveStoneMask,
    CardKind::PassiveSkullOManiac,
    CardKind::PassiveTiragisu,
];

impl CardKind {
    fn weapon_matches(self, weapon: &WeaponKind) -> bool {
        match self {
            CardKind::WhipDamage | CardKind::WhipArea | CardKind::WhipCooldown => {
                matches!(weapon, WeaponKind::Whip { .. })
            }
            CardKind::MagicWandDamage | CardKind::MagicWandSpeed | CardKind::MagicWandCooldown => {
                matches!(weapon, WeaponKind::MagicWand { .. })
            }
            CardKind::KnifeDamage | CardKind::KnifeAmount | CardKind::KnifeCooldown => {
                matches!(weapon, WeaponKind::Knife { .. })
            }
            CardKind::AxeDamage | CardKind::AxePierce | CardKind::AxeCooldown => {
                matches!(weapon, WeaponKind::Axe { .. })
            }
            CardKind::CrossDamage | CardKind::CrossReturnAt | CardKind::CrossCooldown => {
                matches!(weapon, WeaponKind::Cross { .. })
            }
            CardKind::FireWandDamage | CardKind::FireWandCooldown => {
                matches!(weapon, WeaponKind::FireWand { .. })
            }
            CardKind::GarlicDamage | CardKind::GarlicRadius | CardKind::GarlicCooldown => {
                matches!(weapon, WeaponKind::Garlic { .. })
            }
            CardKind::HolyWaterDamage
            | CardKind::HolyWaterDropCount
            | CardKind::HolyWaterCooldown => {
                matches!(weapon, WeaponKind::HolyWater { .. })
            }
            CardKind::KingBibleDamage
            | CardKind::KingBibleBookCount
            | CardKind::KingBibleCooldown => {
                matches!(weapon, WeaponKind::KingBible { .. })
            }
            CardKind::LightningDamage
            | CardKind::LightningStrikeCount
            | CardKind::LightningCooldown => matches!(weapon, WeaponKind::LightningRing { .. }),
            _ => false,
        }
    }

    fn passive_kind(self) -> Option<PassiveKind> {
        match self {
            CardKind::PassiveSpinach => Some(PassiveKind::Spinach),
            CardKind::PassiveArmor => Some(PassiveKind::Armor),
            CardKind::PassiveHollowHeart => Some(PassiveKind::HollowHeart),
            CardKind::PassivePummarola => Some(PassiveKind::Pummarola),
            CardKind::PassiveEmptyTome => Some(PassiveKind::EmptyTome),
            CardKind::PassiveCandelabrador => Some(PassiveKind::Candelabrador),
            CardKind::PassiveBracer => Some(PassiveKind::Bracer),
            CardKind::PassiveSpellbinder => Some(PassiveKind::Spellbinder),
            CardKind::PassiveDuplicator => Some(PassiveKind::Duplicator),
            CardKind::PassiveWings => Some(PassiveKind::Wings),
            CardKind::PassiveAttractorb => Some(PassiveKind::Attractorb),
            CardKind::PassiveClover => Some(PassiveKind::Clover),
            CardKind::PassiveCrown => Some(PassiveKind::Crown),
            CardKind::PassiveStoneMask => Some(PassiveKind::StoneMask),
            CardKind::PassiveSkullOManiac => Some(PassiveKind::SkullOManiac),
            CardKind::PassiveTiragisu => Some(PassiveKind::Tiragisu),
            _ => None,
        }
    }

    fn is_available_for(self, weapons: &WeaponInventory, passives: &PassiveInventory) -> bool {
        if let Some(passive) = self.passive_kind() {
            return passives.level_of(passive) < PassiveKind::MAX_LEVEL;
        }

        weapons
            .slots
            .iter()
            .any(|slot| slot.level < 8 && self.weapon_matches(&slot.kind))
    }

    pub fn label(self, lang: Lang) -> &'static str {
        match self {
            // Whip
            CardKind::WhipDamage => loc(lang, "채찍: 공격력 +5", "Whip: DMG +5"),
            CardKind::WhipArea => loc(lang, "채찍: 범위 UP", "Whip: AREA UP"),
            CardKind::WhipCooldown => loc(lang, "채찍: 쿨다운 -15%", "Whip: CD -15%"),
            // MagicWand
            CardKind::MagicWandDamage => loc(lang, "마법 지팡이: 공격력 +4", "Magic Wand: DMG +4"),
            CardKind::MagicWandSpeed => loc(lang, "마법 지팡이: 속도 +60", "Magic Wand: SPD +60"),
            CardKind::MagicWandCooldown => {
                loc(lang, "마법 지팡이: 쿨다운 -15%", "Magic Wand: CD -15%")
            }
            // Knife
            CardKind::KnifeDamage => loc(lang, "나이프: 공격력 +4", "Knife: DMG +4"),
            CardKind::KnifeAmount => loc(lang, "나이프: +1개", "Knife: +1 Knife"),
            CardKind::KnifeCooldown => loc(lang, "나이프: 쿨다운 -15%", "Knife: CD -15%"),
            // Axe
            CardKind::AxeDamage => loc(lang, "도끼: 공격력 +5", "Axe: DMG +5"),
            CardKind::AxePierce => loc(lang, "도끼: 관통 +1", "Axe: PIERCE +1"),
            CardKind::AxeCooldown => loc(lang, "도끼: 쿨다운 -15%", "Axe: CD -15%"),
            // Cross
            CardKind::CrossDamage => loc(lang, "크로스: 공격력 +6", "Cross: DMG +6"),
            CardKind::CrossReturnAt => loc(lang, "크로스: 범위 UP", "Cross: RANGE UP"),
            CardKind::CrossCooldown => loc(lang, "크로스: 쿨다운 -15%", "Cross: CD -15%"),
            // FireWand
            CardKind::FireWandDamage => loc(lang, "불 지팡이: 공격력 +8", "Fire Wand: DMG +8"),
            CardKind::FireWandCooldown => loc(lang, "불 지팡이: 쿨다운 -15%", "Fire Wand: CD -15%"),
            // Garlic
            CardKind::GarlicDamage => loc(lang, "마늘: 공격력 +2", "Garlic: DMG +2"),
            CardKind::GarlicRadius => loc(lang, "마늘: 범위 +15", "Garlic: AREA +15"),
            CardKind::GarlicCooldown => loc(lang, "마늘: 쿨다운 -15%", "Garlic: CD -15%"),
            // HolyWater
            CardKind::HolyWaterDamage => loc(lang, "성수: 공격력 +3", "Holy Water: DMG +3"),
            CardKind::HolyWaterDropCount => loc(lang, "성수: 웅덩이 +1", "Holy Water: +1 Pool"),
            CardKind::HolyWaterCooldown => loc(lang, "성수: 쿨다운 -15%", "Holy Water: CD -15%"),
            // KingBible
            CardKind::KingBibleDamage => loc(lang, "킹 바이블: 공격력 +3", "King Bible: DMG +3"),
            CardKind::KingBibleBookCount => loc(lang, "킹 바이블: 책 +1", "King Bible: +1 Book"),
            CardKind::KingBibleCooldown => {
                loc(lang, "킹 바이블: 쿨다운 -15%", "King Bible: CD -15%")
            }
            // LightningRing
            CardKind::LightningDamage => loc(lang, "번개: 공격력 +8", "Lightning: DMG +8"),
            CardKind::LightningStrikeCount => loc(lang, "번개: 타격 +1", "Lightning: +1 Strike"),
            CardKind::LightningCooldown => loc(lang, "번개: 쿨다운 -15%", "Lightning: CD -15%"),
            // Passives — PassiveKind::label 로 위임
            CardKind::PassiveSpinach => PassiveKind::Spinach.label(lang),
            CardKind::PassiveArmor => PassiveKind::Armor.label(lang),
            CardKind::PassiveHollowHeart => PassiveKind::HollowHeart.label(lang),
            CardKind::PassivePummarola => PassiveKind::Pummarola.label(lang),
            CardKind::PassiveEmptyTome => PassiveKind::EmptyTome.label(lang),
            CardKind::PassiveCandelabrador => PassiveKind::Candelabrador.label(lang),
            CardKind::PassiveBracer => PassiveKind::Bracer.label(lang),
            CardKind::PassiveSpellbinder => PassiveKind::Spellbinder.label(lang),
            CardKind::PassiveDuplicator => PassiveKind::Duplicator.label(lang),
            CardKind::PassiveWings => PassiveKind::Wings.label(lang),
            CardKind::PassiveAttractorb => PassiveKind::Attractorb.label(lang),
            CardKind::PassiveClover => PassiveKind::Clover.label(lang),
            CardKind::PassiveCrown => PassiveKind::Crown.label(lang),
            CardKind::PassiveStoneMask => PassiveKind::StoneMask.label(lang),
            CardKind::PassiveSkullOManiac => PassiveKind::SkullOManiac.label(lang),
            CardKind::PassiveTiragisu => PassiveKind::Tiragisu.label(lang),
        }
    }
}

/// LevelUp 진행 중에만 World 에 삽입되는 리소스.
///
/// 카드 선택이 끝나면 최신 엔진의 `World::remove_resource` 로 제거한다.
pub struct PendingLevelUp {
    pub offered: [CardKind; 3],
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LevelUpActions {
    pub reroll_remaining: u32,
    pub skip_remaining: u32,
    pub banish_remaining: u32,
    pub banished: Vec<CardKind>,
}

impl LevelUpActions {
    pub fn from_meta(meta: Option<&MetaSave>) -> Self {
        fn count(meta: Option<&MetaSave>, kind: PowerUpKind) -> u32 {
            meta.and_then(|m| m.powerup_levels.get(kind.key()).copied())
                .unwrap_or(0) as u32
        }

        Self {
            reroll_remaining: count(meta, PowerUpKind::Reroll),
            skip_remaining: count(meta, PowerUpKind::Skip),
            banish_remaining: count(meta, PowerUpKind::Banish),
            banished: Vec::new(),
        }
    }
}

/// 레벨업 감지 + 카드 선택 처리 시스템.
///
/// - 가드 없음: 이 시스템 자체가 `GameState` 전환을 담당하므로 외부 가드 불필요.
/// - 등록 순서: 시스템 목록 첫 번째 (상태 전환 → 나머지 시스템이 가드에서 조기 반환).
pub struct LevelUpSystem;

impl LevelUpSystem {
    /// 다음 레벨업 임계치. L1→5, L2→10, L3→15 …
    pub fn next_threshold(level: u32) -> u32 {
        5 + 5 * level
    }

    fn ensure_actions(world: &mut World) {
        if world.resource::<LevelUpActions>().is_some() {
            return;
        }
        let actions = LevelUpActions::from_meta(world.resource::<MetaSave>());
        world.insert_resource(actions);
    }

    fn available_cards_for(world: &World, player_entity: engine::Entity) -> Vec<CardKind> {
        let Some(weapon_inv) = world.get::<WeaponInventory>(player_entity) else {
            return Vec::new();
        };
        let Some(passive_inv) = world.get::<PassiveInventory>(player_entity) else {
            return Vec::new();
        };
        let banished = world
            .resource::<LevelUpActions>()
            .map(|actions| actions.banished.as_slice())
            .unwrap_or(&[]);
        ALL_CARDS
            .iter()
            .copied()
            .filter(|card| !banished.contains(card))
            .filter(|card| card.is_available_for(weapon_inv, passive_inv))
            .collect()
    }

    fn random_offer(available_cards: &[CardKind]) -> Option<[CardKind; 3]> {
        if available_cards.len() < 3 {
            return None;
        }
        let mut rng = rand::thread_rng();
        let chosen: Vec<CardKind> = available_cards
            .choose_multiple(&mut rng, 3)
            .copied()
            .collect();
        Some([chosen[0], chosen[1], chosen[2]])
    }

    fn replace_offer(world: &mut World, player_entity: engine::Entity) -> Option<[CardKind; 3]> {
        let available_cards = Self::available_cards_for(world, player_entity);
        let offered = Self::random_offer(&available_cards)?;
        world.insert_resource(PendingLevelUp { offered });
        Some(offered)
    }

    fn consume_level_without_card(
        world: &mut World,
        player_entity: engine::Entity,
    ) -> Option<(u32, u32)> {
        let (new_current, new_threshold) =
            if let Some(acc) = world.get_mut::<XpAccumulator>(player_entity) {
                acc.level += 1;
                acc.next_threshold = Self::next_threshold(acc.level);
                (acc.current, acc.next_threshold)
            } else {
                return None;
            };

        world.remove_resource::<PendingLevelUp>();
        if let Some(gs) = world.resource_mut::<GameState>() {
            *gs = GameState::Playing;
        }
        Some((new_current, new_threshold))
    }

    pub fn reroll_offer(world: &mut World, player_entity: engine::Entity) -> bool {
        let Some(actions) = world.resource::<LevelUpActions>() else {
            return false;
        };
        if actions.reroll_remaining == 0 {
            return false;
        }
        if Self::available_cards_for(world, player_entity).len() < 3 {
            return false;
        }
        if let Some(actions) = world.resource_mut::<LevelUpActions>() {
            actions.reroll_remaining -= 1;
        }
        Self::replace_offer(world, player_entity).is_some()
    }

    pub fn skip_levelup(world: &mut World, player_entity: engine::Entity) -> Option<(u32, u32)> {
        let Some(actions) = world.resource::<LevelUpActions>() else {
            return None;
        };
        if actions.skip_remaining == 0 {
            return None;
        }
        if let Some(actions) = world.resource_mut::<LevelUpActions>() {
            actions.skip_remaining -= 1;
        }
        Self::consume_level_without_card(world, player_entity)
    }

    pub fn banish_offer(world: &mut World, player_entity: engine::Entity, idx: usize) -> bool {
        let Some(pending) = world.resource::<PendingLevelUp>() else {
            return false;
        };
        let Some(card) = pending.offered.get(idx).copied() else {
            return false;
        };
        let Some(actions) = world.resource::<LevelUpActions>() else {
            return false;
        };
        if actions.banish_remaining == 0 || actions.banished.contains(&card) {
            return false;
        }

        let mut candidate_banished = actions.banished.clone();
        candidate_banished.push(card);
        let available_count = Self::available_cards_for(world, player_entity)
            .into_iter()
            .filter(|candidate| !candidate_banished.contains(candidate))
            .count();
        if available_count < 3 {
            return false;
        }

        if let Some(actions) = world.resource_mut::<LevelUpActions>() {
            actions.banish_remaining -= 1;
            actions.banished.push(card);
        }
        Self::replace_offer(world, player_entity).is_some()
    }

    /// 선택한 카드 효과를 Player 엔티티의 WeaponInventory 에 적용하고
    /// XpAccumulator 를 갱신한다.
    ///
    /// 키 입력 없이 직접 호출 가능하므로 단위 테스트에서 재사용.
    /// 반환: 적용 성공 시 (새 XP current, 새 threshold) — println 메시지 구성용.
    ///
    /// 보유하지 않은 무기 카드나 이미 최대 레벨인 카드는 실패로 처리한다.
    pub fn apply_card(
        world: &mut World,
        player_entity: engine::Entity,
        card: CardKind,
    ) -> Option<(u32, u32)> {
        let mut applied = false;
        if let Some(inv) = world.get_mut::<WeaponInventory>(player_entity) {
            match card {
                // ── Whip ────────────────────────────────────────────────────
                CardKind::WhipDamage => {
                    if let Some(slot) = inv.whip_slot_mut() {
                        if slot.level < 8 {
                            if let WeaponKind::Whip { damage, .. } = &mut slot.kind {
                                *damage += 5.0;
                            }
                            slot.level += 1;
                            applied = true;
                        }
                    }
                }
                CardKind::WhipArea => {
                    if let Some(slot) = inv.whip_slot_mut() {
                        if slot.level < 8 {
                            if let WeaponKind::Whip {
                                area_width,
                                area_height,
                                ..
                            } = &mut slot.kind
                            {
                                *area_width += 20.0;
                                *area_height += 10.0;
                            }
                            slot.level += 1;
                            applied = true;
                        }
                    }
                }
                CardKind::WhipCooldown => {
                    if let Some(slot) = inv.whip_slot_mut() {
                        if slot.level < 8 {
                            slot.cooldown *= 0.85;
                            slot.level += 1;
                            applied = true;
                        }
                    }
                }

                // ── MagicWand ────────────────────────────────────────────────
                CardKind::MagicWandDamage => {
                    if let Some(slot) = inv.magic_wand_slot_mut() {
                        if slot.level < 8 {
                            if let WeaponKind::MagicWand { damage, .. } = &mut slot.kind {
                                *damage += 4.0;
                            }
                            slot.level += 1;
                            applied = true;
                        }
                    }
                }
                CardKind::MagicWandSpeed => {
                    if let Some(slot) = inv.magic_wand_slot_mut() {
                        if slot.level < 8 {
                            if let WeaponKind::MagicWand {
                                projectile_speed, ..
                            } = &mut slot.kind
                            {
                                *projectile_speed += 60.0;
                            }
                            slot.level += 1;
                            applied = true;
                        }
                    }
                }
                CardKind::MagicWandCooldown => {
                    if let Some(slot) = inv.magic_wand_slot_mut() {
                        if slot.level < 8 {
                            slot.cooldown *= 0.85;
                            slot.level += 1;
                            applied = true;
                        }
                    }
                }

                // ── Knife ────────────────────────────────────────────────────
                CardKind::KnifeDamage => {
                    if let Some(slot) = inv.knife_slot_mut() {
                        if slot.level < 8 {
                            if let WeaponKind::Knife { damage, .. } = &mut slot.kind {
                                *damage += 4.0;
                            }
                            slot.level += 1;
                            applied = true;
                        }
                    }
                }
                CardKind::KnifeAmount => {
                    if let Some(slot) = inv.knife_slot_mut() {
                        if slot.level < 8 {
                            if let WeaponKind::Knife { amount, .. } = &mut slot.kind {
                                *amount += 1;
                            }
                            slot.level += 1;
                            applied = true;
                        }
                    }
                }
                CardKind::KnifeCooldown => {
                    if let Some(slot) = inv.knife_slot_mut() {
                        if slot.level < 8 {
                            slot.cooldown *= 0.85;
                            slot.level += 1;
                            applied = true;
                        }
                    }
                }

                // ── Axe ──────────────────────────────────────────────────────
                CardKind::AxeDamage => {
                    if let Some(slot) = inv.axe_slot_mut() {
                        if slot.level < 8 {
                            if let WeaponKind::Axe { damage, .. } = &mut slot.kind {
                                *damage += 5.0;
                            }
                            slot.level += 1;
                            applied = true;
                        }
                    }
                }
                CardKind::AxePierce => {
                    if let Some(slot) = inv.axe_slot_mut() {
                        if slot.level < 8 {
                            if let WeaponKind::Axe { pierce, .. } = &mut slot.kind {
                                *pierce += 1;
                            }
                            slot.level += 1;
                            applied = true;
                        }
                    }
                }
                CardKind::AxeCooldown => {
                    if let Some(slot) = inv.axe_slot_mut() {
                        if slot.level < 8 {
                            slot.cooldown *= 0.85;
                            slot.level += 1;
                            applied = true;
                        }
                    }
                }

                // ── Cross ────────────────────────────────────────────────────
                CardKind::CrossDamage => {
                    if let Some(slot) = inv.cross_slot_mut() {
                        if slot.level < 8 {
                            if let WeaponKind::Cross { damage, .. } = &mut slot.kind {
                                *damage += 6.0;
                            }
                            slot.level += 1;
                            applied = true;
                        }
                    }
                }
                CardKind::CrossReturnAt => {
                    if let Some(slot) = inv.cross_slot_mut() {
                        if slot.level < 8 {
                            if let WeaponKind::Cross { return_at, .. } = &mut slot.kind {
                                *return_at += 0.1;
                            }
                            slot.level += 1;
                            applied = true;
                        }
                    }
                }
                CardKind::CrossCooldown => {
                    if let Some(slot) = inv.cross_slot_mut() {
                        if slot.level < 8 {
                            slot.cooldown *= 0.85;
                            slot.level += 1;
                            applied = true;
                        }
                    }
                }

                // ── FireWand ─────────────────────────────────────────────────
                CardKind::FireWandDamage => {
                    if let Some(slot) = inv.fire_wand_slot_mut() {
                        if slot.level < 8 {
                            if let WeaponKind::FireWand { damage, .. } = &mut slot.kind {
                                *damage += 8.0;
                            }
                            slot.level += 1;
                            applied = true;
                        }
                    }
                }
                CardKind::FireWandCooldown => {
                    if let Some(slot) = inv.fire_wand_slot_mut() {
                        if slot.level < 8 {
                            slot.cooldown *= 0.85;
                            slot.level += 1;
                            applied = true;
                        }
                    }
                }

                // ── Garlic ───────────────────────────────────────────────────
                CardKind::GarlicDamage => {
                    if let Some(slot) = inv.garlic_slot_mut() {
                        if slot.level < 8 {
                            if let WeaponKind::Garlic { damage, .. } = &mut slot.kind {
                                *damage += 2.0;
                            }
                            slot.level += 1;
                            applied = true;
                        }
                    }
                }
                CardKind::GarlicRadius => {
                    if let Some(slot) = inv.garlic_slot_mut() {
                        if slot.level < 8 {
                            if let WeaponKind::Garlic { radius, .. } = &mut slot.kind {
                                *radius += 15.0;
                            }
                            slot.level += 1;
                            applied = true;
                        }
                    }
                }
                CardKind::GarlicCooldown => {
                    if let Some(slot) = inv.garlic_slot_mut() {
                        if slot.level < 8 {
                            slot.cooldown *= 0.85;
                            slot.level += 1;
                            applied = true;
                        }
                    }
                }

                // ── HolyWater ────────────────────────────────────────────────
                CardKind::HolyWaterDamage => {
                    if let Some(slot) = inv.holy_water_slot_mut() {
                        if slot.level < 8 {
                            if let WeaponKind::HolyWater { damage, .. } = &mut slot.kind {
                                *damage += 3.0;
                            }
                            slot.level += 1;
                            applied = true;
                        }
                    }
                }
                CardKind::HolyWaterDropCount => {
                    if let Some(slot) = inv.holy_water_slot_mut() {
                        if slot.level < 8 {
                            if let WeaponKind::HolyWater { drop_count, .. } = &mut slot.kind {
                                *drop_count += 1;
                            }
                            slot.level += 1;
                            applied = true;
                        }
                    }
                }
                CardKind::HolyWaterCooldown => {
                    if let Some(slot) = inv.holy_water_slot_mut() {
                        if slot.level < 8 {
                            slot.cooldown *= 0.85;
                            slot.level += 1;
                            applied = true;
                        }
                    }
                }

                // ── KingBible ────────────────────────────────────────────────
                CardKind::KingBibleDamage => {
                    if let Some(slot) = inv.king_bible_slot_mut() {
                        if slot.level < 8 {
                            if let WeaponKind::KingBible { damage, .. } = &mut slot.kind {
                                *damage += 3.0;
                            }
                            slot.level += 1;
                            applied = true;
                        }
                    }
                }
                CardKind::KingBibleBookCount => {
                    if let Some(slot) = inv.king_bible_slot_mut() {
                        if slot.level < 8 {
                            if let WeaponKind::KingBible { book_count, .. } = &mut slot.kind {
                                *book_count += 1;
                            }
                            slot.level += 1;
                            applied = true;
                        }
                    }
                }
                CardKind::KingBibleCooldown => {
                    if let Some(slot) = inv.king_bible_slot_mut() {
                        if slot.level < 8 {
                            slot.cooldown *= 0.85;
                            slot.level += 1;
                            applied = true;
                        }
                    }
                }

                // ── LightningRing ────────────────────────────────────────────
                CardKind::LightningDamage => {
                    if let Some(slot) = inv.lightning_ring_slot_mut() {
                        if slot.level < 8 {
                            if let WeaponKind::LightningRing { damage, .. } = &mut slot.kind {
                                *damage += 8.0;
                            }
                            slot.level += 1;
                            applied = true;
                        }
                    }
                }
                CardKind::LightningStrikeCount => {
                    if let Some(slot) = inv.lightning_ring_slot_mut() {
                        if slot.level < 8 {
                            if let WeaponKind::LightningRing { strike_count, .. } = &mut slot.kind {
                                *strike_count += 1;
                            }
                            slot.level += 1;
                            applied = true;
                        }
                    }
                }
                CardKind::LightningCooldown => {
                    if let Some(slot) = inv.lightning_ring_slot_mut() {
                        if slot.level < 8 {
                            slot.cooldown *= 0.85;
                            slot.level += 1;
                            applied = true;
                        }
                    }
                }

                // ── Passives ─────────────────────────────────────────────────
                // 무기 카드와 달리 WeaponInventory 가 아닌 PassiveInventory 를
                // 갱신해야 하므로 여기서는 no-op. match 종료 후 별도 분기에서 처리.
                CardKind::PassiveSpinach
                | CardKind::PassiveArmor
                | CardKind::PassiveHollowHeart
                | CardKind::PassivePummarola
                | CardKind::PassiveEmptyTome
                | CardKind::PassiveCandelabrador
                | CardKind::PassiveBracer
                | CardKind::PassiveSpellbinder
                | CardKind::PassiveDuplicator
                | CardKind::PassiveWings
                | CardKind::PassiveAttractorb
                | CardKind::PassiveClover
                | CardKind::PassiveCrown
                | CardKind::PassiveStoneMask
                | CardKind::PassiveSkullOManiac
                | CardKind::PassiveTiragisu => {}
            }
        }

        // 패시브 카드: PassiveInventory 에 add_or_levelup. WeaponInventory mut 빌림
        // 이 끝난 시점에 처리한다.
        if let Some(kind) = card.passive_kind() {
            if let Some(inv) = world.get_mut::<PassiveInventory>(player_entity) {
                let before = inv.level_of(kind);
                if before < PassiveKind::MAX_LEVEL {
                    inv.add_or_levelup(kind);
                    applied = true;
                }
            }
        }

        if !applied {
            return None;
        }

        // XpAccumulator: 현재 XP 유지, 레벨 +1, 다음 임계치 갱신
        let (new_current, new_threshold) =
            if let Some(acc) = world.get_mut::<XpAccumulator>(player_entity) {
                acc.level += 1;
                acc.next_threshold = Self::next_threshold(acc.level);
                (acc.current, acc.next_threshold)
            } else {
                (0, 0)
            };

        Some((new_current, new_threshold))
    }

    pub fn choose_card(
        world: &mut World,
        player_entity: engine::Entity,
        card: CardKind,
    ) -> Option<(u32, u32)> {
        let result = Self::apply_card(world, player_entity, card)?;
        world.remove_resource::<PendingLevelUp>();
        if let Some(gs) = world.resource_mut::<GameState>() {
            *gs = GameState::Playing;
        }
        Some(result)
    }
}

impl System for LevelUpSystem {
    fn run(&mut self, world: &mut World, _dt: f32) {
        let state = world.resource::<GameState>().cloned();

        let has_pending = world.resource::<PendingLevelUp>().is_some();

        match (state, has_pending) {
            // ── 평소: XP 임계치 도달 체크 ──────────────────────────────────────
            (Some(GameState::Playing), false) => {
                // XpAccumulator 캐시 (borrow 를 즉시 끊음)
                let player_data = world
                    .query2::<Player, XpAccumulator>()
                    .next()
                    .map(|(e, _, acc)| (e, acc.current, acc.level, acc.next_threshold));
                let Some((_player_entity, current, level, threshold)) = player_data else {
                    return;
                };

                if current >= threshold {
                    Self::ensure_actions(world);
                    let available_cards = Self::available_cards_for(world, _player_entity);
                    if available_cards.len() < 3 {
                        eprintln!(
                            "LEVEL UP skipped: available card count is {}",
                            available_cards.len()
                        );
                        return;
                    }

                    let offered = Self::random_offer(&available_cards)
                        .expect("available_cards length was checked above");

                    world.insert_resource(PendingLevelUp { offered });
                    if let Some(gs) = world.resource_mut::<GameState>() {
                        *gs = GameState::Paused;
                    }
                    if let Some(q) = world.resource_mut::<SfxQueue>() {
                        q.push(SfxEvent::LevelUp);
                    }

                    println!(
                        "LEVEL UP! (Level {}) — Press: 1={} 2={} 3={}",
                        level + 1,
                        offered[0].label(Lang::En),
                        offered[1].label(Lang::En),
                        offered[2].label(Lang::En)
                    );
                }
            }

            // ── 카드 선택 대기: 1/2/3 키 감지 ─────────────────────────────────
            (Some(GameState::Paused), true) => {
                // 키 입력 확인 — InputState borrow 를 블록 안에서 끝냄
                let action: Option<LevelUpInputAction> = {
                    let Some(input) = world.resource::<InputState>() else {
                        return;
                    };
                    if input.just_pressed(KeyCode::Digit1) {
                        Some(LevelUpInputAction::Choose(0))
                    } else if input.just_pressed(KeyCode::Digit2) {
                        Some(LevelUpInputAction::Choose(1))
                    } else if input.just_pressed(KeyCode::Digit3) {
                        Some(LevelUpInputAction::Choose(2))
                    } else if input.just_pressed(KeyCode::KeyR) {
                        Some(LevelUpInputAction::Reroll)
                    } else if input.just_pressed(KeyCode::KeyS) {
                        Some(LevelUpInputAction::Skip)
                    } else if input.just_pressed(KeyCode::Digit4) {
                        Some(LevelUpInputAction::Banish(0))
                    } else if input.just_pressed(KeyCode::Digit5) {
                        Some(LevelUpInputAction::Banish(1))
                    } else if input.just_pressed(KeyCode::Digit6) {
                        Some(LevelUpInputAction::Banish(2))
                    } else {
                        None
                    }
                };
                let Some(action) = action else { return };

                // Player 엔티티 조회 — borrow 를 즉시 끊음
                let player_entity = world
                    .query_with::<Player, WeaponInventory>()
                    .next()
                    .map(|(e, _)| e);

                let Some(pe) = player_entity else { return };

                match action {
                    LevelUpInputAction::Choose(idx) => {
                        // 선택된 카드 종류를 값으로 copy — PendingLevelUp borrow 를 여기서 끊음
                        let card = world.resource::<PendingLevelUp>().unwrap().offered[idx];
                        if let Some((new_current, new_threshold)) =
                            Self::choose_card(world, pe, card)
                        {
                            println!(
                                "Resumed (XP={}, next threshold={})",
                                new_current, new_threshold
                            );
                        } else {
                            eprintln!("Ignored invalid LevelUp card: {:?}", card);
                        }
                    }
                    LevelUpInputAction::Reroll => {
                        if !Self::reroll_offer(world, pe) {
                            eprintln!("Ignored reroll: no remaining rerolls or cards");
                        }
                    }
                    LevelUpInputAction::Skip => {
                        if let Some((new_current, new_threshold)) = Self::skip_levelup(world, pe) {
                            println!(
                                "Skipped LevelUp (XP={}, next threshold={})",
                                new_current, new_threshold
                            );
                        } else {
                            eprintln!("Ignored skip: no remaining skips");
                        }
                    }
                    LevelUpInputAction::Banish(idx) => {
                        if !Self::banish_offer(world, pe, idx) {
                            eprintln!("Ignored banish: no remaining banishes or cards");
                        }
                    }
                }
            }

            _ => {}
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LevelUpInputAction {
    Choose(usize),
    Reroll,
    Skip,
    Banish(usize),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::survivor::inventory::{WeaponInventory, WeaponKind, WeaponSlot};
    use crate::survivor::meta::MetaSave;
    use crate::survivor::player::Player;
    use crate::survivor::powerup::PowerUpKind;
    use crate::survivor::world_setup::spawn_player;
    use engine::World;

    fn make_world_with_player() -> (World, engine::Entity) {
        let mut world = World::new();
        spawn_player(&mut world);
        let player_entity = world
            .query::<Player>()
            .next()
            .map(|(e, _)| e)
            .expect("Player 엔티티가 없음");
        // levelup 테스트는 여러 무기 슬롯을 필요로 하므로 전체 로드아웃 사용
        if let Some(inv) = world.get_mut::<WeaponInventory>(player_entity) {
            inv.slots.push(WeaponSlot {
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
            });
            inv.slots.push(WeaponSlot {
                kind: WeaponKind::LightningRing {
                    damage: 30.0,
                    strike_count: 1,
                    hit_radius: 40.0,
                },
                level: 1,
                cooldown: 4.0,
                elapsed: 0.0,
                evolved: false,
            });
        }
        (world, player_entity)
    }

    #[test]
    fn apply_card_magic_wand_damage_increases() {
        let (mut world, player_entity) = make_world_with_player();

        // 초기 MagicWand damage == 8.0 (RON 정의)
        {
            let inv = world.get_mut::<WeaponInventory>(player_entity).unwrap();
            let slot = inv.magic_wand_slot().unwrap();
            assert!(
                matches!(slot.kind, WeaponKind::MagicWand { damage, .. } if (damage - 8.0).abs() < 0.001)
            );
            assert_eq!(slot.level, 1);
        }

        assert!(
            LevelUpSystem::apply_card(&mut world, player_entity, CardKind::MagicWandDamage)
                .is_some()
        );

        let inv = world.get_mut::<WeaponInventory>(player_entity).unwrap();
        let slot = inv.magic_wand_slot().unwrap();
        assert!(
            matches!(slot.kind, WeaponKind::MagicWand { damage, .. } if (damage - 12.0).abs() < 0.001),
            "MagicWandDamage 적용 후 damage 가 12.0 이어야 함"
        );
        assert_eq!(
            slot.level, 2,
            "MagicWandDamage 적용 후 level 이 2 이어야 함"
        );
    }

    #[test]
    fn apply_card_lightning_strike_count_increases() {
        let (mut world, player_entity) = make_world_with_player();

        // 초기 LightningRing strike_count == 1 (RON 정의)
        {
            let inv = world.get_mut::<WeaponInventory>(player_entity).unwrap();
            let slot = inv.lightning_ring_slot().unwrap();
            assert!(
                matches!(slot.kind, WeaponKind::LightningRing { strike_count, .. } if strike_count == 1)
            );
            assert_eq!(slot.level, 1);
        }

        assert!(LevelUpSystem::apply_card(
            &mut world,
            player_entity,
            CardKind::LightningStrikeCount
        )
        .is_some());

        let inv = world.get_mut::<WeaponInventory>(player_entity).unwrap();
        let slot = inv.lightning_ring_slot().unwrap();
        assert!(
            matches!(slot.kind, WeaponKind::LightningRing { strike_count, .. } if strike_count == 2),
            "LightningStrikeCount 적용 후 strike_count 가 2 이어야 함"
        );
        assert_eq!(
            slot.level, 2,
            "LightningStrikeCount 적용 후 level 이 2 이어야 함"
        );
    }

    #[test]
    fn apply_card_rejects_unowned_weapon_without_leveling() {
        let mut world = World::new();
        spawn_player(&mut world);
        let player_entity = world
            .query::<Player>()
            .next()
            .map(|(e, _)| e)
            .expect("Player 엔티티가 없음");

        let before_level = world
            .get::<XpAccumulator>(player_entity)
            .map(|acc| acc.level)
            .unwrap();

        let result =
            LevelUpSystem::apply_card(&mut world, player_entity, CardKind::MagicWandDamage);

        assert!(result.is_none(), "없는 MagicWand 카드는 적용 실패해야 함");
        let after_level = world
            .get::<XpAccumulator>(player_entity)
            .map(|acc| acc.level)
            .unwrap();
        assert_eq!(
            after_level, before_level,
            "없는 무기 카드 선택은 레벨을 소비하면 안 됨"
        );
    }

    #[test]
    fn choose_card_removes_pending_levelup_resource() {
        let (mut world, player_entity) = make_world_with_player();
        world.insert_resource(GameState::Paused);
        world.insert_resource(PendingLevelUp {
            offered: [
                CardKind::WhipDamage,
                CardKind::WhipArea,
                CardKind::WhipCooldown,
            ],
        });

        let result = LevelUpSystem::choose_card(&mut world, player_entity, CardKind::WhipDamage);

        assert!(result.is_some(), "owned Whip card should apply");
        assert!(
            world.resource::<PendingLevelUp>().is_none(),
            "PendingLevelUp should be removed after selection"
        );
        assert_eq!(world.resource::<GameState>(), Some(&GameState::Playing));
    }

    #[test]
    fn levelup_actions_read_counts_from_meta() {
        let mut meta = MetaSave::default();
        meta.powerup_levels
            .insert(PowerUpKind::Reroll.key().to_string(), 2);
        meta.powerup_levels
            .insert(PowerUpKind::Skip.key().to_string(), 1);
        meta.powerup_levels
            .insert(PowerUpKind::Banish.key().to_string(), 3);

        let actions = LevelUpActions::from_meta(Some(&meta));

        assert_eq!(actions.reroll_remaining, 2);
        assert_eq!(actions.skip_remaining, 1);
        assert_eq!(actions.banish_remaining, 3);
        assert!(actions.banished.is_empty());
    }

    #[test]
    fn skip_levelup_consumes_count_and_resumes() {
        let (mut world, player_entity) = make_world_with_player();
        world.insert_resource(GameState::Paused);
        world.insert_resource(LevelUpActions {
            skip_remaining: 1,
            ..LevelUpActions::default()
        });
        world.insert_resource(PendingLevelUp {
            offered: [
                CardKind::WhipDamage,
                CardKind::WhipArea,
                CardKind::WhipCooldown,
            ],
        });

        let result = LevelUpSystem::skip_levelup(&mut world, player_entity);

        assert!(result.is_some());
        assert_eq!(
            world
                .resource::<LevelUpActions>()
                .map(|actions| actions.skip_remaining),
            Some(0)
        );
        assert!(world.resource::<PendingLevelUp>().is_none());
        assert_eq!(world.resource::<GameState>(), Some(&GameState::Playing));
    }

    #[test]
    fn reroll_offer_consumes_count_and_keeps_pending() {
        let (mut world, player_entity) = make_world_with_player();
        world.insert_resource(LevelUpActions {
            reroll_remaining: 1,
            ..LevelUpActions::default()
        });
        world.insert_resource(PendingLevelUp {
            offered: [
                CardKind::WhipDamage,
                CardKind::WhipArea,
                CardKind::WhipCooldown,
            ],
        });

        assert!(LevelUpSystem::reroll_offer(&mut world, player_entity));

        assert_eq!(
            world
                .resource::<LevelUpActions>()
                .map(|actions| actions.reroll_remaining),
            Some(0)
        );
        assert!(world.resource::<PendingLevelUp>().is_some());
    }

    #[test]
    fn banish_offer_consumes_count_and_excludes_card() {
        let (mut world, player_entity) = make_world_with_player();
        world.insert_resource(LevelUpActions {
            banish_remaining: 1,
            ..LevelUpActions::default()
        });
        world.insert_resource(PendingLevelUp {
            offered: [
                CardKind::WhipDamage,
                CardKind::WhipArea,
                CardKind::WhipCooldown,
            ],
        });

        assert!(LevelUpSystem::banish_offer(&mut world, player_entity, 0));

        let actions = world.resource::<LevelUpActions>().unwrap();
        assert_eq!(actions.banish_remaining, 0);
        assert!(actions.banished.contains(&CardKind::WhipDamage));
        let pending = world.resource::<PendingLevelUp>().unwrap();
        assert!(!pending.offered.contains(&CardKind::WhipDamage));
    }
}
