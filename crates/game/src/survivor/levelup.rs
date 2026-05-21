use engine::{System, World};
use engine::components::GameState;
use engine::input::InputState;
use rand::seq::SliceRandom;
use winit::keyboard::KeyCode;
use super::xp::XpAccumulator;
use super::inventory::{WeaponInventory, WeaponKind};
use super::player::Player;

/// 카드 강화 종류 — 10 무기 × 2~3 카드씩 총 28 variant.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CardKind {
    // ── Whip (3)
    WhipDamage,      // damage += 5
    WhipArea,        // area_width += 20, area_height += 10
    WhipCooldown,    // cooldown *= 0.85

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
    CrossDamage,    // damage += 6
    CrossReturnAt,  // return_at += 0.1 (부메랑 더 늦게 반전 → 더 멀리 감)
    CrossCooldown,  // cooldown *= 0.85

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
];

impl CardKind {
    pub fn label(self) -> &'static str {
        match self {
            // Whip
            CardKind::WhipDamage      => "Whip: DMG +5",
            CardKind::WhipArea        => "Whip: AREA UP",
            CardKind::WhipCooldown    => "Whip: CD -15%",
            // MagicWand
            CardKind::MagicWandDamage   => "Magic Wand: DMG +4",
            CardKind::MagicWandSpeed    => "Magic Wand: SPD +60",
            CardKind::MagicWandCooldown => "Magic Wand: CD -15%",
            // Knife
            CardKind::KnifeDamage   => "Knife: DMG +4",
            CardKind::KnifeAmount   => "Knife: +1 Knife",
            CardKind::KnifeCooldown => "Knife: CD -15%",
            // Axe
            CardKind::AxeDamage   => "Axe: DMG +5",
            CardKind::AxePierce   => "Axe: PIERCE +1",
            CardKind::AxeCooldown => "Axe: CD -15%",
            // Cross
            CardKind::CrossDamage   => "Cross: DMG +6",
            CardKind::CrossReturnAt => "Cross: RANGE UP",
            CardKind::CrossCooldown => "Cross: CD -15%",
            // FireWand
            CardKind::FireWandDamage   => "Fire Wand: DMG +8",
            CardKind::FireWandCooldown => "Fire Wand: CD -15%",
            // Garlic
            CardKind::GarlicDamage   => "Garlic: DMG +2",
            CardKind::GarlicRadius   => "Garlic: AREA +15",
            CardKind::GarlicCooldown => "Garlic: CD -15%",
            // HolyWater
            CardKind::HolyWaterDamage    => "Holy Water: DMG +3",
            CardKind::HolyWaterDropCount => "Holy Water: +1 Pool",
            CardKind::HolyWaterCooldown  => "Holy Water: CD -15%",
            // KingBible
            CardKind::KingBibleDamage    => "King Bible: DMG +3",
            CardKind::KingBibleBookCount => "King Bible: +1 Book",
            CardKind::KingBibleCooldown  => "King Bible: CD -15%",
            // LightningRing
            CardKind::LightningDamage      => "Lightning: DMG +8",
            CardKind::LightningStrikeCount => "Lightning: +1 Strike",
            CardKind::LightningCooldown    => "Lightning: CD -15%",
        }
    }
}

/// LevelUp 진행 중에만 World 에 삽입되는 리소스.
///
/// `consumed = false` → 카드 선택 대기 중.
/// `consumed = true`  → 선택 완료 (World::remove_resource 가 없어서 sentinel 처리).
pub struct PendingLevelUp {
    pub offered:  [CardKind; 3],
    pub consumed: bool,
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

