//! Phase 5: 보스 3종 + 멀티 페이즈 + StageClear.
//!
//! - BossKind: GiantSlime(10min/HP1000) / GhostKing(20min/HP2500) / Death(30min/HP6000)
//! - BossSpawnSystem: GameStats.elapsed >= spawn_time 시 스폰
//! - BossPhaseSystem: HP 50%/20% 미만 시 phase 전환 + 속도 강화
//! - BossDeathSystem: HP <= 0 → despawn + Death 처치 시 StageClear
//! - CameraShake: duration/intensity 기반 랜덤 offset 리소스
//! - StageProgress: cleared 플래그

use super::enemy::{Enemy, EnemyAi, EnemyAiKind, EnemyKind};
use super::health::Health;
use super::hud::GameStats;
use super::locale::{loc, Lang};
use super::player::Player;
use super::sprites::{add_sprite, SurvivorSprite};
use super::LAYER_ENEMY;
use engine::components::GameState;
use engine::{Collider, CollisionLayer, System, Transform, World};
use glam::Vec2;

// ─── BossKind ────────────────────────────────────────────────────────────────

/// 보스 종류 — 등장 시간 + 기본 스탯의 키.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BossKind {
    GiantSlime, // 10:00 등장. HP 1000.
    GhostKing,  // 20:00 등장. HP 2500.
    Death,      // 30:00 등장. HP 6000. 처치 시 StageClear.
}

impl BossKind {
    pub fn spawn_time(self) -> f32 {
        match self {
            BossKind::GiantSlime => 600.0, // 10분
            BossKind::GhostKing => 1200.0, // 20분
            BossKind::Death => 1800.0,     // 30분
        }
    }

    pub fn max_hp(self) -> f32 {
        match self {
            BossKind::GiantSlime => 1000.0,
            BossKind::GhostKing => 2500.0,
            BossKind::Death => 6000.0,
        }
    }

    pub fn color(self) -> [f32; 3] {
        match self {
            BossKind::GiantSlime => [0.2, 0.6, 0.8],
            BossKind::GhostKing => [0.7, 0.7, 1.0],
            BossKind::Death => [0.5, 0.0, 0.0],
        }
    }

    /// 화면상 크기(픽셀). HP 가 클수록 크지만 최대 +40px.
    pub fn scale(self) -> f32 {
        80.0 + (self.max_hp() / 100.0).min(40.0)
    }

    pub fn label(self, lang: Lang) -> &'static str {
        match self {
            BossKind::GiantSlime => loc(lang, "자이언트 슬라임", "GIANT SLIME"),
            BossKind::GhostKing => loc(lang, "고스트 킹", "GHOST KING"),
            BossKind::Death => loc(lang, "죽음", "DEATH"),
        }
    }
}

// ─── Boss 컴포넌트 ────────────────────────────────────────────────────────────

/// 보스 컴포넌트. multi-phase 트래킹.
#[derive(Debug, Clone)]
pub struct Boss {
    pub kind: BossKind,
    /// 현재 페이즈. 0=full HP, 1=50% 미만, 2=20% 미만.
    pub phase: u8,
    pub move_speed: f32,
    pub spawn_time: f32,
    pub spawned: bool,
}

// ─── BossSpawnQueue ───────────────────────────────────────────────────────────

/// 아직 등장하지 않은 보스 목록.
#[derive(Debug, Clone)]
pub struct BossSpawnQueue {
    pub pending: Vec<BossKind>,
}

impl Default for BossSpawnQueue {
    fn default() -> Self {
        Self {
            pending: vec![BossKind::GiantSlime, BossKind::GhostKing, BossKind::Death],
        }
    }
}

// ─── CameraShake 리소스 ───────────────────────────────────────────────────────

/// 화면 흔들기 리소스.
/// CameraFollowSystem 이 매 프레임 elapsed 를 증가시키고, is_active() 이면 카메라에 오프셋 적용.
#[derive(Debug, Default)]
pub struct CameraShake {
    pub elapsed: f32,
    pub duration: f32,
    pub intensity: f32,
}

impl CameraShake {
    pub fn trigger(&mut self, duration: f32, intensity: f32) {
        self.elapsed = 0.0;
        self.duration = duration;
        self.intensity = intensity;
    }

    pub fn is_active(&self) -> bool {
        self.elapsed < self.duration
    }
}

// ─── StageProgress 리소스 ────────────────────────────────────────────────────

/// StageClear 진행 상태.
#[derive(Debug, Default)]
pub struct StageProgress {
    pub cleared: bool,
}

// ─── spawn_boss ───────────────────────────────────────────────────────────────

/// 지정 위치에 보스 엔티티를 스폰한다.
pub fn spawn_boss(world: &mut World, pos: Vec2, kind: BossKind) {
    let e = world.spawn();
    let scale = kind.scale();
    world.add_component(
        e,
        Transform {
            position: pos,
            scale: Vec2::new(scale, scale),
            rotation: 0.0,
            z: 0.6, // 일반 적(0.5) 보다 위
        },
    );
    let sprite = match kind {
        BossKind::GiantSlime => SurvivorSprite::GiantSlime,
        BossKind::GhostKing => SurvivorSprite::GhostKing,
        BossKind::Death => SurvivorSprite::Death,
    };
    add_sprite(world, e, sprite);
    world.add_component(
        e,
        Boss {
            kind,
            phase: 0,
            move_speed: 60.0,
            spawn_time: kind.spawn_time(),
            spawned: true,
        },
    );
    // Enemy 컴포넌트 — apply_damage_to_enemy 호환을 위해 부착.
    world.add_component(
        e,
        Enemy {
            kind: EnemyKind::Knight,
            contact_damage: 25.0,
            split_remaining: 0,
            is_elite: false,
        },
    );
    world.add_component(
        e,
        EnemyAi {
            move_speed: 60.0,
            ai: EnemyAiKind::Chase,
        },
    );
    world.add_component(e, Health::new(kind.max_hp()));
    world.add_component(e, CollisionLayer(LAYER_ENEMY));
    world.add_component(
        e,
        Collider::Circle {
            radius: scale / 2.0,
        },
    );
}

