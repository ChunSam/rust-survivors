//! Vampire Survivors 클론 — Phase 1-E 진입점.
//!
//! 현재 상태: 노란 플레이어가 WASD/방향키로 이동하고, 카메라가 따라감.
//!            초록 좀비가 카메라 외곽에서 스폰되어 플레이어를 추격함.
//!            Whip 이 1초마다 좌/우 AABB 로 적을 타격하고 HP 0 이면 despawn.
//!            적 사망 시 XpGem(청록색 12×12) 드롭, 자석 반경(80px) 내 젬 끌어당김,
//!            픽업 반경(20px) 내 젬 수집 → XpAccumulator 누적.
//!            XP 임계치(5+5×level) 도달 시 Paused → 콘솔에 카드 3장 출력.
//!            1/2/3 키로 Whip 강화 카드 선택 → 효과 적용 → Playing 복귀.
//! 다음 단계 (Phase 1-F): HUD + 사망/GameOver/R 재시작

use engine::App;
use game::survivor::{
    spawn_player,
    CameraFollowSystem,
    EnemyAiSystem,
    EnemySpawnSystem,
    HitFlashSystem,
    LevelUpSystem,
    MagnetSystem,
    PlayerMovementSystem,
    WhipSystem,
};

fn main() {
    env_logger::init();

    let mut app = App::new();

    // 시스템 등록 순서:
    // LevelUp (상태 전환·키 입력) → 플레이어 이동 → 적 추격 → 스폰/정리
    // → 카메라 follow → 무기(XpGem 스폰) → 자석(픽업/끌어당김) → 히트플래시
    app.add_system(LevelUpSystem);
    app.add_system(PlayerMovementSystem);
    app.add_system(EnemyAiSystem);
    app.add_system(EnemySpawnSystem::default());
    app.add_system(CameraFollowSystem::default());
    app.add_system(WhipSystem::default());
    app.add_system(MagnetSystem::default());
    app.add_system(HitFlashSystem);

    // 초기 엔티티 + SpawnTimer 리소스
    spawn_player(&mut app.world);

    // 이벤트 루프 진입 (ESC 로 종료)
    app.run();
}
