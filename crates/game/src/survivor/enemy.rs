use engine::{Entity, System, Transform, World};
use engine::components::GameState;
use glam::Vec2;
use super::player::Player;

/// 적 종류. 시각화 + AI variant + 기본 스탯의 키.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnemyKind {
    Zombie,   // 직진 추격 (chase, 기존 Zombie)
    Bat,      // 빠른 chase + 약한 HP
    Ghost,    // chase 하다 일정 거리에서 멈춰 hover
    Skeleton, // chase + 약간 빠름
    Mage,     // 거리 유지 (kite — player 에서 일정 거리 미만이면 멀어짐, 그 외 chase)
    Mantis,   // dash — 일정 cooldown 마다 player 방향 짧게 가속
    Plant,    // 정지 — 안 움직임, HP 만 높음
    Slime,    // chase + split (사망 시 작은 슬라임 2마리 스폰)
    Mummy,    // 매우 느림 + 매우 단단함
    Knight,   // chase + 높은 HP + 큰 접촉 데미지
}

impl EnemyKind {
    /// 기본 HP / 이동속도 / sprite color / scale / 접촉 데미지.
    pub fn stats(self) -> EnemyStats {
        match self {
            EnemyKind::Zombie   => EnemyStats { hp: 30.0,  move_speed: 80.0,  color: [0.4, 0.7, 0.4],  scale: 40.0, contact_damage: 10.0 },
            EnemyKind::Bat      => EnemyStats { hp: 15.0,  move_speed: 140.0, color: [0.3, 0.2, 0.4],  scale: 30.0, contact_damage: 6.0  },
            EnemyKind::Ghost    => EnemyStats { hp: 25.0,  move_speed: 100.0, color: [0.8, 0.8, 1.0],  scale: 36.0, contact_damage: 8.0  },
            EnemyKind::Skeleton => EnemyStats { hp: 35.0,  move_speed: 110.0, color: [0.9, 0.9, 0.85], scale: 38.0, contact_damage: 9.0  },
            EnemyKind::Mage     => EnemyStats { hp: 40.0,  move_speed: 90.0,  color: [0.7, 0.3, 0.9],  scale: 38.0, contact_damage: 7.0  },
            EnemyKind::Mantis   => EnemyStats { hp: 45.0,  move_speed: 80.0,  color: [0.5, 0.9, 0.3],  scale: 42.0, contact_damage: 12.0 },
            EnemyKind::Plant    => EnemyStats { hp: 60.0,  move_speed: 0.0,   color: [0.3, 0.6, 0.2],  scale: 44.0, contact_damage: 8.0  },
            EnemyKind::Slime    => EnemyStats { hp: 30.0,  move_speed: 70.0,  color: [0.4, 0.8, 0.8],  scale: 36.0, contact_damage: 7.0  },
            EnemyKind::Mummy    => EnemyStats { hp: 120.0, move_speed: 40.0,  color: [0.8, 0.7, 0.5],  scale: 48.0, contact_damage: 15.0 },
            EnemyKind::Knight   => EnemyStats { hp: 80.0,  move_speed: 90.0,  color: [0.6, 0.6, 0.7],  scale: 44.0, contact_damage: 14.0 },
        }
    }
}

/// 각 EnemyKind 의 기본 스탯 묶음.
#[derive(Debug, Clone, Copy)]
pub struct EnemyStats {
    pub hp:             f32,
    pub move_speed:     f32,
    pub color:          [f32; 3],
    pub scale:          f32,
    pub contact_damage: f32,
}

/// 적 AI 행동 종류.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnemyAiKind {
    /// 단순 추격 (Zombie / Bat / Skeleton / Knight)
    Chase,
    /// 일정 거리에서 멈춤 (Ghost)
    Hover { stop_at: f32 },
    /// 거리 유지 (Mage) — 너무 가까우면 달아남, 멀면 추격
    Kite { min_dist: f32 },
    /// 짧은 dash 후 일반 이동 반복 (Mantis)
    Dash {
        cooldown:      f32,
        elapsed:       f32,
        dash_speed:    f32,
        dash_lifetime: f32,
        dashing:       f32,
    },
    /// 정지 (Plant)
    Stay,
    /// chase + 사망 시 split (Slime)
    Split,
}