    /// 선택한 카드 효과를 Player 엔티티의 WeaponInventory 에 적용하고
    /// XpAccumulator 를 갱신한다.
    ///
    /// 키 입력 없이 직접 호출 가능하므로 단위 테스트에서 재사용.
    /// 반환: (새 XP current, 새 threshold) — println 메시지 구성용
    pub fn apply_card(world: &mut World, player_entity: engine::Entity, card: CardKind) -> (u32, u32) {
        if let Some(inv) = world.get_mut::<WeaponInventory>(player_entity) {
            match card {
                // ── Whip ────────────────────────────────────────────────────
                CardKind::WhipDamage => {
                    if let Some(slot) = inv.whip_slot_mut() {
                        if let WeaponKind::Whip { damage, .. } = &mut slot.kind {
                            *damage += 5.0;
                        }
                        slot.level += 1;
                    }
                }
                CardKind::WhipArea => {
                    if let Some(slot) = inv.whip_slot_mut() {
                        if let WeaponKind::Whip { area_width, area_height, .. } = &mut slot.kind {
                            *area_width  += 20.0;
                            *area_height += 10.0;
                        }
                        slot.level += 1;
                    }
                }
                CardKind::WhipCooldown => {
                    if let Some(slot) = inv.whip_slot_mut() {
                        slot.cooldown *= 0.85;
                        slot.level += 1;
                    }
                }

                // ── MagicWand ────────────────────────────────────────────────
                CardKind::MagicWandDamage => {
                    if let Some(slot) = inv.magic_wand_slot_mut() {
                        if let WeaponKind::MagicWand { damage, .. } = &mut slot.kind {
                            *damage += 4.0;
                        }
                        slot.level += 1;
                    }
                }
                CardKind::MagicWandSpeed => {
                    if let Some(slot) = inv.magic_wand_slot_mut() {
                        if let WeaponKind::MagicWand { projectile_speed, .. } = &mut slot.kind {
                            *projectile_speed += 60.0;
                        }
                        slot.level += 1;
                    }
                }
                CardKind::MagicWandCooldown => {
                    if let Some(slot) = inv.magic_wand_slot_mut() {
                        slot.cooldown *= 0.85;
                        slot.level += 1;
                    }
                }

                // ── Knife ────────────────────────────────────────────────────
                CardKind::KnifeDamage => {
                    if let Some(slot) = inv.knife_slot_mut() {
                        if let WeaponKind::Knife { damage, .. } = &mut slot.kind {
                            *damage += 4.0;
                        }
                        slot.level += 1;
                    }
                }
                CardKind::KnifeAmount => {
                    if let Some(slot) = inv.knife_slot_mut() {
                        if let WeaponKind::Knife { amount, .. } = &mut slot.kind {
                            *amount += 1;
                        }
                        slot.level += 1;
                    }
                }
                CardKind::KnifeCooldown => {
                    if let Some(slot) = inv.knife_slot_mut() {
                        slot.cooldown *= 0.85;
                        slot.level += 1;
                    }
                }

                // ── Axe ──────────────────────────────────────────────────────
                CardKind::AxeDamage => {
                    if let Some(slot) = inv.axe_slot_mut() {
                        if let WeaponKind::Axe { damage, .. } = &mut slot.kind {
                            *damage += 5.0;
                        }
                        slot.level += 1;
                    }
                }
                CardKind::AxePierce => {
                    if let Some(slot) = inv.axe_slot_mut() {
                        if let WeaponKind::Axe { pierce, .. } = &mut slot.kind {
                            *pierce += 1;
                        }
                        slot.level += 1;
                    }
                }
                CardKind::AxeCooldown => {
                    if let Some(slot) = inv.axe_slot_mut() {
                        slot.cooldown *= 0.85;
                        slot.level += 1;
                    }
                }

                // ── Cross ────────────────────────────────────────────────────
                CardKind::CrossDamage => {
                    if let Some(slot) = inv.cross_slot_mut() {
                        if let WeaponKind::Cross { damage, .. } = &mut slot.kind {
                            *damage += 6.0;
                        }
                        slot.level += 1;
                    }
                }
                CardKind::CrossReturnAt => {
                    if let Some(slot) = inv.cross_slot_mut() {
                        if let WeaponKind::Cross { return_at, .. } = &mut slot.kind {
                            *return_at += 0.1;
                        }
                        slot.level += 1;
                    }
                }
                CardKind::CrossCooldown => {
                    if let Some(slot) = inv.cross_slot_mut() {
                        slot.cooldown *= 0.85;
                        slot.level += 1;
                    }
                }

                // ── FireWand ─────────────────────────────────────────────────
                CardKind::FireWandDamage => {
                    if let Some(slot) = inv.fire_wand_slot_mut() {
                        if let WeaponKind::FireWand { damage, .. } = &mut slot.kind {
                            *damage += 8.0;
                        }
                        slot.level += 1;
                    }
                }
                CardKind::FireWandCooldown => {
                    if let Some(slot) = inv.fire_wand_slot_mut() {
                        slot.cooldown *= 0.85;
                        slot.level += 1;
                    }
                }

                // ── Garlic ───────────────────────────────────────────────────
                CardKind::GarlicDamage => {
                    if let Some(slot) = inv.garlic_slot_mut() {
                        if let WeaponKind::Garlic { damage, .. } = &mut slot.kind {
                            *damage += 2.0;
                        }
                        slot.level += 1;
                    }
                }
                CardKind::GarlicRadius => {
                    if let Some(slot) = inv.garlic_slot_mut() {
                        if let WeaponKind::Garlic { radius, .. } = &mut slot.kind {
                            *radius += 15.0;
                        }
                        slot.level += 1;
                    }
                }
                CardKind::GarlicCooldown => {
                    if let Some(slot) = inv.garlic_slot_mut() {
                        slot.cooldown *= 0.85;
                        slot.level += 1;
                    }
                }

                // ── HolyWater ────────────────────────────────────────────────
                CardKind::HolyWaterDamage => {
                    if let Some(slot) = inv.holy_water_slot_mut() {
                        if let WeaponKind::HolyWater { damage, .. } = &mut slot.kind {
                            *damage += 3.0;
                        }
                        slot.level += 1;
                    }
                }
                CardKind::HolyWaterDropCount => {
                    if let Some(slot) = inv.holy_water_slot_mut() {
                        if let WeaponKind::HolyWater { drop_count, .. } = &mut slot.kind {
                            *drop_count += 1;
                        }
                        slot.level += 1;
                    }
                }
                CardKind::HolyWaterCooldown => {
                    if let Some(slot) = inv.holy_water_slot_mut() {
                        slot.cooldown *= 0.85;
                        slot.level += 1;
                    }
                }

                // ── KingBible ────────────────────────────────────────────────
                CardKind::KingBibleDamage => {
                    if let Some(slot) = inv.king_bible_slot_mut() {
                        if let WeaponKind::KingBible { damage, .. } = &mut slot.kind {
                            *damage += 3.0;
                        }
                        slot.level += 1;
                    }
                }
                CardKind::KingBibleBookCount => {
                    if let Some(slot) = inv.king_bible_slot_mut() {
                        if let WeaponKind::KingBible { book_count, .. } = &mut slot.kind {
                            *book_count += 1;
                        }
                        slot.level += 1;
                    }
                }
                CardKind::KingBibleCooldown => {
                    if let Some(slot) = inv.king_bible_slot_mut() {
                        slot.cooldown *= 0.85;
                        slot.level += 1;
                    }
                }

                // ── LightningRing ────────────────────────────────────────────
                CardKind::LightningDamage => {
                    if let Some(slot) = inv.lightning_ring_slot_mut() {
                        if let WeaponKind::LightningRing { damage, .. } = &mut slot.kind {
                            *damage += 8.0;
                        }
                        slot.level += 1;
                    }
                }
                CardKind::LightningStrikeCount => {
                    if let Some(slot) = inv.lightning_ring_slot_mut() {
                        if let WeaponKind::LightningRing { strike_count, .. } = &mut slot.kind {
                            *strike_count += 1;
                        }
                        slot.level += 1;
                    }
                }
                CardKind::LightningCooldown => {
                    if let Some(slot) = inv.lightning_ring_slot_mut() {
                        slot.cooldown *= 0.85;
                        slot.level += 1;
                    }
                }
            }
        }

        // XpAccumulator: 현재 XP 유지, 레벨 +1, 다음 임계치 갱신
        let (new_current, new_threshold) = if let Some(acc) = world.get_mut::<XpAccumulator>(player_entity) {
            acc.level += 1;
            acc.next_threshold = Self::next_threshold(acc.level);
            (acc.current, acc.next_threshold)
        } else {
            (0, 0)
        };

        (new_current, new_threshold)
    }
}

