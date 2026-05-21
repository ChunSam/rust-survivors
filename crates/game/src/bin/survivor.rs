//! Vampire Survivors 클론 — Phase 2-E 진입점.
//!
//! 현재 상태: 노란 플레이어가 WASD/방향키로 이동하고, 카메라가 따라감.
//!            초록 좀비가 카메라 외곽에서 스폰되어 플레이어를 추격함.
//!            Whip 이 1초마다 좌/우 AABB 로 적을 타격하고 HP 0 이면 despawn.
//!            MagicWand 가 1.2초마다 가장 가까운 적 1마리에게 직선 투사체 발사.
//!            Knife 가 0.8초마다 가장 가까운 적 방향으로 직선 투사체 발사.
//!            Axe 가 1.5초마다 위로 던지는 포물선(Arc) 투사체 발사.
//!            Cross 가 1.8초마다 가장 가까운 적 방향으로 부메랑(Boomerang) 투사체 발사.
//!            FireWand 가 2.5초마다 랜덤 적에게 고데미지 단발 투사체 발사.
//!            Garlic 이 0.5초마다 플레이어 반경 70px 안 모든 적에게 4 데미지 (오라).
//!            HolyWater 가 3초마다 플레이어 주변 랜덤 위치에 지속 풀 드롭.
//!            HolyWaterPool 이 tick_cooldown(0.5s) 마다 반경 50px 안 적에게 5 데미지.
//!            ProjectileSystem 이 투사체 이동/충돌/데미지/despawn 처리.
//!            적 사망 시 XpGem(청록색 12×12) 드롭, 자석 반경(80px) 내 젬 끌어당김,
//!            픽업 반경(20px) 내 젬 수집 → XpAccumulator 누적.
//!            XP 임계치(5+5×level) 도달 시 Paused → HUD 에 카드 3장 출력.
//!            1/2/3 키로 Whip 강화 카드 선택 → 효과 적용 → Playing 복귀.
//!            좌상단 HUD: 시간/Lv/XP/HP/Kills. 적 접촉 시 -10HP/초.
//!            HP <= 0 → GameOver. R 키 → 재시작.
//! **Phase 2-E 완료** — Garlic(오라) + HolyWater(필드 풀).

use engine::App;
use game::survivor::{
    setup_survivor_world,
    AxeSystem,
    CameraFollowSystem,
    CrossSystem,
    DeathSystem,
    EnemyAiSystem,
    EnemyContactDamageSystem,
    EnemySpawnSystem,
    FireWandSystem,
    GarlicSystem,
    HitFlashSystem,
    HolyWaterPoolSystem,
    HolyWaterSystem,
    HudSystem,
    KnifeSystem,
    LevelUpSystem,
    MagicWandSystem,
    MagnetSystem,
    PlayerMovementSystem,
    ProjectileSystem,
    RestartSystem,
    WhipSystem,
};

fn main() {
    env_logger::init();

    let mut app = App::new();

    // 시스템 등록 순서:
    // LevelUp (상태 전환·키 입력) → 플레이어 이동 → 적 추격 → 스폰/정리
    // → 접촉 데미지 → 사망 → 재시작 → 카메라 follow
    // → 무기: Whip → MagicWand(발사) → Knife(발사) → Axe(발사) → Cross(발사) → FireWand(발사)
    //       → Garlic(오라) → HolyWater(풀 스폰) → HolyWaterPool(풀 tick+lifetime)
    // → ProjectileSystem(이동·충돌·데미지)
    // → 자석(픽업/끌어당김) → 히트플래시 → HUD (TextQueue push — 마지막)
    app.add_system(LevelUpSystem);
    app.add_system(PlayerMovementSystem);
    app.add_system(EnemyAiSystem);
    app.add_system(EnemySpawnSystem::default());
    app.add_system(EnemyContactDamageSystem::default());
    app.add_system(DeathSystem);
    app.add_system(RestartSystem);
    app.add_system(CameraFollowSystem::default());
    app.add_system(WhipSystem::default());
    app.add_system(MagicWandSystem::default());
    app.add_system(KnifeSystem::default());
    app.add_system(AxeSystem::default());
    app.add_system(CrossSystem::default());
    app.add_system(FireWandSystem::default());
    app.add_system(GarlicSystem::default());        // Phase 2-E 신규 (오라)
    app.add_system(HolyWaterSystem);               // Phase 2-E 신규 (풀 스폰)
    app.add_system(HolyWaterPoolSystem::default()); // Phase 2-E 신규 (풀 tick + lifetime)
    app.add_system(ProjectileSystem::default());
    app.add_system(MagnetSystem::default());
    app.add_system(HitFlashSystem);
    app.add_system(HudSystem);                      // 마지막 — TextQueue push

    // 초기 엔티티 + 리소스 설정 (GameStats 포함)
    setup_survivor_world(&mut app.world);

    // 이벤트 루프 진입 (ESC 로 종료)
    app.run();
}
