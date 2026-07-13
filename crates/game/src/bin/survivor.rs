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
//! **Phase 3-A 이후** — PlayerStats 16 필드 + StatRecalcSystem + HealthRegenSystem
//!                    + 모든 무기/시스템(10종)에 stats 적용.

use engine::{AnimationSystem, App, FontData, WindowConfig};
use game::survivor::{
    set_working_dir_to_asset_root, setup_survivor_world, AchievementSystem, ActorFacingSystem,
    AxeSystem, BackgroundSystem, BgmSystem, BossDeathSystem, BossPhaseSystem, BossSpawnSystem,
    CameraFollowSystem, CharacterSelectSystem, ChestPickupSystem, CrossSystem, DamageNumberSystem,
    DeathSystem, DebugInputSystem, EnemyAiSystem, EnemyContactDamageSystem, FireWandSystem,
    GarlicSystem, HealthRegenSystem, HitFlashSystem, HolyWaterPoolSystem, HolyWaterSystem,
    HudSystem, KingBibleSystem, KnifeSystem, LevelUpSystem, LightningFlashSystem,
    LightningRingSystem, MagicWandSystem, MagnetSystem, MetaSave, ModeTransitionSystem,
    OrbitingBookSystem, ParticleSystem, PickupSystem, PlayerMovementSystem, ProjectileSystem,
    ResolutionPreset, RestartSystem, SfxSystem, ShopInputSystem, SpawnDirectorSystem,
    StageSelectSystem, StatRecalcSystem, SurvivorTextureHandles, TitleVisualSystem, UiIconSystem,
    WhipSystem, ACTOR_FRAMES_PATH, ATLAS_PATH, DAIRY_PLANT_TILES_PATH, EFFECTS_PATH,
    EVOLUTIONS_PATH, GAMEOVER_TITLE_EN_PATH, GAMEOVER_TITLE_KO_PATH, ICONS_PATH,
    INGAME_LABEL_GOLD_EN_PATH, INGAME_LABEL_GOLD_KO_PATH, INGAME_LABEL_HP_EN_PATH,
    INGAME_LABEL_HP_KO_PATH, INGAME_LABEL_KILLS_EN_PATH, INGAME_LABEL_KILLS_KO_PATH,
    INGAME_LABEL_LV_EN_PATH, INGAME_LABEL_LV_KO_PATH, INGAME_LABEL_PASSIVES_EN_PATH,
    INGAME_LABEL_PASSIVES_KO_PATH, INGAME_LABEL_XP_EN_PATH, INGAME_LABEL_XP_KO_PATH,
    INLAID_LIBRARY_TILES_PATH, LEVELUP_TITLE_EN_PATH, LEVELUP_TITLE_KO_PATH, MAD_FOREST_TILES_PATH,
    MENU_BUTTON_ACHIEVEMENTS_EN_PATH, MENU_BUTTON_ACHIEVEMENTS_KO_PATH,
    MENU_BUTTON_CHARACTER_EN_PATH, MENU_BUTTON_CHARACTER_KO_PATH, MENU_BUTTON_SETTINGS_EN_PATH,
    MENU_BUTTON_SETTINGS_KO_PATH, MENU_BUTTON_SHOP_EN_PATH, MENU_BUTTON_SHOP_KO_PATH,
    MENU_BUTTON_STAGE_EN_PATH, MENU_BUTTON_STAGE_KO_PATH, MENU_BUTTON_START_EN_PATH,
    MENU_BUTTON_START_KO_PATH, PASSIVES_PATH, POWERUPS_PATH, RESTART_HINT_EN_PATH,
    RESTART_HINT_KO_PATH, SECTION_PASSIVES_EN_PATH, SECTION_PASSIVES_KO_PATH,
    SECTION_WEAPONS_EN_PATH, SECTION_WEAPONS_KO_PATH, TITLE_BACKDROP_PATH, TITLE_LOGO_PLAQUE_PATH,
    UI_MODAL_PANEL_PATH, UI_SLOT_FRAME_PATH,
};

