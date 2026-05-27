//! 스프라이트 기반 파티클 시스템 — Phase 11-E.
//!
//! ## 설계
//! - `Particle` 컴포넌트 + `Transform` + `Sprite` 로 렌더링은 SpriteRenderer 가 처리
//! - `ParticleSystem` 이 매 프레임 age 증가 + 위치 이동 + 색상 페이드 + 만료 despawn
//! - `spawn_burst()` / `spawn_collect_burst()` 헬퍼로 간단하게 이펙트 생성
//!
//! ## borrow checker 패턴
//! query 의 불변 borrow 와 get_mut 의 가변 borrow 충돌 방지 →
//! 읽기 패스(collect) 후 쓰기 패스(get_mut/despawn) 로 분리.

use engine::{Entity, Sprite, System, Transform, World};
use glam::Vec2;
use rand::Rng;

/// 파티클 동작 컴포넌트. `Transform` + `Sprite` 와 함께 동작.
#[derive(Debug, Clone, Copy)]
pub struct Particle {
    pub velocity: Vec2,        // px/s
    pub age: f32,              // 현재 경과 시간
    pub lifetime: f32,         // 총 수명 (초)
    pub size: f32,             // 초기 크기(px) — 스케일 계산 기준
    pub color_start: [f32; 4], // 생성 직후 색
    pub color_end: [f32; 4],   // 수명 만료 직전 색 (보통 투명)
}

impl Particle {
    fn t_norm(&self) -> f32 {
        (self.age / self.lifetime).clamp(0.0, 1.0)
    }
    fn lerp_color(&self) -> [f32; 4] {
        let t = self.t_norm();
        [
            self.color_start[0] + (self.color_end[0] - self.color_start[0]) * t,
            self.color_start[1] + (self.color_end[1] - self.color_start[1]) * t,
            self.color_start[2] + (self.color_end[2] - self.color_start[2]) * t,
            self.color_start[3] + (self.color_end[3] - self.color_start[3]) * t,
        ]
    }
}

/// 파티클 업데이트 시스템. DamageNumberSystem 과 함께 HudSystem 직전에 등록.
#[derive(Default)]
pub struct ParticleSystem {
    snapshot: Vec<(Entity, f32)>,
    expired: Vec<Entity>,
}

impl System for ParticleSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        // 읽기 패스
        self.snapshot.clear();
        self.expired.clear();
        self.snapshot
            .extend(world.query::<Particle>().map(|(e, p)| (e, p.age)));

        for (e, age) in self.snapshot.drain(..) {
            let new_age = age + dt;
            let (lifetime, vel, size, color) = if let Some(p) = world.get_mut::<Particle>(e) {
                p.age = new_age;
                (p.lifetime, p.velocity, p.size, p.lerp_color())
            } else {
                continue;
            };

            if new_age >= lifetime {
                self.expired.push(e);
            } else {
                if let Some(t) = world.get_mut::<Transform>(e) {
                    t.position += vel * dt;
                    // size 기준으로 스케일 축소 — 원본 크기에서 0까지 줄어드는 느낌
                    let s = 1.0 - (new_age / lifetime).clamp(0.0, 1.0);
                    t.scale = Vec2::splat((s * size).max(1.0));
                }
                if let Some(sp) = world.get_mut::<Sprite>(e) {
                    sp.color = color;
                }
            }
        }

        for e in self.expired.drain(..) {
            world.despawn(e);
        }
    }
}

// ─── 스폰 헬퍼 ───────────────────────────────────────────────────────────────

