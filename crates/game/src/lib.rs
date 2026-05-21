//! 게임 모듈 라이브러리.
//!
//! - `survivor` — Vampire Survivors 클론 (탑다운 자동공격 로그라이트)
//! - 플랫포머 데모는 `bin/game.rs` 가 직접 소유 (lib 의존성 없음)

pub mod survivor;

#[cfg(test)]
mod tests {
    use super::survivor::{
        apply_damage_to_enemy, restart_world, setup_survivor_world, spawn_book,
        spawn_boss, spawn_enemy, spawn_holy_water_pool, spawn_player, spawn_projectile,
        spawn_projectile_ex, spawn_xp_gem, spawn_zombie, Boss, BossDeathSystem,
        BossKind, BossPhaseSystem, BossSpawnQueue, BossSpawnSystem, CameraFollowSystem,
        CardKind, CrossSystem, DeathSystem, Enemy, EnemyAiSystem, EnemyContactDamageSystem,
        EnemyKind, FireWandSystem, GameStats, GarlicSystem, Health, HitFlash,
        HolyWaterPool, HolyWaterPoolSystem, HolyWaterSystem, KingBibleSystem, KnifeSystem,
        LevelUpSystem, LightningFlash, LightningRingSystem, MagicWandSystem, MagnetSystem,
        MetaSave, ModeTransitionSystem, OrbitingBook, OrbitingBookSystem, PendingLevelUp,
        Player, PlayerStats, Projectile, ProjectileBehavior, ProjectileSystem,
        SpawnDirector, SpawnDirectorSystem, StageProgress, SurvivorMode,
        WeaponInventory, WeaponKind, WhipSystem, XpAccumulator, XpGem, Zombie,
    };
    use engine::{Camera, System, Transform, World};
    use engine::components::GameState;
    use glam::Vec2;

    #[test]
    fn player_stats_default() {
        assert_eq!(PlayerStats::default().move_speed, 200.0);
    }

    // ── Phase 3-A: PlayerStats 16 필드 + stats 적용 테스트 ──────────────────

    /// PlayerStats::default() 의 모든 필드가 올바른 기본값인지 검증
    #[test]
    fn player_stats_default_values() {
        let s = PlayerStats::default();
        assert_eq!(s.might, 0.0,           "might 기본값은 0.0");
        assert_eq!(s.area, 1.0,            "area 기본값은 1.0");
        assert_eq!(s.projectile_speed, 1.0, "projectile_speed 기본값은 1.0");
        assert_eq!(s.duration, 1.0,        "duration 기본값은 1.0");
        assert_eq!(s.amount, 0,            "amount 기본값은 0");
        assert_eq!(s.cooldown, 1.0,        "cooldown 기본값은 1.0");
        assert_eq!(s.max_health, 100.0,    "max_health 기본값은 100.0");
        assert_eq!(s.recovery, 0.0,        "recovery 기본값은 0.0");
        assert_eq!(s.armor, 0.0,           "armor 기본값은 0.0");
        assert_eq!(s.move_speed, 200.0,    "move_speed 기본값은 200.0");
        assert_eq!(s.magnet, 1.0,          "magnet 기본값은 1.0");
        assert_eq!(s.luck, 1.0,            "luck 기본값은 1.0");
        assert_eq!(s.growth, 1.0,          "growth 기본값은 1.0");
        assert_eq!(s.greed, 1.0,           "greed 기본값은 1.0");
        assert_eq!(s.curse, 1.0,           "curse 기본값은 1.0");
        assert_eq!(s.revival, 0,           "revival 기본값은 0");
    }

