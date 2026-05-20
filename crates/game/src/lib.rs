//! 게임 모듈 라이브러리.
//!
//! - `survivor` — Vampire Survivors 클론 (탑다운 자동공격 로그라이트)
//! - 플랫포머 데모는 `bin/game.rs` 가 직접 소유 (lib 의존성 없음)

pub mod survivor;

#[cfg(test)]
mod tests {
    use super::survivor::{
        restart_world, setup_survivor_world, spawn_player, spawn_xp_gem, spawn_zombie,
        CameraFollowSystem, CardKind, DeathSystem, EnemyAiSystem, EnemyContactDamageSystem,
        EnemySpawnSystem, GameStats, Health, HitFlash, LevelUpSystem, MagnetSystem,
        PendingLevelUp, Player, PlayerStats, SpawnTimer, WeaponInventory, WeaponKind, WhipSystem,
        XpAccumulator, XpGem, Zombie,
    };
    use engine::{Camera, System, Transform, World};
    use engine::components::GameState;
    use glam::Vec2;

    #[test]
    fn player_stats_default() {
        assert_eq!(PlayerStats::default().move_speed, 200.0);
    }

    #[test]
    fn spawn_player_inserts_components() {
        let mut world = World::new();
        spawn_player(&mut world);
        // Player 가 있는 엔티티가 정확히 1개
        let count = world.query::<Player>().count();
        assert_eq!(count, 1);
    }

    #[test]
    fn camera_follow_moves_toward_player() {
        let mut world = World::new();
        world.insert_resource(Camera::default());
        spawn_player(&mut world);
        // 플레이어를 멀리 이동 — query 임시값이 drop 된 뒤 get_mut 호출해야 borrow checker 통과
        let player_entity = world.query2::<Player, Transform>().next().map(|(e, _, _)| e);
        if let Some(entity) = player_entity {
            if let Some(t) = world.get_mut::<Transform>(entity) {
                t.position = Vec2::new(1000.0, 1000.0);
            }
        }
        let cam_before = world.resource::<Camera>().map(|c| c.position).unwrap();
        let mut sys = CameraFollowSystem::default();
        sys.run(&mut world, 0.016);
        let cam_after = world.resource::<Camera>().map(|c| c.position).unwrap();
        // 카메라가 (0,0) 에서 (1000-400, 1000-300)=(600,700) 방향으로 lerp 0.15 만큼 움직였어야 함
        assert!(cam_after.x > cam_before.x, "camera should move toward player");
        assert!(cam_after.y > cam_before.y);
    }

    #[test]
    fn enemy_ai_moves_toward_player() {
        let mut world = World::new();
        world.insert_resource(GameState::Playing);
        world.insert_resource(Camera::default());
        spawn_player(&mut world);

        // 플레이어를 원점에 고정
        let player_entity = world.query2::<Player, Transform>().next().map(|(e, _, _)| e);
        if let Some(entity) = player_entity {
            if let Some(t) = world.get_mut::<Transform>(entity) {
                t.position = Vec2::new(0.0, 0.0);
            }
        }

        // 좀비를 왼쪽(-100, 0)에 스폰
        spawn_zombie(&mut world, Vec2::new(-100.0, 0.0));

        let zombie_entity = world.query2::<Zombie, Transform>().next().map(|(e, _, _)| e);
        let before_x = zombie_entity
            .and_then(|e| world.get::<Transform>(e))
            .map(|t| t.position.x)
            .unwrap();

        EnemyAiSystem.run(&mut world, 0.5);

        let after_x = zombie_entity
            .and_then(|e| world.get::<Transform>(e))
            .map(|t| t.position.x)
            .unwrap();

        assert!(after_x > before_x, "zombie should move toward player (+x direction)");
    }

    #[test]
    fn spawn_timer_triggers_after_interval() {
        let mut world = World::new();
        world.insert_resource(GameState::Playing);
        world.insert_resource(Camera::new(Vec2::ZERO, 1.0));
        world.insert_resource(SpawnTimer { interval: 1.0, elapsed: 0.0 });

        EnemySpawnSystem::default().run(&mut world, 1.0);

        let count = world.query::<Zombie>().count();
        assert_eq!(count, 1, "one zombie should have spawned");
    }

