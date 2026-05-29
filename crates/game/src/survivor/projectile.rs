use engine::{Collider, CollisionLayer, Entity, GameState, SpatialGrid, System, Transform, World};
use glam::Vec2;

use super::combat::apply_damage_events;
use super::enemy::Enemy;
use super::sprites::{add_sprite, SurvivorSprite};
use super::LAYER_ENEMY;

type ProjectileSnapshot = (
    Entity,
    Vec2,
    Vec2,
    f32,
    u8,
    f32,
    CollisionLayer,
    f32,
    ProjectileBehavior,
);
type ProjectileUpdate = (Entity, Vec2, f32, u8, Vec2, ProjectileBehavior);

/// 투사체 이동 방식.
///
/// - `Straight`: 매 프레임 velocity 그대로 이동 (직선)
/// - `Arc`: velocity.y 가 매 프레임 += gravity * dt (포물선). 도끼처럼 위로 던져 중력으로 떨어질 때 사용.
/// - `Boomerang`: `elapsed` 가 `return_at` 에 도달하면 velocity 반전 (부메랑). `returned` 플래그로 한 번만 반전.
///
/// ## Y 축 방향 주의
/// 렌더 좌표계는 `(0,0) = 좌상단, y 클수록 아래`. 따라서:
/// - "위로 던지기" = `velocity.y < 0` (음수)
/// - "중력" = `gravity > 0` (매 프레임 velocity.y 증가 → 결국 아래로)
#[derive(Debug, Clone, Copy)]
pub enum ProjectileBehavior {
    Straight,
    Arc {
        gravity: f32,
    },
    /// 부메랑 동작. `elapsed` 가 `return_at` 초에 도달하면 velocity 반전.
    /// `returned = true` 이면 이미 반전 완료 — 다시 반전하지 않는다.
    Boomerang {
        return_at: f32,
        elapsed: f32,
        returned: bool,
    },
}

/// 투사체 컴포넌트. velocity 단위는 px/s.
#[derive(Debug, Clone)]
pub struct Projectile {
    pub velocity: Vec2,
    pub lifetime: f32, // 남은 수명 (초). 0 이하면 despawn
    pub pierce: u8,    // 남은 관통 횟수. 0 이면 다음 hit 후 despawn
    pub damage: f32,
    pub layer_mask: CollisionLayer,   // 어느 레이어를 타격할지
    pub radius: f32,                  // 충돌 판정 반경 (px)
    pub behavior: ProjectileBehavior, // 이동 방식 (직선 / 포물선)
}

/// 모든 Projectile 의 lifetime / 이동 / 충돌 일괄 처리.
///
/// SpatialGrid 를 직접 소유 — WhipSystem / EnemyContactDamageSystem 패턴 답습.
/// borrow checker 충돌을 피하기 위해 ECS 리소스로 넣지 않는다.
pub struct ProjectileSystem {
    pub grid: SpatialGrid,
    projectiles: Vec<ProjectileSnapshot>,
    to_despawn: Vec<Entity>,
    hits_buffer: Vec<(Entity, f32)>,
    proj_updates: Vec<ProjectileUpdate>,
}

impl Default for ProjectileSystem {
    fn default() -> Self {
        Self {
            grid: SpatialGrid::new(128.0),
            projectiles: Vec::new(),
            to_despawn: Vec::new(),
            hits_buffer: Vec::new(),
            proj_updates: Vec::new(),
        }
    }
}

