use engine::{Sprite, Transform, World};
use glam::Vec2;
use super::health::Health;
use super::player::{Player, PlayerStats, Velocity};
use super::spawn::SpawnTimer;
use super::weapon::Whip;
use super::xp::XpAccumulator;

/// 플레이어 엔티티를 World 에 스폰. 좌표는 월드 (0,0) — 카메라 follow 가 화면 중앙에 둠.
pub fn spawn_player(world: &mut World) {
    let e = world.spawn();
    world.add_component(e, Transform {
        position: Vec2::new(0.0, 0.0),
        scale:    Vec2::new(48.0, 48.0),
        rotation: 0.0,
        z:        1.0, // 적보다 위에 그려지도록 z=1
    });
    // 임시 노란 사각형 (Phase 1-B 에서 텍스처로 교체)
    world.add_component(e, Sprite::colored(0.95, 0.85, 0.20));
    world.add_component(e, Player);
    world.add_component(e, PlayerStats::default());
    world.add_component(e, Velocity(Vec2::ZERO));
    world.add_component(e, Health::new(100.0));
    world.add_component(e, Whip::default());
    world.add_component(e, XpAccumulator::default());
    world.insert_resource(SpawnTimer::default());
}