impl System for LevelUpSystem {
    fn run(&mut self, world: &mut World, _dt: f32) {
        let state = world.resource::<GameState>().cloned();

        // has_pending: PendingLevelUp 리소스가 있고 아직 소비되지 않았는지
        let has_pending = world
            .resource::<PendingLevelUp>()
            .map(|p| !p.consumed)
            .unwrap_or(false);

        match (state, has_pending) {
            // ── 평소: XP 임계치 도달 체크 ──────────────────────────────────────
            (Some(GameState::Playing), false) => {
                // XpAccumulator 캐시 (borrow 를 즉시 끊음)
                let player_data = world
                    .query2::<Player, XpAccumulator>()
                    .next()
                    .map(|(e, _, acc)| (e, acc.current, acc.level, acc.next_threshold));
                let Some((_player_entity, current, level, threshold)) = player_data else { return };

                if current >= threshold {
                    // 레벨업: 카드 풀에서 3장 랜덤 추출
                    let mut rng = rand::thread_rng();
                    let chosen: Vec<CardKind> = ALL_CARDS
                        .choose_multiple(&mut rng, 3)
                        .copied()
                        .collect();
                    let offered = [chosen[0], chosen[1], chosen[2]];

                    world.insert_resource(PendingLevelUp { offered, consumed: false });
                    if let Some(gs) = world.resource_mut::<GameState>() { *gs = GameState::Paused; }

                    println!(
                        "LEVEL UP! (Level {}) — Press: 1={} 2={} 3={}",
                        level + 1,
                        offered[0].label(),
                        offered[1].label(),
                        offered[2].label()
                    );
                }
            }

            // ── 카드 선택 대기: 1/2/3 키 감지 ─────────────────────────────────
            (Some(GameState::Paused), true) => {
                // 키 입력 확인 — InputState borrow 를 블록 안에서 끝냄
                let chosen: Option<usize> = {
                    let Some(input) = world.resource::<InputState>() else { return };
                    if      input.just_pressed(KeyCode::Digit1) { Some(0) }
                    else if input.just_pressed(KeyCode::Digit2) { Some(1) }
                    else if input.just_pressed(KeyCode::Digit3) { Some(2) }
                    else { None }
                };
                let Some(idx) = chosen else { return };

                // 선택된 카드 종류를 값으로 copy — PendingLevelUp borrow 를 여기서 끊음
                let card = world.resource::<PendingLevelUp>().unwrap().offered[idx];

                // Player 엔티티 조회 — borrow 를 즉시 끊음
                let player_entity = world
                    .query2::<Player, WeaponInventory>()
                    .next()
                    .map(|(e, _, _)| e);

                if let Some(pe) = player_entity {
                    let (new_current, new_threshold) = Self::apply_card(world, pe, card);
                    println!("Resumed (XP={}, next threshold={})", new_current, new_threshold);
                }

                // 소비 완료 sentinel 설정 (remove_resource 대체)
                if let Some(p) = world.resource_mut::<PendingLevelUp>() {
                    p.consumed = true;
                }

                // Playing 으로 복귀
                if let Some(gs) = world.resource_mut::<GameState>() { *gs = GameState::Playing; }
            }

            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine::World;
    use crate::survivor::world_setup::spawn_player;
    use crate::survivor::inventory::{WeaponInventory, WeaponKind};
    use crate::survivor::player::Player;

    fn make_world_with_player() -> (World, engine::Entity) {
        let mut world = World::new();
        spawn_player(&mut world);
        // spawn_player 는 Entity 를 반환하지 않으므로 Player 컴포넌트로 쿼리
        let player_entity = world.query::<Player>().next().map(|(e, _)| e)
            .expect("Player 엔티티가 없음");
        (world, player_entity)
    }

    #[test]
    fn apply_card_magic_wand_damage_increases() {
        let (mut world, player_entity) = make_world_with_player();

        // 초기 MagicWand damage == 8.0 (RON 정의)
        {
            let inv = world.get_mut::<WeaponInventory>(player_entity).unwrap();
            let slot = inv.magic_wand_slot().unwrap();
            assert!(matches!(slot.kind, WeaponKind::MagicWand { damage, .. } if (damage - 8.0).abs() < 0.001));
            assert_eq!(slot.level, 1);
        }

        LevelUpSystem::apply_card(&mut world, player_entity, CardKind::MagicWandDamage);

        let inv = world.get_mut::<WeaponInventory>(player_entity).unwrap();
        let slot = inv.magic_wand_slot().unwrap();
        assert!(
            matches!(slot.kind, WeaponKind::MagicWand { damage, .. } if (damage - 12.0).abs() < 0.001),
            "MagicWandDamage 적용 후 damage 가 12.0 이어야 함"
        );
        assert_eq!(slot.level, 2, "MagicWandDamage 적용 후 level 이 2 이어야 함");
    }

    #[test]
    fn apply_card_lightning_strike_count_increases() {
        let (mut world, player_entity) = make_world_with_player();

        // 초기 LightningRing strike_count == 1 (RON 정의)
        {
            let inv = world.get_mut::<WeaponInventory>(player_entity).unwrap();
            let slot = inv.lightning_ring_slot().unwrap();
            assert!(matches!(slot.kind, WeaponKind::LightningRing { strike_count, .. } if strike_count == 1));
            assert_eq!(slot.level, 1);
        }

        LevelUpSystem::apply_card(&mut world, player_entity, CardKind::LightningStrikeCount);

        let inv = world.get_mut::<WeaponInventory>(player_entity).unwrap();
        let slot = inv.lightning_ring_slot().unwrap();
        assert!(
            matches!(slot.kind, WeaponKind::LightningRing { strike_count, .. } if strike_count == 2),
            "LightningStrikeCount 적용 후 strike_count 가 2 이어야 함"
        );
        assert_eq!(slot.level, 2, "LightningStrikeCount 적용 후 level 이 2 이어야 함");
    }
}
