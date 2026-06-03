/// Phase 7: 픽업 5종 (Coin / Chicken / Vacuum / Bomb / Rosary).
///
/// 적 사망 시 낮은 확률로 드롭 (Coin/Chicken/Rosary),
/// 보스 사망 시 항상 드롭 (Vacuum/Bomb).
/// 픽업 시 즉시 효과 발동.
use engine::{Entity, GameState, System, Transform, World};
use glam::Vec2;
use rand::Rng;

use super::boss::CameraShake;
use super::combat::apply_damage_to_targets;
use super::enemy::Enemy;
use super::health::Health;
use super::player::{Player, PlayerStats};
use super::sfx::{SfxEvent, SfxQueue};
use super::sprites::{add_sprite, SurvivorSprite};
use super::xp::XpGem;

/// 픽업 종류 — 적/보스 사망 시 드롭.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PickupKind {
    Coin,    // 적 사망 시 ~1% 드롭. 픽업 → GoldWallet.current += 1
    Chicken, // 적 사망 시 ~0.5% 드롭. 픽업 → Player Health +20 (max clamp)
    Vacuum,  // 보스 사망 시 100% 드롭. 픽업 → 화면 내 모든 XpGem 을 Player 위치로 즉시 이동
    Bomb,    // 보스 사망 시 100% 드롭. 픽업 → 화면 내 모든 적에 100 데미지 (광역)
    Rosary,  // 적 사망 시 ~0.1% 드롭. 픽업 → 화면 내 모든 적 즉사
}

impl PickupKind {
    pub fn color(self) -> [f32; 3] {
        match self {
            PickupKind::Coin => [1.0, 0.85, 0.2],
            PickupKind::Chicken => [0.95, 0.6, 0.4],
            PickupKind::Vacuum => [0.5, 0.9, 1.0],
            PickupKind::Bomb => [0.3, 0.3, 0.3],
            PickupKind::Rosary => [1.0, 1.0, 1.0],
        }
    }

    pub fn scale(self) -> f32 {
        match self {
            PickupKind::Coin => 14.0,
            PickupKind::Chicken => 18.0,
            PickupKind::Vacuum => 20.0,
            PickupKind::Bomb => 22.0,
            PickupKind::Rosary => 16.0,
        }
    }
}

/// 픽업 컴포넌트.
#[derive(Debug, Clone, Copy)]
pub struct Pickup {
    pub kind: PickupKind,
}

/// 메타 골드 지갑 (Phase 8 메타 진행에서 영구화 예정).
#[derive(Debug, Default, Clone)]
pub struct GoldWallet {
    pub current: u32,
}

/// 픽업 자동 픽업 시스템.
pub struct PickupSystem {
    pub pickup_radius: f32,
    to_pickup: Vec<(Entity, PickupKind)>,
}

impl Default for PickupSystem {
    fn default() -> Self {
        Self {
            pickup_radius: 35.0,
            to_pickup: Vec::new(),
        }
    }
}

impl System for PickupSystem {
    fn run(&mut self, world: &mut World, _dt: f32) {
        if !matches!(world.resource::<GameState>(), Some(GameState::Playing)) {
            return;
        }

        let Some((player_entity, player_pos)) = world
            .query2::<Player, Transform>()
            .next()
            .map(|(e, _, t)| (e, t.position))
        else {
            return;
        };

        let pickup_radius_sq = self.pickup_radius * self.pickup_radius;
        self.to_pickup.clear();
        self.to_pickup.extend(
            world
                .query2::<Pickup, Transform>()
                .filter(|(_, _, t)| (t.position - player_pos).length_squared() <= pickup_radius_sq)
                .map(|(e, p, _)| (e, p.kind)),
        );

        for (entity, kind) in self.to_pickup.drain(..) {
            world.despawn(entity);
            apply_pickup_effect(world, player_entity, player_pos, kind);
        }
    }
}