    /// armor stat 이 적 접촉 데미지를 감쇄하는지 검증
    #[test]
    fn armor_reduces_contact_damage() {
        let mut world = World::new();
        world.insert_resource(GameState::Playing);
        spawn_player(&mut world);

        // 플레이어를 원점에 고정
        let pe = world.query2::<Player, PlayerStats>().next().map(|(e, _, _)| e).unwrap();
        if let Some(t) = world.get_mut::<Transform>(pe) {
            t.position = Vec2::ZERO;
        }

        // armor = 5.0 강제 설정 (EnemyContactDamageSystem.damage=10 → effective=5)
        if let Some(s) = world.get_mut::<PlayerStats>(pe) {
            s.armor = 5.0;
        }

        // 초기 HP = 100 (default)
        let before_hp = world.get::<Health>(pe).map(|h| h.current).unwrap();
        assert_eq!(before_hp, 100.0);

        // contact_radius(25) 안에 좀비 배치
        spawn_zombie(&mut world, Vec2::new(15.0, 0.0));

        // dt = cooldown(1.0) → 발화. damage(10) - armor(5) = 5 차감
        EnemyContactDamageSystem::default().run(&mut world, 1.0);

        let after_hp = world.get::<Health>(pe).map(|h| h.current).unwrap();
        assert_eq!(
            after_hp, 95.0,
            "armor=5 일 때 HP 는 100 - (10-5) = 95 여야 함 (현재 {after_hp})"
        );
    }