/// 적 컴포넌트 — 종류·접촉 데미지·슬라임 split 카운터.
#[derive(Debug, Clone)]
pub struct Enemy {
    pub kind:            EnemyKind,
    pub contact_damage:  f32,
    /// Slime 만 사용. 남은 split 횟수(2 → 1회 split 후 0 → 더 이상 split 안 함).
    pub split_remaining: u8,
}

/// 적 AI 파라미터 컴포넌트.
#[derive(Debug, Clone)]
pub struct EnemyAi {
    pub move_speed: f32,
    pub ai:         EnemyAiKind,
}

impl Default for EnemyAi {
    fn default() -> Self {
        Self { move_speed: 80.0, ai: EnemyAiKind::Chase }
    }
}

/// 좀비 적 태그 컴포넌트 (하위 호환 유지).
pub struct Zombie;

/// 적 AI 처리 시스템.
///
/// EnemyAiKind 에 따라 이동 방식 분기:
/// - Chase / Split : 단순 추격
/// - Hover         : 일정 거리에서 멈춤
/// - Kite          : 너무 가까우면 달아남
/// - Dash          : 짧은 burst 추격 + 일반 이동 반복
/// - Stay          : 정지
pub struct EnemyAiSystem;

impl System for EnemyAiSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        if !matches!(world.resource::<GameState>(), Some(GameState::Playing)) {
            return;
        }

        // 1) 플레이어 위치 캐시 — borrow 를 여기서 끝냄
        let Some(player_pos) = world
            .query2::<Player, Transform>()
            .next()
            .map(|(_, _, t)| t.position)
        else {
            return;
        };

        // 2) 모든 적 엔티티 상태 수집 (불변 query 종료)
        let enemies: Vec<(Entity, Vec2, f32, EnemyAiKind)> = world
            .query3::<Enemy, Transform, EnemyAi>()
            .map(|(e, _, t, ai)| (e, t.position, ai.move_speed, ai.ai))
            .collect();

        // 3) 새 위치 + 갱신된 ai_kind 계산
        let mut updates: Vec<(Entity, Vec2, Option<EnemyAiKind>)> = Vec::new();

        for (e, pos, move_speed, ai_kind) in enemies {
            let to_player = player_pos - pos;
            let dist = to_player.length();
            let dir = if dist > 0.0 { to_player / dist } else { Vec2::ZERO };

            let (new_pos, new_ai) = match ai_kind {
                EnemyAiKind::Chase | EnemyAiKind::Split => {
                    (pos + dir * move_speed * dt, None)
                }
                EnemyAiKind::Hover { stop_at } => {
                    if dist > stop_at {
                        (pos + dir * move_speed * dt, None)
                    } else {
                        (pos, None)
                    }
                }
                EnemyAiKind::Kite { min_dist } => {
                    if dist < min_dist {
                        // 너무 가까움 — 반대 방향으로 이동
                        (pos - dir * move_speed * dt, None)
                    } else {
                        // 멀면 추격
                        (pos + dir * move_speed * dt, None)
                    }
                }
                EnemyAiKind::Dash {
                    cooldown,
                    elapsed,
                    dash_speed,
                    dash_lifetime,
                    dashing,
                } => {
                    let mut new_elapsed = elapsed;
                    let mut new_dashing = dashing;

                    let new_pos = if new_dashing > 0.0 {
                        new_dashing -= dt;
                        pos + dir * dash_speed * dt
                    } else {
                        new_elapsed += dt;
                        if new_elapsed >= cooldown {
                            new_elapsed -= cooldown;
                            new_dashing = dash_lifetime;
                        }
                        pos + dir * move_speed * dt
                    };

                    let new_ai = EnemyAiKind::Dash {
                        cooldown,
                        elapsed:       new_elapsed,
                        dash_speed,
                        dash_lifetime,
                        dashing:       new_dashing,
                    };
                    (new_pos, Some(new_ai))
                }
                EnemyAiKind::Stay => (pos, None),
            };

            updates.push((e, new_pos, new_ai));
        }

        // 4) Transform + EnemyAi 갱신 (query 끝난 뒤 get_mut — borrow checker 회피)
        for (e, new_pos, new_ai) in updates {
            if let Some(t) = world.get_mut::<Transform>(e) {
                t.position = new_pos;
            }
            if let Some(new_ai_kind) = new_ai {
                if let Some(ai_comp) = world.get_mut::<EnemyAi>(e) {
                    ai_comp.ai = new_ai_kind;
                }
            }
        }
    }
}