fn main() {
    env_logger::init();

    // Textures and SFX load through `assets/...` relative paths; anchor the working
    // directory so they resolve no matter where the executable was launched from.
    set_working_dir_to_asset_root();

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
    app.add_system(DebugInputSystem); // 시각검증: H=도움말 F5=Bomb F6=Rosary B=보스 L=LevelUp
    app.add_system(ModeTransitionSystem); // Phase 8-A: 최상단 — SurvivorMode 전환 + GameState 동기화
    app.add_system(BgmSystem::default()); // Phase 11-H: BGM 자동 전환
    app.add_system(ShopInputSystem); // Phase 8-B: Shop 입력 처리
    app.add_system(CharacterSelectSystem); // Phase 9: 캐릭터 선택 입력 처리
    app.add_system(StageSelectSystem); // Phase 10: 스테이지 선택 입력 처리
    app.add_system(StatRecalcSystem); // 패시브/파워업/캐릭터 보정을 합산해 PlayerStats 갱신
    app.add_system(LevelUpSystem);
    app.add_system(PlayerMovementSystem::default());
    app.add_system(EnemyAiSystem::default());
    app.add_system(SpawnDirectorSystem::default());
    app.add_system(BossSpawnSystem); // Phase 5: 시간축 보스 등장 트리거
    app.add_system(BossPhaseSystem); // Phase 5: HP 비율 → phase 전환
    app.add_system(BossDeathSystem); // Phase 5: 보스 사망 + StageClear
    app.add_system(EnemyContactDamageSystem::default());
    app.add_system(HealthRegenSystem::default()); // Phase 3-A: recovery stat 적용 (0.0 default → no-op)
    app.add_system(DeathSystem);
    app.add_system(RestartSystem);
    app.add_system(CameraFollowSystem::default());
    app.add_system(BackgroundSystem::default());
    app.add_system(TitleVisualSystem);
    app.add_system(WhipSystem::default());
    app.add_system(MagicWandSystem::default());
    app.add_system(KnifeSystem::default());
    app.add_system(AxeSystem::default());
    app.add_system(CrossSystem::default());
    app.add_system(FireWandSystem::default());
    app.add_system(GarlicSystem::default());
    app.add_system(HolyWaterSystem);
    app.add_system(HolyWaterPoolSystem::default());
    app.add_system(KingBibleSystem); // Phase 2-F 신규 (책 스폰)
    app.add_system(OrbitingBookSystem::default()); // Phase 2-F 신규 (책 회전 + tick)
    app.add_system(LightningRingSystem::default()); // Phase 2-F 신규 (랜덤 적 즉시 area damage)
    app.add_system(LightningFlashSystem); // Phase 2-F 신규 (flash lifetime)
    app.add_system(ProjectileSystem::default());
    app.add_system(MagnetSystem::default());
    app.add_system(ChestPickupSystem::default()); // Phase 6: Chest 픽업 + 진화 시도
    app.add_system(PickupSystem::default()); // Phase 7: 픽업 5종 자동 픽업
    app.add_system(AchievementSystem); // Phase C: 업적 감지 + 해금 보상
    app.add_system(HitFlashSystem);
    app.add_system(DamageNumberSystem::default()); // Phase 11: 데미지 숫자 age/이동/despawn
    app.add_system(ParticleSystem::default()); // Phase 11-E: 파티클 이동/페이드/despawn
    app.add_system(AnimationSystem::new()); // 스프라이트시트 기반 actor/effect UV 진행
    app.add_system(ActorFacingSystem::default()); // 이동 방향에 맞춰 actor UV 좌우 반전
    app.add_system(UiIconSystem); // HUD/카드/상점 아이콘을 화면 고정 스프라이트로 표시
    app.add_system(HudSystem); // 마지막 — TextQueue push (데미지 숫자 포함)
    app.add_system(SfxSystem::default()); // Phase 11-D: SfxQueue drain → AudioManager

    // 한글 폰트 — 엔진이 아닌 게임이 책임짐
    app.world.insert_resource(FontData(
        include_bytes!("../../../../assets/fonts/NotoSansKR-Regular.ttf").to_vec(),
    ));

    // 창 설정: 저장된 해상도 + 게임 전용 제목/배경색
    let meta = MetaSave::load_or_default();
    let (w, h) = ResolutionPreset::from_key(&meta.resolution_key).dimensions();
    app.world.insert_resource(WindowConfig {
        width: w,
        height: h,
        title: "Rust Survivors".to_string(),
        clear_color: [0.04, 0.04, 0.08, 1.0],
    });

    let survivor_textures = SurvivorTextureHandles {
        atlas: app.load_image(ATLAS_PATH),
        effects: app.load_image(EFFECTS_PATH),
        actor_frames: app.load_image(ACTOR_FRAMES_PATH),
        evolutions: app.load_image(EVOLUTIONS_PATH),
        icons: app.load_image(ICONS_PATH),
        passives: app.load_image(PASSIVES_PATH),
        powerups: app.load_image(POWERUPS_PATH),
        mad_forest_tiles: app.load_image(MAD_FOREST_TILES_PATH),
        inlaid_library_tiles: app.load_image(INLAID_LIBRARY_TILES_PATH),
        dairy_plant_tiles: app.load_image(DAIRY_PLANT_TILES_PATH),
        title_backdrop: app.load_image(TITLE_BACKDROP_PATH),
        title_logo_plaque: app.load_image(TITLE_LOGO_PLAQUE_PATH),
        menu_button_start_ko: app.load_image(MENU_BUTTON_START_KO_PATH),
        menu_button_start_en: app.load_image(MENU_BUTTON_START_EN_PATH),
        menu_button_character_ko: app.load_image(MENU_BUTTON_CHARACTER_KO_PATH),
        menu_button_character_en: app.load_image(MENU_BUTTON_CHARACTER_EN_PATH),
        menu_button_stage_ko: app.load_image(MENU_BUTTON_STAGE_KO_PATH),
        menu_button_stage_en: app.load_image(MENU_BUTTON_STAGE_EN_PATH),
        menu_button_shop_ko: app.load_image(MENU_BUTTON_SHOP_KO_PATH),
        menu_button_shop_en: app.load_image(MENU_BUTTON_SHOP_EN_PATH),
        menu_button_achievements_ko: app.load_image(MENU_BUTTON_ACHIEVEMENTS_KO_PATH),
        menu_button_achievements_en: app.load_image(MENU_BUTTON_ACHIEVEMENTS_EN_PATH),
        menu_button_settings_ko: app.load_image(MENU_BUTTON_SETTINGS_KO_PATH),
        menu_button_settings_en: app.load_image(MENU_BUTTON_SETTINGS_EN_PATH),
        ui_modal_panel: app.load_image(UI_MODAL_PANEL_PATH),
        ui_slot_frame: app.load_image(UI_SLOT_FRAME_PATH),
        ingame_label_lv_ko: app.load_image(INGAME_LABEL_LV_KO_PATH),
        ingame_label_lv_en: app.load_image(INGAME_LABEL_LV_EN_PATH),
        ingame_label_hp_ko: app.load_image(INGAME_LABEL_HP_KO_PATH),
        ingame_label_hp_en: app.load_image(INGAME_LABEL_HP_EN_PATH),
        ingame_label_xp_ko: app.load_image(INGAME_LABEL_XP_KO_PATH),
        ingame_label_xp_en: app.load_image(INGAME_LABEL_XP_EN_PATH),
        ingame_label_gold_ko: app.load_image(INGAME_LABEL_GOLD_KO_PATH),
        ingame_label_gold_en: app.load_image(INGAME_LABEL_GOLD_EN_PATH),
        ingame_label_kills_ko: app.load_image(INGAME_LABEL_KILLS_KO_PATH),
        ingame_label_kills_en: app.load_image(INGAME_LABEL_KILLS_EN_PATH),
        ingame_label_passives_ko: app.load_image(INGAME_LABEL_PASSIVES_KO_PATH),
        ingame_label_passives_en: app.load_image(INGAME_LABEL_PASSIVES_EN_PATH),
        levelup_title_ko: app.load_image(LEVELUP_TITLE_KO_PATH),
        levelup_title_en: app.load_image(LEVELUP_TITLE_EN_PATH),
        gameover_title_ko: app.load_image(GAMEOVER_TITLE_KO_PATH),
        gameover_title_en: app.load_image(GAMEOVER_TITLE_EN_PATH),
        restart_hint_ko: app.load_image(RESTART_HINT_KO_PATH),
        restart_hint_en: app.load_image(RESTART_HINT_EN_PATH),
        section_weapons_ko: app.load_image(SECTION_WEAPONS_KO_PATH),
        section_weapons_en: app.load_image(SECTION_WEAPONS_EN_PATH),
        section_passives_ko: app.load_image(SECTION_PASSIVES_KO_PATH),
        section_passives_en: app.load_image(SECTION_PASSIVES_EN_PATH),
    };
    app.world.insert_resource(survivor_textures);

    // 초기 엔티티 + 리소스 설정 (GameStats 포함)
    setup_survivor_world(&mut app.world);

    // 이벤트 루프 진입 (ESC → 일시정지 메뉴, 메뉴에서 종료 선택 가능)
    app.run();
}
