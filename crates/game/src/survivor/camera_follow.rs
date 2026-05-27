use super::boss::{Boss, CameraShake};
use super::player::Player;
use engine::{Camera, System, Transform, ViewportSize, World};
use glam::Vec2;
use rand::Rng;

/// 보스 활성 여부에 따른 줌 목표값.
const ZOOM_NORMAL: f32 = 1.0;
/// 보스가 살아있을 때 줌아웃 → 전투 공간 확보.
const ZOOM_BOSS: f32 = 0.65;
/// zoom 보간 속도 (1.0 = 즉시, 0.01 = 매우 느림).
const ZOOM_LERP: f32 = 0.05;

/// 카메라를 플레이어가 화면 중앙에 오도록 따라가게 한다.
pub struct CameraFollowSystem {
    /// 예전 smooth follow 설정값. 현재 서바이버에서는 플레이어 중앙 고정을 우선한다.
    pub lerp: f32,
}

impl Default for CameraFollowSystem {
    fn default() -> Self {
        Self { lerp: 0.15 }
    }
}

impl System for CameraFollowSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        // 1. 플레이어 위치 찾기 (없으면 종료)
        let player_pos = world
            .query2::<Player, Transform>()
            .next()
            .map(|(_, _, t)| t.position);
        let Some(target) = player_pos else { return };

        // 2. 동적 줌 — 보스 살아있으면 줌아웃, 없으면 복귀 (Phase 11-F)
        // zoom 이 바뀌면 화면에 보이는 월드 크기도 바뀐다. 그래서 위치 계산보다 먼저 갱신한다.
        let boss_alive = world.query::<Boss>().next().is_some();
        let target_zoom = if boss_alive { ZOOM_BOSS } else { ZOOM_NORMAL };
        let camera_zoom = if let Some(cam) = world.resource_mut::<Camera>() {
            cam.zoom += (target_zoom - cam.zoom) * ZOOM_LERP;
            cam.zoom
        } else {
            target_zoom
        };

        // 3. ViewportSize 리소스에서 현재 뷰포트 크기를 읽어 카메라 중앙 계산
        let viewport = world
            .resource::<ViewportSize>()
            .map(|v| Vec2::new(v.width, v.height))
            .unwrap_or(Vec2::new(1280.0, 720.0));
        let desired_top_left = desired_camera_top_left(target, viewport, camera_zoom);
        if let Some(cam) = world.resource_mut::<Camera>() {
            cam.position = desired_top_left;
        }

        // 4. CameraShake 적용 — shake 와 cam 을 동시에 mut 빌릴 수 없으므로
        //    shake 값을 먼저 스칼라로 읽은 뒤 cam 을 빌린다.
        let shake_offset: Option<Vec2> = {
            let shake = world.resource_mut::<CameraShake>();
            if let Some(s) = shake {
                s.elapsed += dt;
                if s.is_active() {
                    let intensity = s.intensity * (1.0 - s.elapsed / s.duration);
                    let mut rng = rand::thread_rng();
                    Some(Vec2::new(
                        rng.gen_range(-intensity..intensity),
                        rng.gen_range(-intensity..intensity),
                    ))
                } else {
                    None
                }
            } else {
                None
            }
        }; // shake 의 mutable borrow 여기서 종료

        if let Some(offset) = shake_offset {
            if let Some(cam) = world.resource_mut::<Camera>() {
                cam.position += offset;
            }
        }
    }
}

fn desired_camera_top_left(target: Vec2, viewport: Vec2, zoom: f32) -> Vec2 {
    let zoom = zoom.max(0.01);
    target - viewport / (2.0 * zoom)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::survivor::boss::BossKind;
    use crate::survivor::sprites::PLAYER_VISUAL_SIZE;
    use engine::World;

    #[test]
    fn desired_top_left_keeps_target_at_screen_center_when_zoomed_out() {
        let target = Vec2::new(1000.0, 800.0);
        let viewport = Vec2::new(800.0, 600.0);
        let zoom = 0.65;

        let top_left = desired_camera_top_left(target, viewport, zoom);
        let screen = (target - top_left) * zoom;

        assert!((screen.x - viewport.x * 0.5).abs() < 0.001);
        assert!((screen.y - viewport.y * 0.5).abs() < 0.001);
    }

    #[test]
    fn camera_follow_keeps_player_centered_when_boss_zoom_is_active() {
        let mut world = World::new();
        let player = world.spawn();
        let player_pos = Vec2::new(1000.0, 800.0);
        let viewport = ViewportSize {
            width: 800.0,
            height: 600.0,
        };

        world.add_component(player, Player);
        world.add_component(
            player,
            Transform {
                position: player_pos,
                scale: Vec2::splat(PLAYER_VISUAL_SIZE),
                rotation: 0.0,
                z: 1.0,
            },
        );
        let boss = world.spawn();
        world.add_component(
            boss,
            Boss {
                kind: BossKind::GiantSlime,
                phase: 0,
                move_speed: 0.0,
                spawn_time: 0.0,
                spawned: true,
            },
        );
        world.insert_resource(Camera::new(Vec2::ZERO, 0.65));
        world.insert_resource(viewport);

        CameraFollowSystem::default().run(&mut world, 0.016);

        let camera = *world.resource::<Camera>().unwrap();
        let screen = (player_pos - camera.position) * camera.zoom;

        assert!((screen.x - viewport.width * 0.5).abs() < 0.001);
        assert!((screen.y - viewport.height * 0.5).abs() < 0.001);
    }
}