// ─── BossSpawnSystem ──────────────────────────────────────────────────────────

/// GameStats.elapsed 가 보스 등장 시간에 도달하면 플레이어 위쪽 200px 에서 스폰.
pub struct BossSpawnSystem;

impl System for BossSpawnSystem {
    fn run(&mut self, world: &mut World, _dt: f32) {
        if !matches!(world.resource::<GameState>(), Some(GameState::Playing)) {
            return;
        }

        let elapsed = world
            .resource::<GameStats>()
            .map(|s| s.elapsed)
            .unwrap_or(0.0);
        let player_pos = world
            .query2::<Player, Transform>()
            .next()
            .map(|(_, _, t)| t.position);
        let Some(player_pos) = player_pos else { return };

        // pending 목록 중 elapsed 이상인 보스 추출 (borrow 분리)
        let to_spawn: Vec<BossKind> = {
            let queue = match world.resource_mut::<BossSpawnQueue>() {
                Some(q) => q,
                None => return,
            };
            let triggered: Vec<BossKind> = queue
                .pending
                .iter()
                .filter(|b| elapsed >= b.spawn_time())
                .copied()
                .collect();
            queue.pending.retain(|b| elapsed < b.spawn_time());
            triggered
        };

        for kind in to_spawn {
            // 플레이어 위쪽 200px 에서 등장
            spawn_boss(world, player_pos + Vec2::new(0.0, -200.0), kind);
            println!("BOSS APPROACHES: {}", kind.label(Lang::En));
        }
    }
}

// ─── BossPhaseSystem ─────────────────────────────────────────────────────────

/// HP 비율에 따라 페이즈 전환. phase 가 오를 때 move_speed × 1.4, contact_damage × 1.3.
pub struct BossPhaseSystem;

impl System for BossPhaseSystem {
    fn run(&mut self, world: &mut World, _dt: f32) {
        if !matches!(world.resource::<GameState>(), Some(GameState::Playing)) {
            return;
        }

        // 전환이 필요한 (entity, new_phase, kind) 목록 수집 (불변 query 종료 후 처리)
        let updates: Vec<(engine::Entity, u8, BossKind)> = world
            .query3::<Boss, Health, Transform>()
            .filter_map(|(e, b, h, _)| {
                let ratio = if h.max > 0.0 { h.current / h.max } else { 0.0 };
                let target_phase = if ratio < 0.20 {
                    2
                } else if ratio < 0.50 {
                    1
                } else {
                    0
                };
                if target_phase > b.phase {
                    Some((e, target_phase, b.kind))
                } else {
                    None
                }
            })
            .collect();

        for (entity, new_phase, kind) in updates {
            if let Some(b) = world.get_mut::<Boss>(entity) {
                b.phase = new_phase;
                b.move_speed *= 1.4;
            }
            if let Some(en) = world.get_mut::<Enemy>(entity) {
                en.contact_damage *= 1.3;
            }
            if let Some(ai) = world.get_mut::<EnemyAi>(entity) {
                ai.move_speed *= 1.4;
            }
            println!("{} entered phase {}", kind.label(Lang::En), new_phase);

            // 페이즈 전환 시 카메라 흔들기
            if let Some(shake) = world.resource_mut::<CameraShake>() {
                shake.trigger(0.4, 12.0);
            }
        }
    }
}

// ─── BossDeathSystem ─────────────────────────────────────────────────────────

/// 보스 HP <= 0 → despawn + CameraShake + kills 카운트.
/// Death 보스 처치 시 StageProgress.cleared = true.
pub struct BossDeathSystem;

impl System for BossDeathSystem {
    fn run(&mut self, world: &mut World, _dt: f32) {
        if !matches!(world.resource::<GameState>(), Some(GameState::Playing)) {
            return;
        }

        // 사망한 보스 목록 수집
        let dead_bosses: Vec<(engine::Entity, BossKind)> = world
            .query2::<Boss, Health>()
            .filter(|(_, _, h)| h.current <= 0.0)
            .map(|(e, b, _)| (e, b.kind))
            .collect();

        for (e, kind) in dead_bosses {
            // 사망 직전 위치 캐치 (despawn 전에 읽어야 함)
            let pos = world.get::<Transform>(e).map(|t| t.position);

            println!("BOSS DEFEATED: {}", kind.label(Lang::En));
            world.despawn(e);

            // Phase 7: Vacuum + Bomb 드롭
            if let Some(p) = pos {
                crate::survivor::pickup::drop_boss_pickups(world, p);
            }

            // 카메라 강하게 흔들기
            if let Some(shake) = world.resource_mut::<CameraShake>() {
                shake.trigger(0.8, 20.0);
            }

            // kills 카운트 +1
            if let Some(stats) = world.resource_mut::<GameStats>() {
                stats.kills += 1;
            }

            // Death 보스 처치 → StageClear
            if matches!(kind, BossKind::Death) {
                if let Some(progress) = world.resource_mut::<StageProgress>() {
                    progress.cleared = true;
                }
                println!("STAGE CLEAR!");
            }
        }
    }
}