/// 픽업 효과 적용.
pub fn apply_pickup_effect(world: &mut World, player: Entity, player_pos: Vec2, kind: PickupKind) {
    match kind {
        PickupKind::Coin => {
            let greed = world
                .query2::<Player, PlayerStats>()
                .next()
                .map(|(_, _, stats)| stats.greed)
                .unwrap_or(1.0);
            let value = coin_gold_value(greed);
            if let Some(wallet) = world.resource_mut::<GoldWallet>() {
                wallet.current += value;
                println!("Coin picked: gold {}", wallet.current);
            }
            if let Some(q) = world.resource_mut::<SfxQueue>() {
                q.push(SfxEvent::Pickup);
            }
        }
        PickupKind::Chicken => {
            if let Some(h) = world.get_mut::<Health>(player) {
                let healed = (h.current + 20.0).min(h.max);
                h.current = healed;
                println!("Chicken picked: HP {:.0}/{:.0}", healed, h.max);
            }
            if let Some(q) = world.resource_mut::<SfxQueue>() {
                q.push(SfxEvent::Pickup);
            }
        }
        PickupKind::Vacuum => {
            // 모든 XpGem 의 Transform.position 을 player 위치로 즉시 이동
            // (MagnetSystem 이 다음 프레임에 픽업 처리)
            let gems: Vec<Entity> = world.query::<XpGem>().map(|(e, _)| e).collect();
            let count = gems.len();
            for e in &gems {
                if let Some(t) = world.get_mut::<Transform>(*e) {
                    t.position = player_pos;
                }
            }
            println!("Vacuum picked: {} gems pulled", count);
        }
        PickupKind::Bomb => {
            // 화면 내 모든 적에 100 데미지
            let enemies: Vec<Entity> = world.query::<Enemy>().map(|(e, _)| e).collect();
            let killed = apply_damage_to_targets(world, enemies, 100.0);
            if let Some(s) = world.resource_mut::<CameraShake>() {
                s.trigger(0.40, 14.0);
            }
            if let Some(q) = world.resource_mut::<SfxQueue>() {
                q.push(SfxEvent::Bomb);
            }
            println!("Bomb picked: {} enemies killed", killed);
        }
        PickupKind::Rosary => {
            // 화면 내 모든 적 즉사 — 매우 큰 데미지
            let enemies: Vec<Entity> = world.query::<Enemy>().map(|(e, _)| e).collect();
            let killed = apply_damage_to_targets(world, enemies, 9999.0);
            if let Some(s) = world.resource_mut::<CameraShake>() {
                s.trigger(0.35, 10.0);
            }
            if let Some(q) = world.resource_mut::<SfxQueue>() {
                q.push(SfxEvent::Bomb);
            }
            println!("Rosary picked: {} enemies cleared", killed);
        }
    }
}

/// 픽업 엔티티 스폰 헬퍼.
pub fn spawn_pickup(world: &mut World, pos: Vec2, kind: PickupKind) {
    let scale = kind.scale();
    let e = world.spawn();
    world.add_component(
        e,
        Transform {
            position: pos,
            scale: Vec2::new(scale, scale),
            rotation: 0.0,
            z: 0.35,
        },
    );
    let sprite = match kind {
        PickupKind::Coin => SurvivorSprite::Coin,
        PickupKind::Chicken => SurvivorSprite::Chicken,
        PickupKind::Vacuum => SurvivorSprite::Vacuum,
        PickupKind::Bomb => SurvivorSprite::Bomb,
        PickupKind::Rosary => SurvivorSprite::Rosary,
    };
    add_sprite(world, e, sprite);
    world.add_component(e, Pickup { kind });
}