    #[test]
    fn enemy_despawned_when_far() {
        let mut world = World::new();
        world.insert_resource(GameState::Playing);
        world.insert_resource(Camera::new(Vec2::ZERO, 1.0));
        // 타이머 발화를 막으려고 interval 을 매우 크게 설정
        world.insert_resource(SpawnTimer { interval: 999.0, elapsed: 0.0 });

        // despawn_radius(900) 밖에 좀비 스폰
        spawn_zombie(&mut world, Vec2::new(2000.0, 0.0));
        assert_eq!(world.query::<Zombie>().count(), 1);

        EnemySpawnSystem::default().run(&mut world, 0.0);

        assert_eq!(world.query::<Zombie>().count(), 0, "far zombie should be despawned");
    }

    // ── Phase 1-C: Whip + DamageSystem 테스트 ────────────────────────────────

    fn setup_whip_world() -> (World, /* zombie */ engine::Entity) {
        let mut world = World::new();
        world.insert_resource(GameState::Playing);
        world.insert_resource(Camera::new(Vec2::ZERO, 1.0));
        spawn_player(&mut world);
        // 플레이어를 원점으로 고정
        let player_entity = world.query2::<Player, Transform>().next().map(|(e, _, _)| e);
        if let Some(e) = player_entity {
            if let Some(t) = world.get_mut::<Transform>(e) {
                t.position = Vec2::ZERO;
            }
        }
        // 좀비를 우측 AABB(0..120, -30..30) 안에 배치
        spawn_zombie(&mut world, Vec2::new(80.0, 0.0));
        let zombie_entity = world.query2::<Zombie, Transform>().next().map(|(e, _, _)| e).unwrap();
        (world, zombie_entity)
    }

    #[test]
    fn whip_does_not_trigger_before_cooldown() {
        let (mut world, zombie) = setup_whip_world();
        // cooldown 1.0 미만 → 발화 안 됨
        WhipSystem::default().run(&mut world, 0.5);
        let hp = world.get::<Health>(zombie).map(|h| h.current).unwrap();
        assert_eq!(hp, 30.0, "cooldown 전에는 데미지 없어야 함");
    }

    #[test]
    fn whip_damages_zombies_in_range() {
        let (mut world, zombie) = setup_whip_world();
        // dt == cooldown → 발화
        WhipSystem::default().run(&mut world, 1.0);
        let hp = world.get::<Health>(zombie).map(|h| h.current).unwrap();
        assert_eq!(hp, 20.0, "10 데미지 차감 기대 (30 → 20)");
    }

    #[test]
    fn zombie_dies_when_hp_reaches_zero() {
        let (mut world, zombie) = setup_whip_world();
        // HP 를 5 로 낮춤 — Whip 데미지(10) 에 즉사
        if let Some(h) = world.get_mut::<Health>(zombie) {
            *h = Health::new(5.0);
        }
        WhipSystem::default().run(&mut world, 1.0);
        assert_eq!(world.query::<Zombie>().count(), 0, "즉사한 좀비는 despawn 돼야 함");
    }

    #[test]
    fn whip_hit_attaches_hit_flash() {
        let (mut world, zombie) = setup_whip_world();
        // dt == cooldown → 발화. 좀비 HP(30) > damage(10) → 생존 → HitFlash 부착 기대
        WhipSystem::default().run(&mut world, 1.0);

        // HitFlash 컴포넌트가 부착됐는지 확인
        assert!(
            world.get::<HitFlash>(zombie).is_some(),
            "hit 된 좀비에 HitFlash 컴포넌트가 부착돼야 함"
        );

        // sprite 색이 빨강으로 변경됐는지 확인
        use engine::Sprite;
        let color = world.get::<Sprite>(zombie).map(|s| s.color).unwrap();
        assert_eq!(color, [1.0, 0.3, 0.3, 1.0], "hit 된 좀비의 sprite 색이 빨강이어야 함");
    }