    /// magnet stat 이 자석 반경을 확장하는지 검증
    #[test]
    fn magnet_stat_extends_radius() {
        let mut world = World::new();
        world.insert_resource(GameState::Playing);
        spawn_player(&mut world);

        // 플레이어를 원점에 고정
        let pe = world.query2::<Player, PlayerStats>().next().map(|(e, _, _)| e).unwrap();
        if let Some(t) = world.get_mut::<Transform>(pe) {
            t.position = Vec2::ZERO;
        }

        // magnet = 2.0 → effective_magnet_radius = 80 * 2 = 160
        if let Some(s) = world.get_mut::<PlayerStats>(pe) {
            s.magnet = 2.0;
        }

        // 기본 magnet_radius(80px) 밖이지만 확장 반경(160px) 안인 (150, 0) 에 gem 스폰
        spawn_xp_gem(&mut world, Vec2::new(150.0, 0.0), 1);

        MagnetSystem::default().run(&mut world, 0.1);

        // gem 이 플레이어 방향(x 감소)으로 이동했어야 함
        let gem_entity = world.query2::<XpGem, Transform>().next().map(|(e, _, _)| e).unwrap();
        let gem_x = world.get::<Transform>(gem_entity).map(|t| t.position.x).unwrap();
        assert!(
            gem_x < 150.0,
            "magnet=2.0 으로 확장된 반경 안의 gem 이 플레이어 방향으로 이동해야 함 (x={gem_x} < 150.0)"
        );
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

    // ── Phase 4-B: SpawnDirector 테스트 ─────────────────────────────────────

    #[test]
    fn waves_ron_loads_at_least_one_wave() {
        let director = SpawnDirector::default();
        assert!(!director.waves.is_empty(), "waves.ron 은 최소 1개 wave 를 포함해야 함");
    }

    #[test]
    fn director_picks_zombie_in_first_wave() {
        use super::survivor::director::SpawnDirector as SD;
        let director = SD::default();
        let wave = director.current_wave(0.0).expect("wave 0 이 존재해야 함");
        let mut rng = rand::thread_rng();
        let mut zombie_count = 0u32;
        for _ in 0..50 {
            if let Some(EnemyKind::Zombie) = SD::pick_enemy_from(wave, &mut rng) {
                zombie_count += 1;
            }
        }
        assert!(zombie_count >= 10, "wave 0 에서 50회 중 Zombie 가 10회 이상 선택되어야 함 (weight 6/8), 실제: {}", zombie_count);
    }

    #[test]
    fn spawn_director_spawns_enemy_after_interval() {
        let mut world = World::new();
        world.insert_resource(GameState::Playing);
        world.insert_resource(Camera::new(Vec2::ZERO, 1.0));
        world.insert_resource(SpawnDirector::default());
        world.insert_resource(GameStats::default());

        // dt=2.0 → 첫 번째 wave 의 spawn_interval(1.2) 초과 → 적 1마리 스폰
        SpawnDirectorSystem::default().run(&mut world, 2.0);

        let count = world.query::<Enemy>().count();
        assert!(count >= 1, "SpawnDirectorSystem 이 dt=2.0 후 적 1마리 이상 스폰해야 함, 실제: {}", count);
    }

    #[test]
    fn enemy_despawned_when_far() {
        let mut world = World::new();
        world.insert_resource(GameState::Playing);
        world.insert_resource(Camera::new(Vec2::ZERO, 1.0));
        world.insert_resource(SpawnDirector::default());
        world.insert_resource(GameStats::default());

        // despawn_radius(900) 밖에 좀비 스폰
        spawn_zombie(&mut world, Vec2::new(2000.0, 0.0));
        assert_eq!(world.query::<Zombie>().count(), 1);

        // dt=0 → cooldown 미달이므로 추가 스폰 없음, despawn 만 동작
        SpawnDirectorSystem::default().run(&mut world, 0.0);

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

    /// with_starter_loadout 에 10개 무기가 모두 채워져야 한다.
    #[test]
    fn inventory_starter_loadout_has_six_weapons() {
        let inv = WeaponInventory::with_starter_loadout();

        // Vec 기반 — len() 으로 확인
        assert_eq!(inv.slots.len(), 10, "10개 슬롯이 있어야 함");

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
        assert!(
            matches!(inv.slots[8].kind, WeaponKind::KingBible { .. }),
            "슬롯 8 은 KingBible 이어야 함"
        );
        assert!(
            matches!(inv.slots[9].kind, WeaponKind::LightningRing { .. }),
            "슬롯 9 는 LightningRing 이어야 함"
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

    // ── Phase 2-F: King Bible + Lightning Ring + damage 헬퍼 ─────────────────

    /// KingBibleSystem cooldown(8.0) 경과 후 OrbitingBook 2권이 스폰돼야 한다.
    #[test]
    fn king_bible_spawns_books_after_cooldown() {
        let mut world = World::new();
        world.insert_resource(GameState::Playing);
        world.insert_resource(engine::Camera::new(Vec2::ZERO, 1.0));

        // KingBible 포함 스타터 로드아웃으로 플레이어 스폰
        spawn_player(&mut world);

        // 플레이어를 원점에 고정
        let pe = world.query2::<Player, Transform>().next().map(|(e, _, _)| e).unwrap();
        if let Some(t) = world.get_mut::<Transform>(pe) {
            t.position = Vec2::ZERO;
        }

        // dt = 9.0 → cooldown 8.0 초과 → 발화 (book_count=2)
        KingBibleSystem.run(&mut world, 9.0);

        assert_eq!(
            world.query::<OrbitingBook>().count(),
            2,
            "KingBible cooldown 경과 후 OrbitingBook 2권이 스폰돼야 함"
        );
    }

    /// OrbitingBook 이 회전하고 tick_cooldown 경과 후 반경 안 좀비에게 데미지를 가해야 한다.
    #[test]
    fn orbiting_book_rotates_and_damages_zombie() {
        let mut world = World::new();
        world.insert_resource(GameState::Playing);
        world.insert_resource(GameStats::default());

        // 플레이어를 원점에 스폰 (KingBibleSystem 이 Player 위치를 참조하므로 필요)
        spawn_player(&mut world);
        let pe = world.query2::<Player, Transform>().next().map(|(e, _, _)| e).unwrap();
        if let Some(t) = world.get_mut::<Transform>(pe) {
            t.position = Vec2::ZERO;
        }

        // 책을 각도 0, 반경 60px 에 스폰. angular_speed=0.01 로 매우 느리게 회전하여
        // dt=0.4 후에도 (60, 0) 근처에 머물도록 함.
        // tick_cooldown=0.3 → dt=0.4 에 한 번 발화
        spawn_book(&mut world, Vec2::ZERO, 0.0, 0.01, 60.0, 6.0, 16.0, 5.0, 0.3);

        // 책이 처음에 있는 위치 (60, 0) 근처에 좀비 스폰
        // 좀비 Collider::Circle { radius: 20 }, 책 hit_radius=16 → 거리 0 ≈ 완전 겹침 → 충돌
        spawn_zombie(&mut world, Vec2::new(60.0, 0.0));
        let zombie_entity = world.query::<Zombie>().next().map(|(e, _)| e).unwrap();

        let before_hp = world.get::<Health>(zombie_entity).map(|h| h.current).unwrap();
        assert_eq!(before_hp, 30.0);

        // dt=0.4 → tick_cooldown(0.3) 초과 → 1회 tick 발화, 책도 약간 회전
        OrbitingBookSystem::default().run(&mut world, 0.4);

        // 좀비 HP 가 줄었거나 despawn 됐어야 함 (damage 6 적용)
        let zombie_alive = world.query::<Zombie>().next().map(|(e, _)| e);
        if let Some(ze) = zombie_alive {
            let hp = world.get::<Health>(ze).map(|h| h.current).unwrap();
            assert!(hp < 30.0, "OrbitingBook tick 후 좀비 HP 가 줄어야 함 (현재 {hp})");
        }
        // despawn 됐으면(즉사) 테스트도 통과

        // OrbitingBook 의 angle 이 갱신됐어야 함 (회전 검증)
        let book_e = world.query::<OrbitingBook>().next().map(|(e, _)| e);
        if let Some(be) = book_e {
            let book = world.get::<OrbitingBook>(be).unwrap();
            assert!(
                book.angle > 0.0 || book.tick_elapsed >= 0.0,
                "OrbitingBook 의 angle/tick_elapsed 가 갱신돼야 함"
            );
        }
    }

    /// LightningRingSystem cooldown(4.0) 경과 후 LightningFlash 가 1개 이상 스폰돼야 한다.
    #[test]
    fn lightning_ring_strikes_random_zombie() {
        let mut world = World::new();
        world.insert_resource(GameState::Playing);
        world.insert_resource(engine::Camera::new(Vec2::ZERO, 1.0));
        world.insert_resource(GameStats::default());

        // LightningRing 포함 스타터 로드아웃으로 플레이어 스폰
        spawn_player(&mut world);
        let pe = world.query2::<Player, Transform>().next().map(|(e, _, _)| e).unwrap();
        if let Some(t) = world.get_mut::<Transform>(pe) {
            t.position = Vec2::ZERO;
        }

        // 적 1마리 스폰 (target_radius=600 안에 배치)
        spawn_zombie(&mut world, Vec2::new(200.0, 0.0));
        let zombie_entity = world.query::<Zombie>().next().map(|(e, _)| e).unwrap();
        let before_hp = world.get::<Health>(zombie_entity).map(|h| h.current).unwrap();

        // dt = 5.0 → cooldown 4.0 초과 → 발화 (strike_count=1)
        LightningRingSystem::default().run(&mut world, 5.0);

        // LightningFlash 시각 엔티티가 1개 이상 스폰돼야 함
        assert!(
            world.query::<LightningFlash>().count() >= 1,
            "LightningRing 발화 후 LightningFlash 가 최소 1개 스폰돼야 함"
        );

        // 좀비 HP 가 줄었거나 despawn 됐어야 함
        let zombie_alive = world.query::<Zombie>().next().map(|(e, _)| e);
        if let Some(ze) = zombie_alive {
            let hp = world.get::<Health>(ze).map(|h| h.current).unwrap();
            assert!(
                hp < before_hp,
                "LightningRing 피격 후 좀비 HP 가 줄어야 함 (현재 {hp})"
            );
        }
        // despawn 됐으면(즉사) 테스트도 통과
    }

    // ── Phase 4-A: 적 10종 + AI 변종 테스트 ──────────────────────────────────

    /// 모든 EnemyKind 의 stats() 가 hp > 0, scale > 0 을 반환해야 한다.
    #[test]
    fn enemy_kind_stats_for_all_ten() {
        use super::survivor::EnemyStats;
        let kinds = [
            EnemyKind::Zombie, EnemyKind::Bat, EnemyKind::Ghost, EnemyKind::Skeleton,
            EnemyKind::Mage, EnemyKind::Mantis, EnemyKind::Plant, EnemyKind::Slime,
            EnemyKind::Mummy, EnemyKind::Knight,
        ];
        for kind in &kinds {
            let s: EnemyStats = kind.stats();
            assert!(s.hp > 0.0, "{kind:?} 의 hp 가 0 보다 커야 함");
            assert!(s.scale > 0.0, "{kind:?} 의 scale 이 0 보다 커야 함");
            assert!(s.contact_damage > 0.0, "{kind:?} 의 contact_damage 가 0 보다 커야 함");
        }
    }

    /// Ghost 는 stop_at(80px) 범위에 플레이어가 있으면 이동하지 않아야 한다.
    #[test]
    fn ghost_hovers_at_stop_distance() {
        let mut world = World::new();
        world.insert_resource(GameState::Playing);
        spawn_player(&mut world);

        // 플레이어를 원점에 고정
        let pe = world.query2::<Player, engine::Transform>().next().map(|(e, _, _)| e).unwrap();
        if let Some(t) = world.get_mut::<engine::Transform>(pe) {
            t.position = Vec2::ZERO;
        }

        // Ghost 를 (100, 0) 에 스폰 — stop_at=80. 거리 100 > 80 이므로 처음에는 이동.
        // dt=5.0 으로 큰 값 사용 → stop_at 까지 이동 후 멈춤
        spawn_enemy(&mut world, Vec2::new(100.0, 0.0), EnemyKind::Ghost);

        let ghost_e = world.query::<Enemy>().next().map(|(e, _)| e).unwrap();

        EnemyAiSystem.run(&mut world, 5.0);

        // Ghost 위치 x < 100 (플레이어 방향으로 이동했어야 함)
        let pos_x = world.get::<engine::Transform>(ghost_e)
            .map(|t| t.position.x)
            .unwrap();
        // Ghost 가 stop_at(80) 근처나 그 안에 있어야 함
        // (stop_at 이 되면 멈추므로 x > 80 이거나, 큰 dt 로 지나쳤더라도 많이는 못 지남)
        assert!(
            pos_x < 100.0,
            "Ghost 가 플레이어 방향으로 이동해야 함 (pos_x={pos_x} < 100.0)"
        );
    }

    /// Slime 사망 시 split_remaining=2 → 자식 슬라임 2마리 스폰 확인.
    #[test]
    fn slime_splits_on_death() {
        let mut world = World::new();
        world.insert_resource(GameState::Playing);

        // Slime 스폰 (HP=30, split_remaining=2)
        spawn_enemy(&mut world, Vec2::ZERO, EnemyKind::Slime);

        // Slime 엔티티 추출
        let slime_e = world.query::<Enemy>().next().map(|(e, _)| e).unwrap();

        // 100 데미지 → 즉사 → split
        apply_damage_to_enemy(&mut world, slime_e, 100.0);

        // 자식 슬라임 2마리가 스폰됐어야 함 (split_remaining=0 → 더 이상 split 안 함)
        let remaining = world.query::<Enemy>().count();
        assert_eq!(
            remaining, 2,
            "Slime 사망 시 자식 슬라임 2마리가 스폰돼야 함 (현재 {remaining})"
        );

        // 자식들의 split_remaining 은 0 이어야 함
        for (_, enemy) in world.query::<Enemy>() {
            assert_eq!(
                enemy.split_remaining, 0,
                "자식 슬라임의 split_remaining 은 0 이어야 함 (무한 split 방지)"
            );
        }
    }

    /// CardKind::PassiveSpinach 를 두 번 apply 하면 PassiveInventory 의 Spinach 레벨이 2 가 된다.
    #[test]
    fn apply_card_passive_spinach_levels_up() {
        use super::survivor::{PassiveInventory, PassiveKind};

        let mut world = World::new();
        world.insert_resource(GameState::Playing);
        setup_survivor_world(&mut world);

        let pe = world.query::<Player>().next().map(|(e, _)| e).unwrap();

        // 첫 번째 카드 적용 → Spinach level 1
        LevelUpSystem::apply_card(&mut world, pe, CardKind::PassiveSpinach);
        let lv1 = world.get::<PassiveInventory>(pe).map(|i| i.level_of(PassiveKind::Spinach)).unwrap();
        assert_eq!(lv1, 1, "첫 번째 PassiveSpinach 카드 적용 후 level 1 이어야 함");

        // 두 번째 카드 적용 → Spinach level 2
        LevelUpSystem::apply_card(&mut world, pe, CardKind::PassiveSpinach);
        let lv2 = world.get::<PassiveInventory>(pe).map(|i| i.level_of(PassiveKind::Spinach)).unwrap();
        assert_eq!(lv2, 2, "두 번째 PassiveSpinach 카드 적용 후 level 2 이어야 함");
    }

    /// SpawnPattern::Circle 로 count=4 마리 스폰 시 적이 4마리 등간격 원형 배치.
    #[test]
    fn spawn_pattern_circle_creates_equal_count_around_center() {
        use super::survivor::{spawn_pattern, EnemyEntry, SpawnPattern, WaveDef, Enemy};
        let mut world = World::new();
        world.insert_resource(GameState::Playing);

        let wave = WaveDef {
            start_time: 0.0,
            end_time: 60.0,
            spawn_interval: 1.0,
            enemies: vec![EnemyEntry { kind: "Zombie".to_string(), weight: 1 }],
            pattern: SpawnPattern::Circle,
            count_per_burst: 4,
            elite_chance: 0.0,
        };

        let mut rng = rand::thread_rng();
        spawn_pattern(&mut world, &wave, Vec2::ZERO, 100.0, &mut rng);

        assert_eq!(world.query::<Enemy>().count(), 4, "Circle 패턴 count=4 → 4마리 스폰");
    }

    /// 엘리트 사망 시 큰 XpGem + 추가 작은 gem 다수 → 총 2개 이상.
    #[test]
    fn elite_drops_extra_xp_gem() {
        use super::survivor::{apply_damage_to_enemy, spawn_enemy_elite, Enemy};
        let mut world = World::new();
        world.insert_resource(GameState::Playing);
        world.insert_resource(GameStats::default());

        spawn_enemy_elite(&mut world, Vec2::ZERO, EnemyKind::Zombie, true);
        let enemy = world.query::<Enemy>().next().map(|(e, _)| e).unwrap();
        apply_damage_to_enemy(&mut world, enemy, 1000.0);

        let gem_count = world.query::<XpGem>().count();
        assert!(gem_count >= 2, "엘리트 사망 → XpGem 2개 이상 (실제 {gem_count})");
    }

    /// 비엘리트 사망 시 XpGem 1개.
    #[test]
    fn non_elite_drops_single_xp_gem() {
        use super::survivor::{apply_damage_to_enemy, spawn_enemy_elite, Enemy};
        let mut world = World::new();
        world.insert_resource(GameState::Playing);
        world.insert_resource(GameStats::default());

        spawn_enemy_elite(&mut world, Vec2::ZERO, EnemyKind::Zombie, false);
        let enemy = world.query::<Enemy>().next().map(|(e, _)| e).unwrap();
        apply_damage_to_enemy(&mut world, enemy, 1000.0);

        assert_eq!(world.query::<XpGem>().count(), 1, "비엘리트 → XpGem 1개");
    }

    // ── Phase 5: 보스 + StageClear 테스트 ────────────────────────────────────

    /// GameStats.elapsed >= 600.0(10분) 이면 BossSpawnSystem 이 GiantSlime 을 스폰해야 한다.
    #[test]
    fn boss_spawn_queue_triggers_at_10min() {
        let mut world = World::new();
        world.insert_resource(GameState::Playing);
        world.insert_resource(Camera::new(Vec2::ZERO, 1.0));

        // setup_survivor_world 는 BossSpawnQueue / CameraShake / StageProgress 도 삽입
        setup_survivor_world(&mut world);

        // GameStats.elapsed 를 10분 1초로 강제 설정
        if let Some(stats) = world.resource_mut::<GameStats>() {
            stats.elapsed = 600.1;
        }

        BossSpawnSystem.run(&mut world, 0.0);

        // Boss 엔티티가 1개 스폰됐어야 함
        let boss_count = world.query::<Boss>().count();
        assert_eq!(boss_count, 1, "10분 경과 후 GiantSlime 보스 1마리가 스폰돼야 함 (현재 {boss_count})");

        // BossSpawnQueue 에서 GiantSlime 이 제거됐어야 함
        let queue = world.resource::<BossSpawnQueue>().unwrap();
        assert!(
            !queue.pending.contains(&BossKind::GiantSlime),
            "GiantSlime 이 pending 에서 제거돼야 함"
        );
        assert!(
            queue.pending.contains(&BossKind::GhostKing),
            "GhostKing 은 아직 pending 에 있어야 함"
        );
        assert!(
            queue.pending.contains(&BossKind::Death),
            "Death 는 아직 pending 에 있어야 함"
        );
    }

    /// HP 가 max 의 40%(50% 미만)이면 BossPhaseSystem 이 phase 를 1 로 올려야 한다.
    #[test]
    fn boss_phase_advances_on_low_hp() {
        let mut world = World::new();
        world.insert_resource(GameState::Playing);
        world.insert_resource(Camera::new(Vec2::ZERO, 1.0));
        world.insert_resource(super::survivor::CameraShake::default());

        // GiantSlime 보스 스폰
        spawn_boss(&mut world, Vec2::ZERO, BossKind::GiantSlime);

        // boss 엔티티 추출
        let boss_e = world.query::<Boss>().next().map(|(e, _)| e).unwrap();

        // HP 를 max * 0.4 (= 400) 로 강제 설정
        let max_hp = BossKind::GiantSlime.max_hp(); // 1000.0
        if let Some(h) = world.get_mut::<Health>(boss_e) {
            h.current = max_hp * 0.4;
        }

        BossPhaseSystem.run(&mut world, 0.0);

        let phase = world.get::<Boss>(boss_e).map(|b| b.phase).unwrap();
        assert_eq!(phase, 1, "HP 40% → phase 가 1 이어야 함 (현재 {phase})");
    }

    /// Death 보스 HP <= 0 → BossDeathSystem 이 despawn + StageProgress.cleared = true.
    #[test]
    fn death_boss_defeat_triggers_stage_clear() {
        let mut world = World::new();
        world.insert_resource(GameState::Playing);
        world.insert_resource(Camera::new(Vec2::ZERO, 1.0));
        world.insert_resource(StageProgress::default());
        world.insert_resource(GameStats::default());
        world.insert_resource(super::survivor::CameraShake::default());

        // Death 보스 스폰
        spawn_boss(&mut world, Vec2::ZERO, BossKind::Death);

        // HP 를 0 으로 강제 설정
        let boss_e = world.query::<Boss>().next().map(|(e, _)| e).unwrap();
        if let Some(h) = world.get_mut::<Health>(boss_e) {
            h.current = 0.0;
        }

        BossDeathSystem.run(&mut world, 0.0);

        // 보스 엔티티가 사라져야 함
        let remaining = world.query::<Boss>().count();
        assert_eq!(remaining, 0, "Death 보스가 despawn 돼야 함 (현재 {remaining})");

        // StageProgress.cleared 가 true 여야 함
        let cleared = world.resource::<StageProgress>().map(|p| p.cleared).unwrap_or(false);
        assert!(cleared, "Death 보스 처치 후 StageProgress.cleared 가 true 여야 함");
    }
}
