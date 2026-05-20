use engine::{Camera, System, Transform, World};
use glam::Vec2;
use super::player::Player;

/// 카메라를 플레이어 위치로 lerp 따라가게 한다.
pub struct CameraFollowSystem {
    /// 0.0 = 정지, 1.0 = 즉시 따라감. 기본 0.15.
    pub lerp: f32,
    /// 카메라 뷰포트 크기(픽셀). 매 프레임 갱신 안 해도 됨 — App 윈도우 크기에 맞춰 setup 에서 설정.
    pub viewport: Vec2,
}

impl Default for CameraFollowSystem {
    fn default() -> Self {
        Self {
            lerp: 0.15,
            viewport: Vec2::new(800.0, 600.0),
        }
    }
}

impl System for CameraFollowSystem {
    fn run(&mut self, world: &mut World, _dt: f32) {
        // 1. 플레이어 위치 찾기 (없으면 종료)
        let player_pos = world
            .query2::<Player, Transform>()
            .next()
            .map(|(_, _, t)| t.position);
        let Some(target) = player_pos else { return };

        // 2. Camera 리소스를 lerp 로 갱신
        // 카메라 좌상단 = 플레이어 - 뷰포트/2 가 되어야 화면 중앙에 플레이어가 위치
        let desired_top_left = target - self.viewport * 0.5;
        if let Some(cam) = world.resource_mut::<Camera>() {
            cam.position = cam.position.lerp(desired_top_left, self.lerp);
        }
    }
}