    // ── Phase 1-D: XpGem 드롭 + 자석 흡수 테스트 ────────────────────────────

    #[test]
    fn xp_gem_spawned_when_zombie_dies() {
        let mut world = World::new();
        world.insert_resource(GameState::Playing);
        world.insert_resource(Camera::new(Vec2::ZERO, 1.0));
        spawn_player(&mut world);

        // 플레이어를 원점으로 고정
        let player_entity = world.query2::<Player, Transform>().next().map(|(e, _, _)| e);
        if let Some(e) = player_entity {
            if let Some(t) = world.get_mut::<Transform>(e) {
                t.position = Vec2::ZERO;
            }
        }

        // 좀비를 Whip AABB(우측 0..120, -30..30) 안에 배치하고 HP 를 5 로 낮춰 즉사
        spawn_zombie(&mut world, Vec2::new(80.0, 0.0));
        let zombie_entity = world.query2::<Zombie, Transform>().next().map(|(e, _, _)| e).unwrap();
        if let Some(h) = world.get_mut::<Health>(zombie_entity) {
            *h = Health::new(5.0);
        }

        // WhipSystem 발화 → zombie 즉사 → XpGem 스폰
        WhipSystem::default().run(&mut world, 1.0);

        assert_eq!(
            world.query::<Zombie>().count(),
            0,
            "zombie 가 despawn 되어야 함"
        );
        assert_eq!(
            world.query::<XpGem>().count(),
            1,
            "zombie 사망 시 XpGem 이 1개 스폰돼야 함"
        );
    }

    #[test]
    fn xp_gem_attracted_within_magnet_radius() {
        let mut world = World::new();
        world.insert_resource(GameState::Playing);
        spawn_player(&mut world);

        // 플레이어를 원점으로 고정
        let player_entity = world.query2::<Player, Transform>().next().map(|(e, _, _)| e);
        if let Some(e) = player_entity {
            if let Some(t) = world.get_mut::<Transform>(e) {
                t.position = Vec2::ZERO;
            }
        }

        // 자석 반경(80px) 안, 픽업 반경(20px) 밖에 gem 스폰
        spawn_xp_gem(&mut world, Vec2::new(50.0, 0.0), 1);

        MagnetSystem::default().run(&mut world, 0.1);

        // gem 이 플레이어 쪽으로(x 감소 방향) 이동했어야 함
        let gem_entity = world.query2::<XpGem, Transform>().next().map(|(e, _, _)| e).unwrap();
        let gem_x = world.get::<Transform>(gem_entity).map(|t| t.position.x).unwrap();
        assert!(
            gem_x < 50.0,
            "자석 반경 안의 gem 이 플레이어 방향으로 이동해야 함 (x={gem_x} < 50.0)"
        );
    }

    #[test]
    fn xp_gem_picked_up_in_range() {
        let mut world = World::new();
        world.insert_resource(GameState::Playing);
        spawn_player(&mut world);

        // 플레이어를 원점으로 고정
        let player_entity = world.query2::<Player, Transform>().next().map(|(e, _, _)| e);
        if let Some(e) = player_entity {
            if let Some(t) = world.get_mut::<Transform>(e) {
                t.position = Vec2::ZERO;
            }
        }

        // 픽업 반경(20px) 안에 gem 스폰
        spawn_xp_gem(&mut world, Vec2::new(10.0, 0.0), 1);

        // XpAccumulator 초기값 확인
        let pe = world.query2::<Player, XpAccumulator>().next().map(|(e, _, _)| e).unwrap();
        let before = world.get::<XpAccumulator>(pe).map(|a| a.current).unwrap();
        assert_eq!(before, 0, "픽업 전 XP 는 0 이어야 함");

        MagnetSystem::default().run(&mut world, 0.1);

        // gem 이 사라졌는지 확인
        assert_eq!(
            world.query::<XpGem>().count(),
            0,
            "픽업 반경 안의 gem 은 픽업돼 despawn 돼야 함"
        );

        // XpAccumulator 가 갱신됐는지 확인
        let after = world.get::<XpAccumulator>(pe).map(|a| a.current).unwrap();
        assert_eq!(after, 1, "XpAccumulator.current 가 1 이어야 함");
    }

