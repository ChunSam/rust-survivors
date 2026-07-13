use engine::{Entity, GameState, InputState, KeyCode, System, Transform, World};
use glam::Vec2;

/// 플레이어 태그 컴포넌트
pub struct Player;

/// 이동 속도(픽셀/초). System 이 매 프레임 Transform 에 누적.
pub struct Velocity(pub Vec2);

/// 캐릭터 스탯. Phase 3-A 에서 16 필드로 확장.
///
/// 기본값이 항등원(might=0, area=1.0 등)이므로 패시브 없이도 게임플레이 변화 없음.
/// Phase 3-B 에서 StatRecalcSystem 이 패시브 합산 후 이 값들을 덮어씀.
#[derive(Debug, Clone, PartialEq)]
pub struct PlayerStats {
    // 공격 관련
    pub might: f32,            // 데미지 가산 (0.0 기본 — damage += might)
    pub area: f32,             // hitbox/오라 반경/AABB 너비 곱 (1.0 기본)
    pub projectile_speed: f32, // 투사체 속도 곱 (1.0 기본)
    pub duration: f32,         // 투사체/필드 lifetime 곱 (1.0 기본)
    pub amount: i32,           // 발사 수 가산 (0 기본 — Knife/HolyWater/KingBible/Lightning 에 +)
    pub cooldown: f32,         // cooldown 곱 (1.0 기본, < 1 이면 빨라짐)

    // 방어/이동
    pub max_health: f32, // 최대 체력. 기본 100.
    pub recovery: f32,   // 초당 자연 회복 (0.0 기본)
    pub armor: f32,      // 받는 데미지 감쇄 (0.0 기본)
    pub move_speed: f32, // 이동 속도 px/s (200.0 기본)

    // 픽업/보조
    pub magnet: f32,  // 자석 반경 곱 (1.0 기본)
    pub luck: f32,    // 카드 슬롯/드롭률 placeholder (1.0 기본)
    pub growth: f32,  // XP 획득 배율 (1.0 기본)
    pub greed: f32,   // 골드 획득 배율 placeholder (1.0 기본)
    pub curse: f32,   // 적 강화 placeholder (1.0 기본)
    pub revival: u32, // 사망 시 부활 횟수 placeholder (0 기본)
}

impl Default for PlayerStats {
    fn default() -> Self {
        Self {
            might: 0.0,
            area: 1.0,
            projectile_speed: 1.0,
            duration: 1.0,
            amount: 0,
            cooldown: 1.0,
            max_health: 100.0,
            recovery: 0.0,
            armor: 0.0,
            move_speed: 200.0,
            magnet: 1.0,
            luck: 1.0,
            growth: 1.0,
            greed: 1.0,
            curse: 1.0,
            revival: 0,
        }
    }
}

/// WASD/방향키 → Velocity → Transform.position 갱신.
#[derive(Default)]
pub struct PlayerMovementSystem {
    targets: Vec<(Entity, f32)>,
}

impl System for PlayerMovementSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        if !matches!(world.resource::<GameState>(), Some(GameState::Playing)) {
            return;
        }

        // 1. 입력 읽기 — 정규화된 방향 벡터 계산
        let mut dir = Vec2::ZERO;
        if let Some(input) = world.resource::<InputState>() {
            if input.is_pressed(KeyCode::KeyA) || input.is_pressed(KeyCode::ArrowLeft) {
                dir.x -= 1.0;
            }
            if input.is_pressed(KeyCode::KeyD) || input.is_pressed(KeyCode::ArrowRight) {
                dir.x += 1.0;
            }
            if input.is_pressed(KeyCode::KeyW) || input.is_pressed(KeyCode::ArrowUp) {
                dir.y -= 1.0; // y 작을수록 위
            }
            if input.is_pressed(KeyCode::KeyS) || input.is_pressed(KeyCode::ArrowDown) {
                dir.y += 1.0;
            }
        }
        let dir = if dir.length_squared() > 0.0 {
            dir.normalize()
        } else {
            Vec2::ZERO
        };

        // 2. Player + PlayerStats + Transform 모두 가진 엔티티에 적용
        // 두 단계로 분리(query 결과를 모은 뒤 mut 접근)해서 borrow checker 회피
        self.targets.clear();
        self.targets.extend(
            world
                .query2::<Player, PlayerStats>()
                .map(|(e, _, stats)| (e, stats.move_speed)),
        );

        for (entity, speed) in self.targets.drain(..) {
            if let Some(t) = world.get_mut::<Transform>(entity) {
                t.position += dir * speed * dt;
            }
        }
    }
}
