use std::collections::HashSet;
use winit::keyboard::KeyCode;

/// 키보드·마우스 상태를 담는 ECS 리소스
///
/// World에 삽입 후 시스템에서 `world.resource::<InputState>()` 로 접근.
#[derive(Default)]
pub struct InputState {
    pressed:      HashSet<KeyCode>,
    just_pressed:  HashSet<KeyCode>,
    just_released: HashSet<KeyCode>,
}

impl InputState {
    /// 키를 누른 순간 true (1프레임만)
    pub fn just_pressed(&self, key: KeyCode) -> bool {
        self.just_pressed.contains(&key)
    }

    /// 키를 누르고 있는 동안 true
    pub fn is_pressed(&self, key: KeyCode) -> bool {
        self.pressed.contains(&key)
    }

    /// 키를 뗀 순간 true (1프레임만)
    pub fn just_released(&self, key: KeyCode) -> bool {
        self.just_released.contains(&key)
    }

    // ── 내부 업데이트 (App에서만 호출) ──────────────────────────────────────

    pub(crate) fn press(&mut self, key: KeyCode) {
        if self.pressed.insert(key) {
            self.just_pressed.insert(key);
        }
    }

    pub(crate) fn release(&mut self, key: KeyCode) {
        self.pressed.remove(&key);
        self.just_released.insert(key);
    }

    /// 프레임 끝에 just_* 초기화
    pub(crate) fn flush(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
    }
}