impl System for ProjectileSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        if !matches!(world.resource::<GameState>(), Some(GameState::Playing)) {
            return;
        }

        // grid rebuild — 적(Zombie) 위치 기준으로 쿼리하기 위해 재구성
        self.grid.rebuild(world);

        self.projectiles.clear();
        self.to_despawn.clear();
        self.hits_buffer.clear();
        self.proj_updates.clear();

        // 1) 모든 Projectile 의 파라미터를 복사해 캐시 (borrow 끊기)
        self.projectiles
            .extend(world.query2::<Projectile, Transform>().map(|(e, p, t)| {
                (
                    e,
                    t.position,
                    p.velocity,
                    p.lifetime,
                    p.pierce,
                    p.damage,
                    p.layer_mask,
                    p.radius,
                    p.behavior,
                )
            }));

        for (proj_e, pos, vel, life, pierce, damage, mask, radius, behavior) in
            self.projectiles.drain(..)
        {
            // behavior 에 따라 velocity 갱신 (Arc 는 중력, Boomerang 은 반전)
            let mut new_velocity = vel;
            let mut new_behavior = behavior;
            match new_behavior {
                ProjectileBehavior::Straight => {}
                ProjectileBehavior::Arc { gravity } => {
                    new_velocity.y += gravity * dt;
                }
                ProjectileBehavior::Boomerang {
                    return_at,
                    ref mut elapsed,
                    ref mut returned,
                } => {
                    *elapsed += dt;
                    if !*returned && *elapsed >= return_at {
                        new_velocity = -new_velocity;
                        *returned = true;
                    }
                }
            }
            let new_pos = pos + new_velocity * dt;
            let new_life = life - dt;

            if new_life <= 0.0 {
                self.to_despawn.push(proj_e);
                continue;
            }

            // 투사체 충돌 반경으로 적 탐색
            let candidates = self.grid.query_radius(new_pos, radius, mask);

            let mut new_pierce = pierce;
            let mut consumed = false;

            for cand in candidates {
                // Enemy 컴포넌트를 가진 엔티티인지 확인 (모든 적 종류 처리)
                let is_enemy = world.get::<Enemy>(cand).is_some();
                if !is_enemy {
                    continue;
                }
                self.hits_buffer.push((cand, damage));
                if new_pierce == 0 {
                    // 관통 횟수 소진 → 투사체 제거 예약
                    consumed = true;
                    break;
                }
                new_pierce -= 1;
            }

            if consumed {
                self.to_despawn.push(proj_e);
            } else {
                self.proj_updates.push((
                    proj_e,
                    new_pos,
                    new_life,
                    new_pierce,
                    new_velocity,
                    new_behavior,
                ));
            }
        }

        // 2) 투사체 위치·수명·관통·속도·behavior 갱신 (behavior 는 Boomerang elapsed/returned 상태 포함)
        for (proj_e, new_pos, new_life, new_pierce, new_velocity, new_behavior) in
            self.proj_updates.drain(..)
        {
            if let Some(t) = world.get_mut::<Transform>(proj_e) {
                t.position = new_pos;
            }
            if let Some(p) = world.get_mut::<Projectile>(proj_e) {
                p.lifetime = new_life;
                p.pierce = new_pierce;
                p.velocity = new_velocity;
                p.behavior = new_behavior;
            }
        }

        // 3) 데미지 적용 + 사망 처리 + XpGem 드롭 + HitFlash + 처치 카운터
        apply_damage_events(world, self.hits_buffer.drain(..));

        // 4) 수명 소진·관통 소진 투사체 despawn
        for proj_e in self.to_despawn.drain(..) {
            world.despawn(proj_e);
        }
    }
}

/// 투사체 엔티티를 스폰한다 (`ProjectileBehavior::Straight` 기본).
///
/// 모든 무기 시스템(MagicWandSystem 등)이 이 헬퍼를 통해 투사체를 생성한다.
/// 포물선 동작이 필요하면 `spawn_projectile_ex` 를 사용한다.
///
/// # 파라미터
/// - `pos`      : 발사 위치 (px)
/// - `velocity` : 초당 이동량 (px/s), 방향 + 속력 포함
/// - `lifetime` : 수명 (초)
/// - `pierce`   : 관통 횟수 (0 = 1마리 타격 후 소멸)
/// - `damage`   : 타격 데미지
/// - `color`    : RGB 색 (Sprite::colored 에 전달)
pub fn spawn_projectile(
    world: &mut World,
    pos: Vec2,
    velocity: Vec2,
    lifetime: f32,
    pierce: u8,
    damage: f32,
    color: [f32; 3],
) {
    spawn_projectile_ex(
        world,
        pos,
        velocity,
        lifetime,
        pierce,
        damage,
        color,
        ProjectileBehavior::Straight,
    );
}

