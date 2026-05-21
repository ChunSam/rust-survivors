use engine::{Sprite, Transform, World};
use glam::Vec2;
use super::boss::{BossSpawnQueue, CameraShake, StageProgress};
use super::director::SpawnDirector;
use super::health::Health;
use super::hud::GameStats;
use super::inventory::WeaponInventory;
use super::meta::{MetaSave, SurvivorMode};
use super::passive::PassiveInventory;
use super::pickup::GoldWallet;
use super::player::{Player, PlayerStats, Velocity};
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
    world.add_component(e, WeaponInventory::with_starter_loadout());
    world.add_component(e, PassiveInventory::default());
    world.add_component(e, XpAccumulator::default());
    // SpawnDirector 가 없으면 삽입 (최초 init 용 — restart 시엔 이미 리셋됨)
    if world.resource::<SpawnDirector>().is_none() {
        world.insert_resource(SpawnDirector::default());
    }
}

/// 씬 초기화 진입점. survivor.rs 와 RestartSystem 모두 이 함수를 호출한다.
///
/// - `spawn_player` 로 플레이어 엔티티 스폰 + SpawnDirector 리소스 삽입.
/// - `GameStats` 리소스가 없으면 default 로 삽입 (재시작 시에는 이미 reset 됐으므로 덮어쓰지 않음).
/// - Phase 5: BossSpawnQueue / CameraShake / StageProgress 리소스도 최초 init 시 삽입.
pub fn setup_survivor_world(world: &mut World) {
    spawn_player(world);
    // GameStats 가 없을 때만 삽입 (최초 init 용)
    // 재시작(restart_world) 에서는 먼저 리셋 후 spawn_player 를 호출하므로 중복 없음.
    if world.resource::<GameStats>().is_none() {
        world.insert_resource(GameStats::default());
    }
    if world.resource::<BossSpawnQueue>().is_none() {
        world.insert_resource(BossSpawnQueue::default());
    }
    if world.resource::<CameraShake>().is_none() {
        world.insert_resource(CameraShake::default());
    }
    if world.resource::<StageProgress>().is_none() {
        world.insert_resource(StageProgress::default());
    }
    // Phase 7: GoldWallet — 사망해도 골드 유지 (메타 진행성). 최초 init 시만 삽입.
    if world.resource::<GoldWallet>().is_none() {
        world.insert_resource(GoldWallet::default());
    }
    // Phase 8-A: 영구 메타 데이터 — 재시작해도 유지 (if_none 으로 보호).
    if world.resource::<MetaSave>().is_none() {
        world.insert_resource(MetaSave::load_or_default());
    }
    // Phase 8-A: SurvivorMode — 최초 시작 시 Title.
    if world.resource::<SurvivorMode>().is_none() {
        world.insert_resource(SurvivorMode::default());
    }
}
