//! Phase 4-B: 시간축 wave 정의 + SpawnDirector / SpawnDirectorSystem.
//!
//! `waves.ron` 을 컴파일 타임에 내장하여 런타임 파일 의존 없이 파싱한다.
//! 게임 진행 시간(`GameStats.elapsed`)을 기준으로 현재 wave 를 결정하고,
//! wave 의 `spawn_interval` 마다 weight 기반 random 으로 적 1마리를 스폰한다.
//! despawn_radius 밖 적 정리도 이 시스템이 담당한다 (기존 EnemySpawnSystem 역할 이전).

use engine::{Camera, System, World};
use engine::components::GameState;
use glam::Vec2;
use rand::Rng;
use serde::Deserialize;
use super::enemy::EnemyKind;
use super::hud::GameStats;
use super::spawn::spawn_enemy_elite;
use engine::Transform;

const WAVES_RON: &str = include_str!("../../../../assets/data/waves.ron");

/// waves.ron 의 개별 적 항목 (kind 문자열 + weight).
#[derive(Debug, Deserialize, Clone)]
pub struct EnemyEntry {
    pub kind:   String,
    pub weight: u32,
}

/// 스폰 패턴 (Phase 4-C).
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub enum SpawnPattern {
    Random,    // 외곽 random angle (각자 random)
    Line,      // 한 방향에서 일렬 (perp 60px 간격)
    Circle,    // 등간격 원형
    Surround,  // 등간격 원형 (count >= 4 권장)
    Swarm,     // 한 점 cluster ±20px
}

fn default_pattern() -> SpawnPattern { SpawnPattern::Random }
fn default_count_per_burst() -> u8 { 1 }
fn default_elite_chance() -> f32 { 0.0 }

/// 하나의 wave 정의. start_time ~ end_time 구간 동안 spawn_interval 마다 적 스폰.
#[derive(Debug, Deserialize, Clone)]
pub struct WaveDef {
    pub start_time:     f32,
    pub end_time:       f32,
    pub spawn_interval: f32,
    pub enemies:        Vec<EnemyEntry>,
    #[serde(default = "default_pattern")]
    pub pattern:         SpawnPattern,
    #[serde(default = "default_count_per_burst")]
    pub count_per_burst: u8,
    #[serde(default = "default_elite_chance")]
    pub elite_chance:    f32,
}

/// waves.ron 최상위 구조체.
#[derive(Debug, Deserialize, Clone)]
pub struct WavesFile {
    pub waves: Vec<WaveDef>,
}

/// 시간축 wave 정의 + wave 내부 cooldown 카운터를 보관하는 ECS 리소스.
///
/// `waves` 는 컴파일 타임에 `waves.ron` 에서 파싱.
/// `spawn_elapsed` 는 현재 wave 의 `spawn_interval` 기준 내부 cooldown.
pub struct SpawnDirector {
    pub waves:         Vec<WaveDef>,
    pub spawn_elapsed: f32,
}

impl Default for SpawnDirector {
    fn default() -> Self {
        let parsed: WavesFile = ron::from_str(WAVES_RON)
            .expect("assets/data/waves.ron 파싱 실패");
        Self {
            waves:         parsed.waves,
            spawn_elapsed: 0.0,
        }
    }
}

impl SpawnDirector {
    /// `game_elapsed` 에 해당하는 wave 를 반환.
    /// 모든 wave 구간이 끝났으면 마지막 wave 를 반복한다.
    pub fn current_wave(&self, game_elapsed: f32) -> Option<&WaveDef> {
        self.waves
            .iter()
            .find(|w| game_elapsed >= w.start_time && game_elapsed < w.end_time)
            .or_else(|| self.waves.last())
    }

    /// `wave.enemies` 에서 weight 기반 random 선택 → `EnemyKind` 반환.
    pub fn pick_enemy_from(wave: &WaveDef, rng: &mut impl Rng) -> Option<EnemyKind> {
        if wave.enemies.is_empty() {
            return None;
        }
        let total_weight: u32 = wave.enemies.iter().map(|e| e.weight).sum();
        if total_weight == 0 {
            return None;
        }
        let mut roll = rng.gen_range(0..total_weight);
        for entry in &wave.enemies {
            if roll < entry.weight {
                return enemy_kind_from_str(&entry.kind);
            }
            roll -= entry.weight;
        }
        None
    }
}

fn enemy_kind_from_str(s: &str) -> Option<EnemyKind> {
    match s {
        "Zombie"   => Some(EnemyKind::Zombie),
        "Bat"      => Some(EnemyKind::Bat),
        "Ghost"    => Some(EnemyKind::Ghost),
        "Skeleton" => Some(EnemyKind::Skeleton),
        "Mage"     => Some(EnemyKind::Mage),
        "Mantis"   => Some(EnemyKind::Mantis),
        "Plant"    => Some(EnemyKind::Plant),
        "Slime"    => Some(EnemyKind::Slime),
        "Mummy"    => Some(EnemyKind::Mummy),
        "Knight"   => Some(EnemyKind::Knight),
        _ => None,
    }
}

