//! Vampire Survivors 클론 — Phase 1-C 진입점.
//!
//! 현재 상태: 노란 플레이어가 WASD/방향키로 이동하고, 카메라가 따라감.
//!            초록 좀비가 카메라 외곽에서 스폰되어 플레이어를 추격함.
//!            Whip 이 1초마다 좌/우 AABB 로 적을 타격하고 HP 0 이면 despawn.
//! 다음 단계 (Phase 1-D): XpGem 드롭 + 자석 + 레벨업 + 플레이어 사망

use engine::App;
use game::survivor::{
    spawn_player,
    CameraFollowSystem,
    EnemyAiSystem,
    EnemySpawnSystem,
    HitFlashSystem,
    PlayerMovementSystem,
    WhipSystem,
};

fn main() {
    env_logger::init();

    let mut app = App::new();

    // 시스템 등록 순서: 입력 → 적 추격 → 스폰/정리 → 카메라 follow → 무기
    app.add_system(PlayerMovementSystem);
    app.add_system(EnemyAiSystem);
    app.add_system(EnemySpawnSystem::default());
    app.add_system(CameraFollowSystem::default());
    app.add_system(WhipSystem::default());
    app.add_system(HitFlashSystem);

    // 초기 엔티티 + SpawnTimer 리소스
    spawn_player(&mut app.world);

    // 이벤트 루프 진입 (ESC 로 종료)
    app.run();
}
