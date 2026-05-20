//! Vampire Survivors 클론 — Phase 1-A 진입점.
//!
//! 현재 상태: 노란 사각형 플레이어가 WASD/방향키로 이동하고, 카메라가 lerp 로 따라감.
//! 다음 단계 (Phase 1-B): Zombie 적 + 스폰 시스템.

use engine::App;
use game::survivor::{spawn_player, CameraFollowSystem, PlayerMovementSystem};

fn main() {
    env_logger::init();

    let mut app = App::new();

    // 1. 시스템 등록 — 입력 → 이동 → 카메라 follow 순서
    app.add_system(PlayerMovementSystem);
    app.add_system(CameraFollowSystem::default());

    // 2. 초기 엔티티
    spawn_player(&mut app.world);

    // 3. 이벤트 루프 진입 (ESC 로 종료)
    app.run();
}