/// `ProjectileBehavior` 를 명시적으로 지정하는 투사체 스폰 헬퍼.
///
/// `spawn_projectile` 이 내부적으로 이 함수를 `Straight` 로 호출한다.
/// `Arc { gravity }` 를 전달하면 포물선(도끼 등) 투사체를,
/// `Boomerang { return_at, elapsed: 0.0, returned: false }` 를 전달하면 부메랑 투사체를 만들 수 있다.
///
/// # 파라미터
/// - `pos`      : 발사 위치 (px)
/// - `velocity` : 초당 이동량 (px/s), 방향 + 속력 포함
/// - `lifetime` : 수명 (초)
/// - `pierce`   : 관통 횟수 (0 = 1마리 타격 후 소멸)
/// - `damage`   : 타격 데미지
/// - `color`    : RGB 색 (Sprite::colored 에 전달)
/// - `behavior` : 이동 방식 (`Straight`, `Arc { gravity }`, `Boomerang { return_at, elapsed, returned }`)
#[allow(clippy::too_many_arguments)]
pub fn spawn_projectile_ex(
    world: &mut World,
    pos: Vec2,
    velocity: Vec2,
    lifetime: f32,
    pierce: u8,
    damage: f32,
    color: [f32; 3],
    behavior: ProjectileBehavior,
) {
    let e = world.spawn();
    world.add_component(
        e,
        Transform {
            position: pos,
            scale: Vec2::new(10.0, 10.0),
            rotation: 0.0,
            z: 0.4,
        },
    );
    let sprite = projectile_sprite(color, behavior);
    add_sprite(world, e, sprite);
    world.add_component(
        e,
        Projectile {
            velocity,
            lifetime,
            pierce,
            damage,
            layer_mask: CollisionLayer(LAYER_ENEMY),
            radius: 6.0,
            behavior,
        },
    );
    // 충돌 그리드에 올리기 위한 Collider (ProjectileSystem 이 query_radius 로 detect)
    world.add_component(e, Collider::Circle { radius: 6.0 });
}

fn projectile_sprite(color: [f32; 3], behavior: ProjectileBehavior) -> SurvivorSprite {
    match behavior {
        ProjectileBehavior::Arc { .. } => SurvivorSprite::Axe,
        ProjectileBehavior::Boomerang { .. } => SurvivorSprite::CrossProjectile,
        ProjectileBehavior::Straight => {
            if color[0] > 0.75 && color[1] > 0.75 {
                SurvivorSprite::MagicBolt
            } else if color[0] > 0.7 && color[1] < 0.5 {
                SurvivorSprite::Fireball
            } else {
                SurvivorSprite::Knife
            }
        }
    }
}

// ─── 단위 테스트 ──────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::super::health::Health;
    use super::super::hud::GameStats;
    use super::super::spawn::spawn_zombie;
    use super::*;
    use engine::{GameState, World};
    use glam::Vec2;

    fn playing_world() -> World {
        let mut world = World::new();
        world.insert_resource(GameState::Playing);
        world
    }

    /// 수명이 다한 Projectile 은 despawn 돼야 한다.
    #[test]
    fn projectile_expires_after_lifetime() {
        let mut world = playing_world();

        // lifetime 0.5초 투사체 스폰
        spawn_projectile(
            &mut world,
            Vec2::ZERO,
            Vec2::new(100.0, 0.0),
            0.5,
            0,
            5.0,
            [1.0, 1.0, 1.0],
        );

        // dt = 1.0 → lifetime 소진
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
        let mut world = playing_world();
        world.insert_resource(GameStats::default());

        // 좀비를 (30, 0) 에 스폰 (HP 30)
        spawn_zombie(&mut world, Vec2::new(30.0, 0.0));

        // 투사체를 원점에서 +x 방향으로 발사 — dt=0.1 에 (10, 0) 이동
        // 좀비 Collider::Circle { radius:20 }, 투사체 radius:6
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
        let enemy_entity = world.query::<Enemy>().next().map(|(e, _)| e);
        if let Some(ze) = enemy_entity {
            let hp = world.get::<Health>(ze).map(|h| h.current).unwrap_or(0.0);
            assert!(hp < 30.0, "투사체 피격 후 좀비 HP 가 줄어야 함 (현재 {hp})");
        }
        // despawn 됐으면 테스트 통과 (HP 5 미만 -> 즉사 가능성도 있음)
    }
}
