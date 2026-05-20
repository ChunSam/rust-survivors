use engine::{System, World};
use engine::components::GameState;
use engine::input::InputState;
use winit::keyboard::KeyCode;
use super::xp::XpAccumulator;
use super::weapon::Whip;
use super::player::Player;

/// 카드 강화 종류
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CardKind {
    WhipDamage,    // damage += 5
    WhipArea,      // area_width += 20, area_height += 10
    WhipCooldown,  // cooldown *= 0.85
}

impl CardKind {
    pub fn label(self) -> &'static str {
        match self {
            CardKind::WhipDamage   => "1=DMG+5",
            CardKind::WhipArea     => "2=AREA UP",
            CardKind::WhipCooldown => "3=CD-15%",
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

    /// 선택한 카드 효과를 Player 엔티티의 Whip 에 적용하고 XpAccumulator 를 갱신한다.
    ///
    /// 키 입력 없이 직접 호출 가능하므로 단위 테스트에서 재사용.
    /// 반환: (새 XP current, 새 threshold) — println 메시지 구성용
    pub fn apply_card(world: &mut World, player_entity: engine::Entity, card: CardKind) -> (u32, u32) {
        if let Some(whip) = world.get_mut::<Whip>(player_entity) {
            match card {
                CardKind::WhipDamage   => whip.damage += 5.0,
                CardKind::WhipArea     => { whip.area_width += 20.0; whip.area_height += 10.0; }
                CardKind::WhipCooldown => whip.cooldown *= 0.85,
            }
            println!(
                "Whip upgraded: damage={:.1} area={:.0}×{:.0} cooldown={:.2}",
                whip.damage, whip.area_width, whip.area_height, whip.cooldown
            );
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
                    // 레벨업: PendingLevelUp 삽입 + Paused 전환
                    let offered = [CardKind::WhipDamage, CardKind::WhipArea, CardKind::WhipCooldown];
                    world.insert_resource(PendingLevelUp { offered, consumed: false });
                    if let Some(gs) = world.resource_mut::<GameState>() { *gs = GameState::Paused; }

                    println!(
                        "LEVEL UP! (Level {}) — Press: {}  {}  {}",
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
                    .query2::<Player, Whip>()
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
