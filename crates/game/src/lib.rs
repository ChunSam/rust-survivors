//! 게임 모듈 라이브러리.
//!
//! - `survivor` — Vampire Survivors 클론 (탑다운 자동공격 로그라이트)
//! - 플랫포머 데모는 `bin/game.rs` 가 직접 소유 (lib 의존성 없음)

pub mod survivor;

#[cfg(test)]
mod tests {
    use super::survivor::{
        restart_world, setup_survivor_world, spawn_holy_water_pool, spawn_player, spawn_projectile,
        spawn_projectile_ex, spawn_xp_gem, spawn_zombie, CameraFollowSystem, CardKind, CrossSystem,
        DeathSystem, EnemyAiSystem, EnemyContactDamageSystem, EnemySpawnSystem, FireWandSystem,
        GameStats, GarlicSystem, Health, HitFlash, HolyWaterPool, HolyWaterPoolSystem,
        HolyWaterSystem, KnifeSystem, LevelUpSystem, MagicWandSystem, MagnetSystem, PendingLevelUp,
        Player, PlayerStats, Projectile, ProjectileBehavior, ProjectileSystem, SpawnTimer,
        WeaponInventory, WeaponKind, WhipSystem, XpAccumulator, XpGem, Zombie,
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
            .and_then(|s| {
                if let WeaponKind::Whip { damage, .. } = s.kind { Some(damage) } else { None }
            })
            .unwrap();
        assert_eq!(before_damage, 10.0);

        // apply_card 직접 호출 (키 입력 우회 — InputState::press 가 pub(crate))
        LevelUpSystem::apply_card(&mut world, pe, CardKind::WhipDamage);

        // WeaponInventory Whip 슬롯 damage +5 확인
        let after_damage = world.get::<WeaponInventory>(pe)
            .and_then(|inv| inv.whip_slot())
            .and_then(|s| {
                if let WeaponKind::Whip { damage, .. } = s.kind { Some(damage) } else { None }
            })
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

    // ── Phase 2-B: ProjectileSystem + MagicWand 테스트 ──────────────────────

    /// 수명이 소진된 투사체는 despawn 돼야 한다.
    #[test]
    fn projectile_expires_after_lifetime() {
        let mut world = World::new();
        world.insert_resource(GameState::Playing);

        spawn_projectile(
            &mut world,
            Vec2::ZERO,
            Vec2::new(100.0, 0.0),
            0.5,
            0,
            5.0,
            [1.0, 1.0, 1.0],
        );

        // dt = 1.0 → lifetime 0.5 소진
        ProjectileSystem::default().run(&mut world, 1.0);

        assert_eq!(
            world.query::<Projectile>().count(),
            0,
            "수명이 소진된 투사체는 despawn 돼야 함"
        );
    }

    /// 투사체가 좀비와 충돌하면 데미지를 입힌다.
    #[test]
    fn projectile_damages_zombie_on_collision() {
        let mut world = World::new();
        world.insert_resource(GameState::Playing);
        world.insert_resource(GameStats::default());

        // 좀비를 (30, 0) 에 스폰 (HP 30, Collider::Circle { radius: 20 })
        spawn_zombie(&mut world, Vec2::new(30.0, 0.0));

        // 투사체를 원점에서 +x 방향으로 발사
        // dt=0.1 → new_pos=(10, 0)
        // 거리 = 30 - 10 = 20, 합산 반경 = 20 + 6 = 26 → 충돌
        spawn_projectile(
            &mut world,
            Vec2::ZERO,
            Vec2::new(100.0, 0.0),
            2.0,
            0,
            5.0,
            [1.0, 1.0, 1.0],
        );

        ProjectileSystem::default().run(&mut world, 0.1);

        // 좀비 HP 가 줄었거나 despawn 됐어야 함
        let zombie_entity = world.query::<Zombie>().next().map(|(e, _)| e);
        if let Some(ze) = zombie_entity {
            let hp = world.get::<Health>(ze).map(|h| h.current).unwrap_or(0.0);
            assert!(hp < 30.0, "투사체 피격 후 좀비 HP 가 줄어야 함 (현재 {hp})");
        }
        // despawn 됐으면(즉사) 테스트도 통과
    }

    /// MagicWand cooldown 경과 후 투사체 1개가 스폰돼야 한다.
    #[test]
    fn magic_wand_spawns_projectile_after_cooldown() {
        let mut world = World::new();
        world.insert_resource(GameState::Playing);
        world.insert_resource(Camera::new(Vec2::ZERO, 1.0));

        // with_starter_loadout 으로 MagicWand 포함 플레이어 스폰
        spawn_player(&mut world);

        // 플레이어를 원점에 고정
        let pe = world.query2::<Player, Transform>().next().map(|(e, _, _)| e).unwrap();
        if let Some(t) = world.get_mut::<Transform>(pe) {
            t.position = Vec2::ZERO;
        }

        // 적이 없으면 MagicWandSystem 이 발사를 건너뜀 — 좀비 1마리 스폰
        spawn_zombie(&mut world, Vec2::new(100.0, 0.0));

        // dt = 1.5 → cooldown 1.2 초과 → 발화
        MagicWandSystem::default().run(&mut world, 1.5);

        assert_eq!(
            world.query::<Projectile>().count(),
            1,
            "MagicWand cooldown 경과 후 투사체 1개가 스폰돼야 함"
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

    // ── Phase 2-C: ProjectileBehavior + Knife + Axe 테스트 ──────────────────

    /// KnifeSystem cooldown 경과 후 투사체 1개가 스폰돼야 한다.
    #[test]
    fn knife_spawns_projectile_after_cooldown() {
        let mut world = World::new();
        world.insert_resource(GameState::Playing);
        world.insert_resource(Camera::new(Vec2::ZERO, 1.0));

        // Knife 포함 스타터 로드아웃으로 플레이어 스폰
        spawn_player(&mut world);

        // 플레이어를 원점에 고정
        let pe = world.query2::<Player, Transform>().next().map(|(e, _, _)| e).unwrap();
        if let Some(t) = world.get_mut::<Transform>(pe) {
            t.position = Vec2::ZERO;
        }

        // 적 없으면 KnifeSystem 이 발사를 건너뜀 — 좀비 1마리 스폰
        spawn_zombie(&mut world, Vec2::new(100.0, 0.0));

        // dt = 1.0 → cooldown 0.8 초과 → 발화
        KnifeSystem::default().run(&mut world, 1.0);

        assert_eq!(
            world.query::<Projectile>().count(),
            1,
            "Knife cooldown 경과 후 투사체 1개가 스폰돼야 함"
        );
    }

    /// Arc behavior 투사체는 매 프레임 velocity.y 가 gravity * dt 만큼 증가해야 한다.
    #[test]
    fn axe_projectile_arcs_downward() {
        let mut world = World::new();
        world.insert_resource(GameState::Playing);

        // 위로 던지는 포물선 투사체 (velocity.y = -200, gravity = 500)
        spawn_projectile_ex(
            &mut world,
            Vec2::ZERO,
            Vec2::new(0.0, -200.0),
            2.0,
            0,
            10.0,
            [1.0, 0.5, 0.2],
            ProjectileBehavior::Arc { gravity: 500.0 },
        );

        // dt = 1.0 → velocity.y = -200 + 500*1 = 300 (양수 = 아래 방향)
        ProjectileSystem::default().run(&mut world, 1.0);

        // 투사체가 아직 살아 있어야 함 (lifetime 2.0 - 1.0 = 1.0 > 0)
        assert_eq!(
            world.query::<Projectile>().count(),
            1,
            "아직 수명이 남은 투사체여야 함"
        );

        // velocity.y 가 중력 적용 후 증가했어야 함 (-200 + 500 = 300)
        let proj_e = world.query::<Projectile>().next().map(|(e, _)| e).unwrap();
        let vel_y = world.get::<Projectile>(proj_e).map(|p| p.velocity.y).unwrap();
        assert!(
            vel_y > 0.0,
            "중력 적용 후 velocity.y 가 양수(아래 방향)여야 함 (현재 {vel_y})"
        );
        // 기대값: -200 + 500 * 1.0 = 300
        assert!(
            (vel_y - 300.0).abs() < 1.0,
            "velocity.y 가 약 300 이어야 함 (현재 {vel_y})"
        );
    }

    // ── Phase 2-D: Cross + FireWand + Inventory Vec 테스트 ──────────────────

    /// with_starter_loadout 에 8개 무기가 모두 채워져야 한다 (Whip/MagicWand/Knife/Axe/Cross/FireWand/Garlic/HolyWater).
    #[test]
    fn inventory_starter_loadout_has_six_weapons() {
        let inv = WeaponInventory::with_starter_loadout();

        // Vec 기반 — len() 으로 확인
        assert_eq!(inv.slots.len(), 8, "8개 슬롯이 있어야 함");

        // 각 슬롯의 kind 확인 (순서 보장)
        assert!(
            matches!(inv.slots[0].kind, WeaponKind::Whip { .. }),
            "슬롯 0 은 Whip 이어야 함"
        );
        assert!(
            matches!(inv.slots[1].kind, WeaponKind::MagicWand { .. }),
            "슬롯 1 은 MagicWand 이어야 함"
        );
        assert!(
            matches!(inv.slots[2].kind, WeaponKind::Knife { .. }),
            "슬롯 2 는 Knife 여야 함"
        );
        assert!(
            matches!(inv.slots[3].kind, WeaponKind::Axe { .. }),
            "슬롯 3 은 Axe 여야 함"
        );
        assert!(
            matches!(inv.slots[4].kind, WeaponKind::Cross { .. }),
            "슬롯 4 는 Cross 여야 함"
        );
        assert!(
            matches!(inv.slots[5].kind, WeaponKind::FireWand { .. }),
            "슬롯 5 는 FireWand 여야 함"
        );
        assert!(
            matches!(inv.slots[6].kind, WeaponKind::Garlic { .. }),
            "슬롯 6 은 Garlic 이어야 함"
        );
        assert!(
            matches!(inv.slots[7].kind, WeaponKind::HolyWater { .. }),
            "슬롯 7 은 HolyWater 여야 함"
        );
    }

    /// CrossSystem cooldown 경과 후 Boomerang behavior 투사체 1개가 스폰돼야 한다.
    #[test]
    fn cross_spawns_boomerang_projectile() {
        let mut world = World::new();
        world.insert_resource(GameState::Playing);
        world.insert_resource(engine::Camera::new(Vec2::ZERO, 1.0));

        // Cross 포함 스타터 로드아웃으로 플레이어 스폰
        spawn_player(&mut world);

        // 플레이어를 원점에 고정
        let pe = world.query2::<Player, engine::Transform>().next().map(|(e, _, _)| e).unwrap();
        if let Some(t) = world.get_mut::<engine::Transform>(pe) {
            t.position = Vec2::ZERO;
        }

        // 적 없으면 CrossSystem 이 발사를 건너뜀 — 좀비 1마리 스폰
        spawn_zombie(&mut world, Vec2::new(80.0, 0.0));

        // dt = 2.0 → cooldown 1.8 초과 → 발화
        CrossSystem::default().run(&mut world, 2.0);

        assert_eq!(
            world.query::<Projectile>().count(),
            1,
            "Cross cooldown 경과 후 투사체 1개가 스폰돼야 함"
        );

        // 스폰된 투사체의 behavior 가 Boomerang 인지 확인
        let proj_e = world.query::<Projectile>().next().map(|(e, _)| e).unwrap();
        let behavior = world.get::<Projectile>(proj_e).map(|p| p.behavior).unwrap();
        assert!(
            matches!(behavior, ProjectileBehavior::Boomerang { .. }),
            "Cross 투사체의 behavior 는 Boomerang 이어야 함"
        );
    }

    /// FireWandSystem cooldown 경과 후 투사체 1개가 스폰돼야 한다 (랜덤 적 타깃).
    #[test]
    fn fire_wand_spawns_projectile_after_cooldown() {
        let mut world = World::new();
        world.insert_resource(GameState::Playing);
        world.insert_resource(engine::Camera::new(Vec2::ZERO, 1.0));

        // FireWand 포함 스타터 로드아웃으로 플레이어 스폰
        spawn_player(&mut world);

        // 플레이어를 원점에 고정
        let pe = world.query2::<Player, engine::Transform>().next().map(|(e, _, _)| e).unwrap();
        if let Some(t) = world.get_mut::<engine::Transform>(pe) {
            t.position = Vec2::ZERO;
        }

        // 적 없으면 FireWandSystem 이 발사를 건너뜀 — 좀비 1마리 스폰
        spawn_zombie(&mut world, Vec2::new(100.0, 0.0));

        // dt = 3.0 → cooldown 2.5 초과 → 발화
        FireWandSystem::default().run(&mut world, 3.0);

        assert_eq!(
            world.query::<Projectile>().count(),
            1,
            "FireWand cooldown 경과 후 투사체 1개가 스폰돼야 함"
        );
    }

    // ── Phase 2-E: Garlic + HolyWater 테스트 ────────────────────────────────

    /// GarlicSystem cooldown 경과 후 반경 안 좀비에게 데미지가 들어가야 한다.
    #[test]
    fn garlic_damages_zombies_in_radius() {
        let mut world = World::new();
        world.insert_resource(GameState::Playing);
        world.insert_resource(engine::Camera::new(Vec2::ZERO, 1.0));

        // Garlic 포함 스타터 로드아웃으로 플레이어 스폰
        spawn_player(&mut world);

        // 플레이어를 원점에 고정
        let pe = world.query2::<Player, Transform>().next().map(|(e, _, _)| e).unwrap();
        if let Some(t) = world.get_mut::<Transform>(pe) {
            t.position = Vec2::ZERO;
        }

        // Garlic radius = 70px 안에 좀비 스폰 (50px)
        spawn_zombie(&mut world, Vec2::new(50.0, 0.0));
        let zombie_entity = world.query::<Zombie>().next().map(|(e, _)| e).unwrap();

        // 초기 HP 확인 (30)
        let before_hp = world.get::<Health>(zombie_entity).map(|h| h.current).unwrap();
        assert_eq!(before_hp, 30.0);

        // dt = 1.0 → cooldown 0.5 초과 → 발화
        GarlicSystem::default().run(&mut world, 1.0);

        // 좀비 HP 가 줄었거나 despawn 됐어야 함 (damage 4 적용)
        let zombie_alive = world.query::<Zombie>().next().map(|(e, _)| e);
        if let Some(ze) = zombie_alive {
            let hp = world.get::<Health>(ze).map(|h| h.current).unwrap();
            assert!(hp < 30.0, "Garlic 피격 후 좀비 HP 가 줄어야 함 (현재 {hp})");
        }
        // despawn 됐으면(즉사) 테스트도 통과
    }

    /// HolyWaterSystem cooldown 경과 후 풀 엔티티가 1개 이상 스폰돼야 한다.
    #[test]
    fn holy_water_spawns_pool_after_cooldown() {
        let mut world = World::new();
        world.insert_resource(GameState::Playing);
        world.insert_resource(engine::Camera::new(Vec2::ZERO, 1.0));

        // HolyWater 포함 스타터 로드아웃으로 플레이어 스폰
        spawn_player(&mut world);

        // 플레이어를 원점에 고정
        let pe = world.query2::<Player, Transform>().next().map(|(e, _, _)| e).unwrap();
        if let Some(t) = world.get_mut::<Transform>(pe) {
            t.position = Vec2::ZERO;
        }

        // dt = 4.0 → cooldown 3.0 초과 → 발화
        HolyWaterSystem.run(&mut world, 4.0);

        assert!(
            world.query::<HolyWaterPool>().count() >= 1,
            "HolyWater cooldown 경과 후 풀이 최소 1개 스폰돼야 함"
        );
    }

    /// HolyWaterPool 이 tick_cooldown 경과 후 반경 안 좀비에게 데미지를 가해야 한다.
    #[test]
    fn pool_damages_zombie_on_tick() {
        let mut world = World::new();
        world.insert_resource(GameState::Playing);
        world.insert_resource(GameStats::default());

        // 원점에 풀 스폰 (damage=5, radius=50, lifetime=3, tick_cooldown=0.5)
        spawn_holy_water_pool(&mut world, Vec2::ZERO, 5.0, 50.0, 3.0, 0.5);

        // 풀 반경(50px) 안에 좀비 스폰 (30px)
        spawn_zombie(&mut world, Vec2::new(30.0, 0.0));
        let zombie_entity = world.query::<Zombie>().next().map(|(e, _)| e).unwrap();

        // dt = 0.6 → tick_cooldown(0.5) 초과 → 1회 tick 발화
        HolyWaterPoolSystem::default().run(&mut world, 0.6);

        // 좀비 HP 가 줄었거나 despawn 됐어야 함 (damage 5 적용)
        let zombie_alive = world.query::<Zombie>().next().map(|(e, _)| e);
        if let Some(ze) = zombie_alive {
            let hp = world.get::<Health>(ze).map(|h| h.current).unwrap();
            assert!(hp < 30.0, "Pool tick 후 좀비 HP 가 줄어야 함 (현재 {hp})");
        }
        // despawn 됐으면(즉사) 테스트도 통과
    }

    /// Boomerang 투사체는 return_at 경과 후 velocity 가 반전돼야 한다.
    #[test]
    fn boomerang_velocity_reverses_after_return_at() {
        let mut world = World::new();
        world.insert_resource(GameState::Playing);

        // return_at = 0.5 초인 Boomerang 투사체 스폰
        spawn_projectile_ex(
            &mut world,
            Vec2::ZERO,
            Vec2::new(100.0, 0.0),
            5.0, // lifetime (충분히 길게)
            0,
            5.0,
            [1.0, 1.0, 0.0],
            ProjectileBehavior::Boomerang { return_at: 0.5, elapsed: 0.0, returned: false },
        );

        // dt = 0.6 → elapsed = 0.6 >= return_at(0.5) → 반전 발생
        ProjectileSystem::default().run(&mut world, 0.6);

        // 투사체가 아직 살아 있어야 함 (lifetime 5.0 - 0.6 = 4.4 > 0)
        assert_eq!(
            world.query::<Projectile>().count(),
            1,
            "아직 수명이 남은 투사체여야 함"
        );

        let proj_e = world.query::<Projectile>().next().map(|(e, _)| e).unwrap();
        let proj = world.get::<Projectile>(proj_e).unwrap();

        // velocity.x 가 음수로 반전돼야 함
        assert!(
            proj.velocity.x < 0.0,
            "반전 후 velocity.x 가 음수여야 함 (현재 {})",
            proj.velocity.x
        );

        // behavior.returned 가 true 여야 함
        assert!(
            matches!(proj.behavior, ProjectileBehavior::Boomerang { returned: true, .. }),
            "반전 후 behavior.returned 가 true 여야 함"
        );
    }
}
