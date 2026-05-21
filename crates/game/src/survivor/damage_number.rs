//! 데미지 숫자(floating combat text) — Phase 11 폴리쉬.
//!
//! 적이 피격되는 순간 적 위치 위로 숫자를 띄운다. 숫자는 일정 시간 동안
//! 위로 떠오르며 페이드아웃한 뒤 자동으로 despawn 된다.
//!
//! ## 구조
//! - `DamageNumber` 컴포넌트(value/age/lifetime) + `Transform` 으로 위치 표현
//! - `spawn_damage_number` 가 적 좌표를 받아 살짝 흔든 위치에 엔티티 생성
//! - `DamageNumberSystem` 이 매 프레임 age 증가 + 위로 이동 + 수명 다하면 despawn
//! - 실제 렌더링은 `HudSystem` 이 `DamageNumber + Transform` 을 카메라로 변환해
//!   `TextQueue` 에 push (이 파일에서는 다루지 않음)

use engine::{Entity, System, Transform, World};
use glam::Vec2;
use rand::Rng;

/// 한 번의 피격에서 발생한 데미지 숫자. 위치는 `Transform` 으로 표현한다.
#[derive(Debug, Clone, Copy)]
pub struct DamageNumber {
    /// 표시할 데미지 값 (소수점 1자리까지 반올림하여 표기).
    pub value:    f32,
    /// 누적 수명(초). 0 에서 시작해 매 프레임 dt 만큼 증가.
    pub age:      f32,
    /// 총 수명(초). `age >= lifetime` 이면 despawn.
    pub lifetime: f32,
}

impl DamageNumber {
    /// 페이드 진행도. 0.0 = 막 스폰, 1.0 = 사라짐 직전.
    pub fn fade(&self) -> f32 {
        (self.age / self.lifetime).clamp(0.0, 1.0)
    }
}

/// 데미지 숫자 한 개를 월드에 스폰한다.
///
/// - `pos` 는 적의 월드 좌표 (보통 적 머리 위 약간 띄움).
/// - `value` 는 표시할 데미지 값 (실제 적용 데미지와 동일).
///
/// 같은 프레임에 여러 무기가 한 적을 때릴 때 숫자가 겹치지 않도록 ±8px
/// 가량 x 를 흔든다 (시각 노이즈).
pub fn spawn_damage_number(world: &mut World, pos: Vec2, value: f32) -> Entity {
    let jitter_x = rand::thread_rng().gen_range(-8.0..8.0);
    let e        = world.spawn();
    world.add_component(e, Transform {
        position: pos + Vec2::new(jitter_x, -16.0),
        scale:    Vec2::new(1.0, 1.0), // 렌더용 스프라이트는 없음. 텍스트로만 그림.
        rotation: 0.0,
        z:        0.95, // HUD 보다 낮고 거의 모든 게임 오브젝트보다 위
    });
    world.add_component(e, DamageNumber {
        value,
        age:      0.0,
        lifetime: 0.6, // 0.6 초간 떠오른 뒤 사라짐
    });
    e
}

/// 매 프레임 데미지 숫자를 위로 이동시키고 수명 다하면 despawn 한다.
///
/// borrow checker 회피: `World::query2` 가 `&` 빌림이라 안에서 despawn 호출 불가.
/// → 1) 먼저 `(entity, new_pos, expired)` 벡터를 모으고
///   2) 그 다음 mut 빌림으로 Transform 업데이트 + age 증가 + 죽은 것 despawn.
#[derive(Default)]
pub struct DamageNumberSystem;

impl System for DamageNumberSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        // 1) 읽기 패스 — entity 와 현재 상태 수집
        let snapshot: Vec<(Entity, f32, f32)> = world
            .query2::<DamageNumber, Transform>()
            .map(|(e, dn, t)| (e, dn.age, t.position.y))
            .collect();

        // 2) 쓰기 패스 — age 증가, 위로 이동, 만료 표시
        let mut expired: Vec<Entity> = Vec::new();
        for (e, _, _) in &snapshot {
            if let Some(dn) = world.get_mut::<DamageNumber>(*e) {
                dn.age += dt;
                if dn.age >= dn.lifetime {
                    expired.push(*e);
                    continue;
                }
            }
            if let Some(t) = world.get_mut::<Transform>(*e) {
                // 위로 떠오름 — 초기엔 빠르고 점점 느려지는 ease-out 느낌
                t.position.y -= 40.0 * dt;
            }
        }

        // 3) 만료 despawn
        for e in expired {
            world.despawn(e);
        }
    }
}

// ─── 단위 테스트 ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fade_progresses_from_zero_to_one() {
        let mut dn = DamageNumber { value: 10.0, age: 0.0, lifetime: 1.0 };
        assert_eq!(dn.fade(), 0.0);
        dn.age = 0.5;
        assert_eq!(dn.fade(), 0.5);
        dn.age = 1.0;
        assert_eq!(dn.fade(), 1.0);
        dn.age = 2.0; // 수명 초과 → 1.0 로 clamp
        assert_eq!(dn.fade(), 1.0);
    }

    #[test]
    fn spawn_creates_entity_with_components() {
        let mut world = World::new();
        let e = spawn_damage_number(&mut world, Vec2::new(100.0, 200.0), 25.0);
        assert!(world.get::<DamageNumber>(e).is_some());
        let dn = world.get::<DamageNumber>(e).unwrap();
        assert_eq!(dn.value, 25.0);
        assert_eq!(dn.age, 0.0);

        let t = world.get::<Transform>(e).unwrap();
        // jitter 가 있어도 y 는 정확히 200 - 16 = 184
        assert_eq!(t.position.y, 184.0);
        // x 는 100 ± 8 범위 안
        assert!((t.position.x - 100.0).abs() <= 8.0);
    }

    #[test]
    fn system_moves_up_and_despawns_after_lifetime() {
        let mut world = World::new();
        let e = spawn_damage_number(&mut world, Vec2::new(0.0, 100.0), 10.0);
        let initial_y = world.get::<Transform>(e).unwrap().position.y;

        let mut sys = DamageNumberSystem;
        sys.run(&mut world, 0.1); // age = 0.1, 위로 4px 이동

        let y_after = world.get::<Transform>(e).unwrap().position.y;
        assert!(y_after < initial_y, "should move up");

        // 충분히 큰 dt 로 한 번에 수명 초과 → despawn
        sys.run(&mut world, 1.0);
        assert!(world.get::<DamageNumber>(e).is_none(), "should be despawned");
    }
}