/// 매 프레임:
/// 1. `GameStats.elapsed` 에 맞는 wave 를 선택.
/// 2. wave 의 `spawn_interval` cooldown 마다 weight random 적 1마리 스폰.
/// 3. `despawn_radius` 밖 적 정리.
pub struct SpawnDirectorSystem {
    pub spawn_radius:   f32,
    pub despawn_radius: f32,
    pub viewport:       Vec2,
}

impl Default for SpawnDirectorSystem {
    fn default() -> Self {
        Self {
            spawn_radius:   500.0,
            despawn_radius: 900.0,
            viewport:       Vec2::new(800.0, 600.0),
        }
    }
}

impl System for SpawnDirectorSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        if !matches!(world.resource::<GameState>(), Some(GameState::Playing)) {
            return;
        }

        // 1) 게임 진행 시간 (GameStats.elapsed 단일 진실원)
        let game_elapsed = world.resource::<GameStats>().map(|s| s.elapsed).unwrap_or(0.0);

        // 2) 화면 중심 계산
        let camera_pos = world.resource::<Camera>().map(|c| c.position).unwrap_or(Vec2::ZERO);
        let center = camera_pos + self.viewport * 0.5;

        // 3) 현재 wave 의 spawn_interval 추출 + wave 클론 (borrow 종료용)
        //    — 이후 resource_mut::<SpawnDirector> 로 cooldown 진행이 필요하므로 미리 클론
        let wave_clone = {
            let director = match world.resource::<SpawnDirector>() {
                Some(d) => d,
                None    => return,
            };
            match director.current_wave(game_elapsed) {
                Some(w) => w.clone(),
                None    => return,
            }
        };
        let interval = wave_clone.spawn_interval;

        // 4) cooldown 진행 — resource_mut borrow 블록
        let should_spawn = {
            let director = world.resource_mut::<SpawnDirector>().unwrap();
            director.spawn_elapsed += dt;
            if director.spawn_elapsed >= interval {
                director.spawn_elapsed -= interval;
                true
            } else {
                false
            }
        };

        // 5) 스폰 (패턴 + count_per_burst + elite_chance)
        if should_spawn {
            let mut rng = rand::thread_rng();
            spawn_pattern(world, &wave_clone, center, self.spawn_radius, &mut rng);
        }

        // 6) despawn_radius 밖 적 정리
        use engine::Entity;
        use super::enemy::Enemy;
        let to_despawn: Vec<Entity> = world
            .query2::<Enemy, Transform>()
            .filter(|(_, _, t)| (t.position - center).length() > self.despawn_radius)
            .map(|(e, _, _)| e)
            .collect();
        for e in to_despawn {
            world.despawn(e);
        }
    }
}

/// 한 발화 분량의 적을 wave 의 패턴/카운트/엘리트 확률에 따라 스폰.
pub fn spawn_pattern(
    world: &mut World,
    wave: &WaveDef,
    center: Vec2,
    spawn_radius: f32,
    rng: &mut impl Rng,
) {
    let count = wave.count_per_burst.max(1) as usize;
    let positions: Vec<Vec2> = match wave.pattern {
        SpawnPattern::Random => (0..count).map(|_| {
            let a = rng.gen_range(0.0..std::f32::consts::TAU);
            center + Vec2::new(a.cos(), a.sin()) * spawn_radius
        }).collect(),
        SpawnPattern::Line => {
            let a = rng.gen_range(0.0..std::f32::consts::TAU);
            let dir = Vec2::new(a.cos(), a.sin());
            let perp = Vec2::new(-dir.y, dir.x);
            let base = center + dir * spawn_radius;
            (0..count).map(|i| {
                let offset = (i as f32 - (count as f32 - 1.0) * 0.5) * 60.0;
                base + perp * offset
            }).collect()
        }
        SpawnPattern::Circle | SpawnPattern::Surround => {
            (0..count).map(|i| {
                let a = (i as f32 / count as f32) * std::f32::consts::TAU;
                center + Vec2::new(a.cos(), a.sin()) * spawn_radius
            }).collect()
        }
        SpawnPattern::Swarm => {
            let a = rng.gen_range(0.0..std::f32::consts::TAU);
            let anchor = center + Vec2::new(a.cos(), a.sin()) * spawn_radius;
            (0..count).map(|_| {
                anchor + Vec2::new(rng.gen_range(-20.0..20.0), rng.gen_range(-20.0..20.0))
            }).collect()
        }
    };

    for pos in positions {
        if let Some(kind) = SpawnDirector::pick_enemy_from(wave, rng) {
            let is_elite = wave.elite_chance > 0.0
                && rng.gen_bool(wave.elite_chance.clamp(0.0, 1.0) as f64);
            spawn_enemy_elite(world, pos, kind, is_elite);
        }
    }
}