    // ── Phase 1-E: 레벨업 + 카드 선택 테스트 ────────────────────────────────

    /// XP 가 임계치(5)에 도달하면 GameState::Paused 로 전환하고 PendingLevelUp 이 삽입돼야 함
    #[test]
    fn levelup_triggered_when_xp_reaches_threshold() {
        let mut world = World::new();
        world.insert_resource(GameState::Playing);
        spawn_player(&mut world);

        // XpAccumulator.current 를 임계치(5) 로 강제 설정
        let pe = world.query2::<Player, XpAccumulator>().next().map(|(e, _, _)| e).unwrap();
        if let Some(acc) = world.get_mut::<XpAccumulator>(pe) {
            acc.current = 5;
        }

        LevelUpSystem.run(&mut world, 0.0);

        // GameState 가 Paused 로 전환됐는지
        let state = world.resource::<GameState>().cloned();
        assert_eq!(state, Some(GameState::Paused), "XP 임계치 도달 시 GameState::Paused 여야 함");

        // PendingLevelUp 리소스가 삽입됐는지
        let pending = world.resource::<PendingLevelUp>();
        assert!(pending.is_some(), "PendingLevelUp 리소스가 삽입돼야 함");
        assert!(!pending.unwrap().consumed, "consumed 는 false 여야 함");
    }

    /// apply_card(WhipDamage) 호출 시 WeaponInventory Whip 슬롯 damage +5, level+1, next_threshold 갱신
    #[test]
    fn levelup_applies_card_whip_damage() {
        let mut world = World::new();
        world.insert_resource(GameState::Playing);
        spawn_player(&mut world);

        // 초기 값 확인 — WeaponInventory 경로로 접근
        let pe = world.query2::<Player, XpAccumulator>().next().map(|(e, _, _)| e).unwrap();
        let before_damage = world.get::<WeaponInventory>(pe)
            .and_then(|inv| inv.whip_slot())
            .map(|s| { let WeaponKind::Whip { damage, .. } = s.kind; damage })
            .unwrap();
        assert_eq!(before_damage, 10.0);

        // apply_card 직접 호출 (키 입력 우회 — InputState::press 가 pub(crate))
        LevelUpSystem::apply_card(&mut world, pe, CardKind::WhipDamage);

        // WeaponInventory Whip 슬롯 damage +5 확인
        let after_damage = world.get::<WeaponInventory>(pe)
            .and_then(|inv| inv.whip_slot())
            .map(|s| { let WeaponKind::Whip { damage, .. } = s.kind; damage })
            .unwrap();
        assert_eq!(after_damage, 15.0, "WhipDamage 카드 적용 후 damage 가 15.0 이어야 함");

        // XpAccumulator.level +1, next_threshold 갱신 확인
        // next_threshold(2) = 5 + 5*2 = 15
        let acc = world.get::<XpAccumulator>(pe).unwrap();
        assert_eq!(acc.level, 2, "레벨이 2 가 돼야 함");
        assert_eq!(acc.next_threshold, 15, "L2 임계치는 next_threshold(2)=15 이어야 함");
    }

    /// GameState::Paused 상태에서 EnemyAiSystem 이 좀비를 이동시키지 않아야 함
    #[test]
    fn paused_blocks_enemy_movement() {
        let mut world = World::new();
        world.insert_resource(GameState::Paused);
        world.insert_resource(Camera::default());
        spawn_player(&mut world);

        // 플레이어를 원점에 고정
        let pe = world.query2::<Player, Transform>().next().map(|(e, _, _)| e);
        if let Some(e) = pe {
            if let Some(t) = world.get_mut::<Transform>(e) {
                t.position = Vec2::ZERO;
            }
        }

        spawn_zombie(&mut world, Vec2::new(100.0, 0.0));

        let ze = world.query2::<Zombie, Transform>().next().map(|(e, _, _)| e).unwrap();
        let before_x = world.get::<Transform>(ze).map(|t| t.position.x).unwrap();

        EnemyAiSystem.run(&mut world, 1.0);

        let after_x = world.get::<Transform>(ze).map(|t| t.position.x).unwrap();
        assert_eq!(
            after_x, before_x,
            "Paused 상태에서 좀비는 이동하면 안 됨 (before={before_x}, after={after_x})"
        );
    }

