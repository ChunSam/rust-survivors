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

/// 카메라를 플레이어 위치로 lerp 따라가게 한다.
pub struct CameraFollowSystem {
    /// 0.0 = 정지, 1.0 = 즉시 따라감. 기본 0.15.
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

        // 2. ViewportSize 리소스에서 현재 뷰포트 크기를 읽어 카메라 중앙 계산
        let viewport = world
            .resource::<ViewportSize>()
            .map(|v| Vec2::new(v.width, v.height))
            .unwrap_or(Vec2::new(1280.0, 720.0));
        let desired_top_left = target - viewport * 0.5;
        if let Some(cam) = world.resource_mut::<Camera>() {
            cam.position = cam.position.lerp(desired_top_left, self.lerp);
        }

        // 3. CameraShake 적용 — shake 와 cam 을 동시에 mut 빌릴 수 없으므로
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

        // 4. 동적 줌 — 보스 살아있으면 줌아웃, 없으면 복귀 (Phase 11-F)
        let boss_alive = world.query::<Boss>().next().is_some();
        let target_zoom = if boss_alive { ZOOM_BOSS } else { ZOOM_NORMAL };
        if let Some(cam) = world.resource_mut::<Camera>() {
            cam.zoom += (target_zoom - cam.zoom) * ZOOM_LERP;
        }
    }
}
