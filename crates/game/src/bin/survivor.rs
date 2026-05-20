//! Vampire Survivors 클론 — Phase 1-F 진입점.
//!
//! 현재 상태: 노란 플레이어가 WASD/방향키로 이동하고, 카메라가 따라감.
//!            초록 좀비가 카메라 외곽에서 스폰되어 플레이어를 추격함.
//!            Whip 이 1초마다 좌/우 AABB 로 적을 타격하고 HP 0 이면 despawn.
//!            적 사망 시 XpGem(청록색 12×12) 드롭, 자석 반경(80px) 내 젬 끌어당김,
//!            픽업 반경(20px) 내 젬 수집 → XpAccumulator 누적.
//!            XP 임계치(5+5×level) 도달 시 Paused → HUD 에 카드 3장 출력.
//!            1/2/3 키로 Whip 강화 카드 선택 → 효과 적용 → Playing 복귀.
//!            좌상단 HUD: 시간/Lv/XP/HP/Kills. 적 접촉 시 -10HP/초.
//!            HP <= 0 → GameOver. R 키 → 재시작.
//! **Phase 1 Vertical Slice MVP 완료** — 한 루프(이동→자동공격→XP→레벨업→사망/재시작)가 닫힘.

use engine::App;
use game::survivor::{
    setup_survivor_world,
    CameraFollowSystem,
    DeathSystem,
    EnemyAiSystem,
    EnemyContactDamageSystem,
    EnemySpawnSystem,
    HitFlashSystem,
    HudSystem,
    LevelUpSystem,
    MagnetSystem,
    PlayerMovementSystem,
    RestartSystem,
    WhipSystem,
};

fn main() {
    env_logger::init();

    let mut app = App::new();

    // 시스템 등록 순서:
    // LevelUp (상태 전환·키 입력) → 플레이어 이동 → 적 추격 → 스폰/정리
    // → 접촉 데미지 → 사망 → 재시작 → 카메라 follow → 무기(XpGem 스폰)
    // → 자석(픽업/끌어당김) → 히트플래시 → HUD (TextQueue push — 마지막)
    app.add_system(LevelUpSystem);
    app.add_system(PlayerMovementSystem);
    app.add_system(EnemyAiSystem);
    app.add_system(EnemySpawnSystem::default());
    app.add_system(EnemyContactDamageSystem::default()); // 신규
    app.add_system(DeathSystem);                         // 신규
    app.add_system(RestartSystem);                       // 신규
    app.add_system(CameraFollowSystem::default());
    app.add_system(WhipSystem::default());
    app.add_system(MagnetSystem::default());
    app.add_system(HitFlashSystem);
    app.add_system(HudSystem);                           // 마지막 — TextQueue push

    // 초기 엔티티 + 리소스 설정 (GameStats 포함)
    setup_survivor_world(&mut app.world);

    // 이벤트 루프 진입 (ESC 로 종료)
    app.run();
}
