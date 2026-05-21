//! Vampire Survivors 클론 — Phase 3-A 진입점.
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
//!            KingBible 이 8초마다 책 2권을 스폰 (반경 60px 회전, lifetime 5s).
//!            OrbitingBook 이 매 tick(0.3s) 마다 반경 16px 안 적에게 6 데미지.
//!            LightningRing 이 4초마다 랜덤 zombie 1마리에 즉시 area damage (반경 40px, 30 데미지).
//!            LightningFlash 가 0.15초짜리 노란 flash 시각 효과.
//!            ProjectileSystem 이 투사체 이동/충돌/데미지/despawn 처리.
//!            적 사망 시 XpGem(청록색 12×12) 드롭, 자석 반경(80px) 내 젬 끌어당김,
//!            픽업 반경(20px) 내 젬 수집 → XpAccumulator 누적.
//!            XP 임계치(5+5×level) 도달 시 Paused → HUD 에 카드 3장 출력.
//!            1/2/3 키로 Whip 강화 카드 선택 → 효과 적용 → Playing 복귀.
//!            좌상단 HUD: 시간/Lv/XP/HP/Kills. 적 접촉 시 -10HP/초. armor 로 데미지 감쇄.
//!            HP <= 0 → GameOver. R 키 → 재시작.
//!            PlayerStats 16 필드 (might, area, projectile_speed, duration, amount, cooldown,
//!            max_health, recovery, armor, move_speed, magnet, luck, growth, greed, curse, revival).
//! **Phase 3-A 완료** — PlayerStats 16 필드 + StatRecalcSystem(stub) + HealthRegenSystem
//!                     + 모든 무기/시스템(10종)에 stats 적용.

use engine::App;
use game::survivor::{
    setup_survivor_world,
    AxeSystem,
    BossDeathSystem,
    BossPhaseSystem,
    BossSpawnSystem,
    CameraFollowSystem,
    CharacterSelectSystem,
    ChestPickupSystem,
    CrossSystem,
    DamageNumberSystem,
    DeathSystem,
    EnemyAiSystem,
    EnemyContactDamageSystem,
    FireWandSystem,
    GarlicSystem,
    HealthRegenSystem,
    HitFlashSystem,
    HolyWaterPoolSystem,
    HolyWaterSystem,
    HudSystem,
    KingBibleSystem,
    KnifeSystem,
    LevelUpSystem,
    LightningFlashSystem,
    LightningRingSystem,
    MagicWandSystem,
    MagnetSystem,
    ModeTransitionSystem,
    OrbitingBookSystem,
    PickupSystem,
    PlayerMovementSystem,
    ProjectileSystem,
    RestartSystem,
    ShopInputSystem,
    SpawnDirectorSystem,
    StageSelectSystem,
    StatRecalcSystem,
    WhipSystem,
};

fn main() {
    env_logger::init();

    let mut app = App::new();

    // 시스템 등록 순서:
    // StatRecalc (패시브 합산 → PlayerStats 갱신) — 첫 시스템
    // → LevelUp (상태 전환·키 입력) → 플레이어 이동 → 적 추격 → 스폰/정리
    // → SpawnDirector(시간축 wave 스폰+despawn) → 접촉 데미지 → HP 회복(HealthRegen) → 사망 → 재시작 → 카메라 follow
    // → 무기: Whip → MagicWand → Knife → Axe → Cross → FireWand
    //       → Garlic(오라) → HolyWater(풀 스폰) → HolyWaterPool(풀 tick)
    //       → KingBible(책 스폰) → OrbitingBook(책 회전+tick) → LightningRing(번개) → LightningFlash(flash lifetime)
    // → ProjectileSystem(이동·충돌·데미지)
    // → 자석(픽업/끌어당김) → 히트플래시 → HUD (TextQueue push — 마지막)
    app.add_system(ModeTransitionSystem);   // Phase 8-A: 최상단 — SurvivorMode 전환 + GameState 동기화
    app.add_system(ShopInputSystem);        // Phase 8-B: Shop 입력 처리
    app.add_system(CharacterSelectSystem);  // Phase 9: 캐릭터 선택 입력 처리
    app.add_system(StageSelectSystem);      // Phase 10: 스테이지 선택 입력 처리
    app.add_system(StatRecalcSystem);      // Phase 3-A: 패시브 합산해 PlayerStats 갱신 (현재 no-op)
    app.add_system(LevelUpSystem);
    app.add_system(PlayerMovementSystem);
    app.add_system(EnemyAiSystem);
    app.add_system(SpawnDirectorSystem::default());
    app.add_system(BossSpawnSystem);       // Phase 5: 시간축 보스 등장 트리거
    app.add_system(BossPhaseSystem);       // Phase 5: HP 비율 → phase 전환
    app.add_system(BossDeathSystem);       // Phase 5: 보스 사망 + StageClear
    app.add_system(EnemyContactDamageSystem::default());
    app.add_system(HealthRegenSystem);             // Phase 3-A: recovery stat 적용 (0.0 default → no-op)
    app.add_system(DeathSystem);
    app.add_system(RestartSystem);
    app.add_system(CameraFollowSystem::default());
    app.add_system(WhipSystem::default());
    app.add_system(MagicWandSystem::default());
    app.add_system(KnifeSystem::default());
    app.add_system(AxeSystem::default());
    app.add_system(CrossSystem::default());
    app.add_system(FireWandSystem::default());
    app.add_system(GarlicSystem::default());
    app.add_system(HolyWaterSystem);
    app.add_system(HolyWaterPoolSystem::default());
    app.add_system(KingBibleSystem);                  // Phase 2-F 신규 (책 스폰)
    app.add_system(OrbitingBookSystem::default());    // Phase 2-F 신규 (책 회전 + tick)
    app.add_system(LightningRingSystem::default());   // Phase 2-F 신규 (랜덤 적 즉시 area damage)
    app.add_system(LightningFlashSystem);             // Phase 2-F 신규 (flash lifetime)
    app.add_system(ProjectileSystem::default());
    app.add_system(MagnetSystem::default());
    app.add_system(ChestPickupSystem::default());     // Phase 6: Chest 픽업 + 진화 시도
    app.add_system(PickupSystem::default());          // Phase 7: 픽업 5종 자동 픽업
    app.add_system(HitFlashSystem);
    app.add_system(DamageNumberSystem);               // Phase 11: 데미지 숫자 age/이동/despawn
    app.add_system(HudSystem);                        // 마지막 — TextQueue push (데미지 숫자 포함)

    // 초기 엔티티 + 리소스 설정 (GameStats 포함)
    setup_survivor_world(&mut app.world);

    // 이벤트 루프 진입 (ESC 로 종료)
    app.run();
}