    // ── Phase 1-F: HUD + 사망 + GameOver + R 재시작 테스트 ──────────────────

    /// HP <= 0 이면 DeathSystem 이 GameState::GameOver 로 전환해야 함
    #[test]
    fn player_dies_when_hp_reaches_zero() {
        let mut world = World::new();
        world.insert_resource(GameState::Playing);
        spawn_player(&mut world);

        // Player Health 를 0 으로 강제 설정
        let pe = world.query2::<Player, Health>().next().map(|(e, _, _)| e).unwrap();
        if let Some(h) = world.get_mut::<Health>(pe) {
            *h = Health::new(0.0);
        }

        DeathSystem.run(&mut world, 0.0);

        let state = world.resource::<GameState>().cloned();
        assert_eq!(
            state,
            Some(GameState::GameOver),
            "HP 0 이면 GameState::GameOver 여야 함"
        );
    }

    /// restart_world 호출 후: Player 1개, Zombie/XpGem 0개, GameState::Playing
    #[test]
    fn restart_clears_world_and_resumes() {
        let mut world = World::new();
        world.insert_resource(GameState::Playing);
        world.insert_resource(Camera::new(Vec2::ZERO, 1.0));

        // 초기 씬 구성
        setup_survivor_world(&mut world);

        // 잡것 추가
        spawn_zombie(&mut world, Vec2::new(200.0, 0.0));
        spawn_zombie(&mut world, Vec2::new(-200.0, 0.0));
        spawn_xp_gem(&mut world, Vec2::new(50.0, 50.0), 1);

        // GameOver 로 강제 전환
        world.insert_resource(GameState::GameOver);

        // 재시작 로직 직접 호출 (InputState 시뮬레이션 없이 헬퍼 사용)
        restart_world(&mut world);

        // Player 1개만 남아야 함
        assert_eq!(
            world.query::<Player>().count(),
            1,
            "재시작 후 Player 는 정확히 1개여야 함"
        );
        // Zombie 전부 제거됐어야 함
        assert_eq!(
            world.query::<Zombie>().count(),
            0,
            "재시작 후 Zombie 는 0개여야 함"
        );
        // XpGem 전부 제거됐어야 함
        assert_eq!(
            world.query::<XpGem>().count(),
            0,
            "재시작 후 XpGem 은 0개여야 함"
        );
        // GameState 가 Playing 으로 복귀했어야 함
        let state = world.resource::<GameState>().cloned();
        assert_eq!(
            state,
            Some(GameState::Playing),
            "재시작 후 GameState::Playing 이어야 함"
        );
    }

    /// 접촉 반경(25px) 안에 좀비가 있고 cooldown 이 경과했으면 Player HP -= 10
    #[test]
    fn enemy_contact_damages_player_after_cooldown() {
        let mut world = World::new();
        world.insert_resource(GameState::Playing);
        spawn_player(&mut world);
        world.insert_resource(GameStats::default());

        // 플레이어를 원점에 고정
        let pe = world.query2::<Player, Transform>().next().map(|(e, _, _)| e).unwrap();
        if let Some(t) = world.get_mut::<Transform>(pe) {
            t.position = Vec2::ZERO;
        }

        // contact_radius(25) 안에 좀비 배치
        spawn_zombie(&mut world, Vec2::new(15.0, 0.0));

        let before_hp = world.get::<Health>(pe).map(|h| h.current).unwrap();
        assert_eq!(before_hp, 100.0);

        // dt = cooldown(1.0) → cooldown 경과, 데미지 적용
        EnemyContactDamageSystem::default().run(&mut world, 1.0);

        let after_hp = world.get::<Health>(pe).map(|h| h.current).unwrap();
        assert_eq!(
            after_hp, 90.0,
            "적 접촉 후 HP 가 90.0 이어야 함 (100 - 10)"
        );
    }
}