/// 적 사망 시 확률 기반 픽업 드롭.
///
/// 드롭 확률:
/// - Rosary: 0.1%
/// - Chicken: 0.5% (누적 0.001 ~ 0.006)
/// - Coin: 1.0% (누적 0.006 ~ 0.016)
pub fn try_drop_normal_pickup(world: &mut World, pos: Vec2) {
    let luck = world
        .query2::<Player, PlayerStats>()
        .next()
        .map(|(_, _, stats)| stats.luck)
        .unwrap_or(1.0);
    let mut rng = rand::thread_rng();
    let roll: f32 = rng.gen_range(0.0..1.0);
    let chances = normal_pickup_thresholds(luck);
    if roll < chances.rosary {
        spawn_pickup(world, pos, PickupKind::Rosary);
    } else if roll < chances.chicken {
        // 0.001 ~ 0.006 → Chicken
        spawn_pickup(world, pos, PickupKind::Chicken);
    } else if roll < chances.coin {
        // 0.006 ~ 0.016 → Coin
        spawn_pickup(world, pos, PickupKind::Coin);
    }
    // 그 외 픽업 없음 — 일반 XpGem 만
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NormalPickupThresholds {
    pub rosary: f32,
    pub chicken: f32,
    pub coin: f32,
}

pub fn normal_pickup_thresholds(luck: f32) -> NormalPickupThresholds {
    let luck = luck.clamp(0.0, 5.0);
    NormalPickupThresholds {
        rosary: 0.001 * luck,
        chicken: 0.006 * luck,
        coin: 0.016 * luck,
    }
}

pub fn coin_gold_value(greed: f32) -> u32 {
    greed.max(0.0).round().max(1.0) as u32
}

/// 보스 사망 시 항상 Vacuum + Bomb 드롭.
pub fn drop_boss_pickups(world: &mut World, pos: Vec2) {
    spawn_pickup(world, pos + Vec2::new(-30.0, 0.0), PickupKind::Vacuum);
    spawn_pickup(world, pos + Vec2::new(30.0, 0.0), PickupKind::Bomb);
}

// ─── 테스트 ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::survivor::enemy::EnemyKind;
    use crate::survivor::player::Player;
    use crate::survivor::spawn::spawn_enemy;
    use crate::survivor::world_setup::spawn_player;
    use engine::{GameState, World};

    fn make_playing_world_with_player() -> (World, Entity) {
        let mut world = World::new();
        world.insert_resource(GameState::Playing);
        world.insert_resource(GoldWallet::default());
        spawn_player(&mut world);
        let player_entity = world
            .query::<Player>()
            .next()
            .map(|(e, _)| e)
            .expect("Player 엔티티가 없음");
        (world, player_entity)
    }

    #[test]
    fn chicken_heals_player() {
        let (mut world, player_entity) = make_playing_world_with_player();

        // Player Health 를 50.0 으로 강제 설정
        if let Some(h) = world.get_mut::<Health>(player_entity) {
            h.current = 50.0;
        }

        apply_pickup_effect(&mut world, player_entity, Vec2::ZERO, PickupKind::Chicken);

        let hp = world
            .get_mut::<Health>(player_entity)
            .map(|h| h.current)
            .unwrap_or(0.0);
        assert!(
            (hp - 70.0).abs() < 0.01,
            "Chicken 픽업 후 HP 가 70.0 이어야 함 (실제: {:.1})",
            hp
        );
    }

    #[test]
    fn bomb_kills_all_enemies() {
        let (mut world, player_entity) = make_playing_world_with_player();

        // 일반 좀비 3마리 스폰 (HP 30, Bomb 100 데미지로 즉사)
        spawn_enemy(&mut world, Vec2::new(100.0, 0.0), EnemyKind::Zombie);
        spawn_enemy(&mut world, Vec2::new(200.0, 0.0), EnemyKind::Zombie);
        spawn_enemy(&mut world, Vec2::new(300.0, 0.0), EnemyKind::Zombie);

        assert_eq!(world.query::<Enemy>().count(), 3, "Bomb 전 적 3마리여야 함");

        apply_pickup_effect(&mut world, player_entity, Vec2::ZERO, PickupKind::Bomb);

        assert_eq!(
            world.query::<Enemy>().count(),
            0,
            "Bomb 후 모든 적이 사망해야 함"
        );
    }

    #[test]
    fn coin_increments_gold_wallet() {
        let (mut world, player_entity) = make_playing_world_with_player();

        apply_pickup_effect(&mut world, player_entity, Vec2::ZERO, PickupKind::Coin);
        apply_pickup_effect(&mut world, player_entity, Vec2::ZERO, PickupKind::Coin);

        let gold = world
            .resource::<GoldWallet>()
            .map(|w| w.current)
            .unwrap_or(0);
        assert_eq!(
            gold, 2,
            "Coin 2회 픽업 후 gold == 2 이어야 함 (실제: {})",
            gold
        );
    }

    #[test]
    fn coin_gold_value_rounds_greed_with_minimum_one() {
        assert_eq!(coin_gold_value(0.0), 1);
        assert_eq!(coin_gold_value(1.49), 1);
        assert_eq!(coin_gold_value(1.50), 2);
        assert_eq!(coin_gold_value(2.49), 2);
    }

    #[test]
    fn normal_pickup_thresholds_scale_and_clamp_luck() {
        assert_eq!(
            normal_pickup_thresholds(1.0),
            NormalPickupThresholds {
                rosary: 0.001,
                chicken: 0.006,
                coin: 0.016,
            }
        );
        assert_eq!(
            normal_pickup_thresholds(2.0),
            NormalPickupThresholds {
                rosary: 0.002,
                chicken: 0.012,
                coin: 0.032,
            }
        );
        assert!((normal_pickup_thresholds(10.0).coin - 0.080).abs() < f32::EPSILON);
    }
}