/// 적 사망 이펙트: 색상 파티클 폭발.
///
/// `color` 는 적의 스프라이트 색 (적 종류별 다름).
pub fn spawn_death_burst(world: &mut World, pos: Vec2, color: [f32; 4]) {
    let mut rng = rand::thread_rng();
    for _ in 0..8 {
        let angle = rng.gen_range(0.0f32..std::f32::consts::TAU);
        let speed = rng.gen_range(60.0..150.0);
        let vel = Vec2::new(angle.cos(), angle.sin()) * speed;
        let life = rng.gen_range(0.6..1.0);
        let size = rng.gen_range(10.0..18.0);
        spawn_particle(
            world,
            pos,
            vel,
            life,
            size,
            color,
            [color[0], color[1], color[2], 0.0],
        );
    }
}

/// XP 젬 수집 이펙트: 밝고 작은 황금빛 파티클.
pub fn spawn_collect_burst(world: &mut World, pos: Vec2) {
    let mut rng = rand::thread_rng();
    for _ in 0..4 {
        let angle = rng.gen_range(0.0f32..std::f32::consts::TAU);
        let speed = rng.gen_range(40.0..90.0);
        let vel = Vec2::new(angle.cos(), angle.sin()) * speed;
        let life = rng.gen_range(0.4..0.7);
        spawn_particle(
            world,
            pos,
            vel,
            life,
            8.0,
            [1.0, 0.9, 0.2, 1.0], // 황금빛
            [1.0, 0.9, 0.2, 0.0],
        );
    }
}

/// 파티클 엔티티 1개 스폰. 내부 공통 헬퍼.
fn spawn_particle(
    world: &mut World,
    pos: Vec2,
    velocity: Vec2,
    lifetime: f32,
    size: f32,
    color_start: [f32; 4],
    color_end: [f32; 4],
) {
    let e = world.spawn();
    world.add_component(
        e,
        Transform {
            position: pos,
            scale: Vec2::splat(size),
            rotation: 0.0,
            z: 2.0, // 플레이어(1.0) 위에 그려지도록
        },
    );
    world.add_component(
        e,
        Sprite {
            texture: None,
            color: color_start,
            image_handle: None,
            normal_texture: None,
            normal_handle: None,
        },
    );
    world.add_component(
        e,
        Particle {
            velocity,
            age: 0.0,
            lifetime,
            size,
            color_start,
            color_end,
        },
    );
}

// ─── 단위 테스트 ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn particle_lerp_color_at_start_and_end() {
        let p = Particle {
            velocity: Vec2::ZERO,
            age: 0.0,
            lifetime: 1.0,
            size: 10.0,
            color_start: [1.0, 0.0, 0.0, 1.0],
            color_end: [0.0, 0.0, 0.0, 0.0],
        };
        let c = p.lerp_color();
        assert_eq!(c, [1.0, 0.0, 0.0, 1.0]); // age=0 → start

        let p2 = Particle { age: 1.0, ..p };
        let c2 = p2.lerp_color();
        assert_eq!(c2, [0.0, 0.0, 0.0, 0.0]); // age=lifetime → end
    }

    #[test]
    fn particle_system_moves_and_despawns() {
        let mut world = World::new();
        let e = world.spawn();
        world.add_component(
            e,
            Transform {
                position: Vec2::ZERO,
                scale: Vec2::ONE,
                rotation: 0.0,
                z: 0.0,
            },
        );
        world.add_component(
            e,
            Sprite {
                texture: None,
                color: [1.0; 4],
                image_handle: None,
            },
        );
        world.add_component(
            e,
            Particle {
                velocity: Vec2::new(100.0, 0.0),
                age: 0.0,
                lifetime: 0.2,
                size: 10.0,
                color_start: [1.0; 4],
                color_end: [0.0; 4],
            },
        );

        let mut sys = ParticleSystem::default();
        sys.run(&mut world, 0.1); // age=0.1, 10px 이동
        let x = world.get::<Transform>(e).unwrap().position.x;
        assert!((x - 10.0).abs() < 0.01, "should move 10px in 0.1s");

        sys.run(&mut world, 0.2); // age=0.3 > lifetime=0.2 → despawn
        assert!(world.get::<Particle>(e).is_none(), "should be despawned");
    }
}
