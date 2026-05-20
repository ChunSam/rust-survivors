//! 게임 모듈 라이브러리.
//!
//! - `survivor` — Vampire Survivors 클론 (탑다운 자동공격 로그라이트)
//! - 플랫포머 데모는 `bin/game.rs` 가 직접 소유 (lib 의존성 없음)

pub mod survivor;

#[cfg(test)]
mod tests {
    use super::survivor::{spawn_player, CameraFollowSystem, Player, PlayerStats};
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
}
