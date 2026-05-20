//! 게임 모듈 라이브러리.
//!
//! - `survivor` — Vampire Survivors 클론 (탑다운 자동공격 로그라이트)
//! - 플랫포머 데모는 `bin/game.rs` 가 직접 소유 (lib 의존성 없음)

pub mod survivor;

#[cfg(test)]
mod tests {
    use super::survivor::{
        spawn_player, spawn_zombie, CameraFollowSystem, EnemyAiSystem, EnemySpawnSystem,
        Player, PlayerStats, SpawnTimer, Zombie,
    };
    use engine::{Camera, System, Transform, World};
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
        world.insert_resource(Camera::new(Vec2::ZERO, 1.0));
        world.insert_resource(SpawnTimer { interval: 1.0, elapsed: 0.0 });

        EnemySpawnSystem::default().run(&mut world, 1.0);

        let count = world.query::<Zombie>().count();
        assert_eq!(count, 1, "one zombie should have spawned");
    }

    #[test]
    fn enemy_despawned_when_far() {
        let mut world = World::new();
        world.insert_resource(Camera::new(Vec2::ZERO, 1.0));
        // 타이머 발화를 막으려고 interval 을 매우 크게 설정
        world.insert_resource(SpawnTimer { interval: 999.0, elapsed: 0.0 });

        // despawn_radius(900) 밖에 좀비 스폰
        spawn_zombie(&mut world, Vec2::new(2000.0, 0.0));
        assert_eq!(world.query::<Zombie>().count(), 1);

        EnemySpawnSystem::default().run(&mut world, 0.0);

        assert_eq!(world.query::<Zombie>().count(), 0, "far zombie should be despawned");
    }
}
