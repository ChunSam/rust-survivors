use super::boss::{BossSpawnQueue, CameraShake, StageProgress};
use super::enemy::Enemy;
use super::health::Health;
use super::hud::GameStats;
use super::player::{Player, PlayerStats};
use super::sfx::{SfxEvent, SfxQueue};
use super::{world_setup, LAYER_ENEMY};
use engine::components::GameState;
use engine::input::InputState;
use engine::{CollisionLayer, SpatialGrid, System, Transform, World};
use winit::keyboard::KeyCode;

/// 매 프레임 Player 주변의 적과 접촉 시 데미지 누적.
/// 1초 cooldown 으로 Player Health -= 10 (적 1마리 이상 접촉 시).
pub struct EnemyContactDamageSystem {
    pub grid: SpatialGrid,
    pub elapsed: f32,
    pub cooldown: f32,
    pub damage: f32,
    pub contact_radius: f32,
}

impl Default for EnemyContactDamageSystem {
    fn default() -> Self {
        Self {
            grid: SpatialGrid::new(128.0),
            elapsed: 0.0,
            cooldown: 1.0,
            damage: 10.0,
            contact_radius: 25.0,
        }
    }
}

impl System for EnemyContactDamageSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        if !matches!(world.resource::<GameState>(), Some(GameState::Playing)) {
            return;
        }

        self.elapsed += dt;
        if self.elapsed < self.cooldown {
            return;
        }
        self.elapsed -= self.cooldown;

        // Player 위치 캐시 (borrow 즉시 종료)
        let player = world
            .query2::<Player, Transform>()
            .next()
            .map(|(e, _, t)| (e, t.position));
        let Some((player_entity, player_pos)) = player else {
            return;
        };

        self.grid.rebuild(world);
        let hits =
            self.grid
                .query_radius(player_pos, self.contact_radius, CollisionLayer(LAYER_ENEMY));
        if hits.is_empty() {
            return;
        }

        // armor stat: 받는 데미지 감쇄 (0.0 기본 → 변화 없음)
        let armor = world
            .query2::<Player, PlayerStats>()
            .next()
            .map(|(_, _, s)| s.armor)
            .unwrap_or(0.0);

        // 각 적의 contact_damage 합산 (Enemy 컴포넌트가 없으면 self.damage fallback)
        let mut total_damage = 0.0_f32;
        for e in &hits {
            if let Some(en) = world.get::<Enemy>(*e) {
                total_damage += en.contact_damage;
            } else {
                total_damage += self.damage; // legacy fallback
            }
        }
        let effective_damage = (total_damage - armor).max(0.0);

        if let Some(h) = world.get_mut::<Health>(player_entity) {
            let died = h.take_damage(effective_damage);
            println!("Player hit by {} enemies (HP={:.0})", hits.len(), h.current);
            if died {
                println!("YOU DIED");
            }
        }
        if effective_damage > 0.0 {
            if let Some(s) = world.resource_mut::<CameraShake>() {
                s.trigger(0.15, 4.0);
            }
            if let Some(q) = world.resource_mut::<SfxQueue>() {
                q.push(SfxEvent::PlayerHit);
            }
        }
    }
}

/// Player Health <= 0 이면 GameState::GameOver 로 전환.
pub struct DeathSystem;

impl System for DeathSystem {
    fn run(&mut self, world: &mut World, _dt: f32) {
        if !matches!(world.resource::<GameState>(), Some(GameState::Playing)) {
            return;
        }
        let hp = world
            .query2::<Player, Health>()
            .next()
            .map(|(_, _, h)| h.current);
        if let Some(hp) = hp {
            if hp <= 0.0 {
                if let Some(gs) = world.resource_mut::<GameState>() {
                    *gs = GameState::GameOver;
                }
            }
        }
    }
}

/// 월드 리셋 로직 헬퍼 — 테스트에서 직접 호출 가능하므로 pub 으로 분리.
///
/// - Transform 컴포넌트를 가진 모든 게임 엔티티 despawn (Player/Zombie/XpGem 포함).
/// - 게임 진행 리소스(GameStats, PendingLevelUp, SpawnDirector) 초기화.
/// - setup_survivor_world 로 Player 재스폰 + 리소스 재설정.
/// - GameState 를 Playing 으로 전환.
pub fn restart_world(world: &mut World) {
    // 모든 게임 엔티티 정리 (Player/Zombie/XpGem 은 모두 Transform 보유)
    let to_despawn: Vec<engine::Entity> = world.query::<Transform>().map(|(e, _)| e).collect();
    for e in to_despawn {
        world.despawn(e);
    }

    // 게임 진행 상태 리셋
    if let Some(stats) = world.resource_mut::<GameStats>() {
        *stats = GameStats::default();
    }
    if let Some(p) = world.resource_mut::<super::levelup::PendingLevelUp>() {
        p.consumed = true; // LevelUpSystem 이 다음 프레임에서 skip
    }
    // SpawnDirector cooldown 리셋 (waves 정의는 유지, 내부 카운터만 초기화)
    if let Some(d) = world.resource_mut::<super::director::SpawnDirector>() {
        d.spawn_elapsed = 0.0;
    }

    // Phase 5: 보스 리소스 리셋
    if let Some(q) = world.resource_mut::<BossSpawnQueue>() {
        *q = BossSpawnQueue::default();
    }
    if let Some(s) = world.resource_mut::<CameraShake>() {
        *s = CameraShake::default();
    }
    if let Some(p) = world.resource_mut::<StageProgress>() {
        *p = StageProgress::default();
    }

    // Player + 관련 리소스 재설정
    world_setup::setup_survivor_world(world);

    // Playing 으로 복귀
    if let Some(gs) = world.resource_mut::<GameState>() {
        *gs = GameState::Playing;
    }

    println!("Restarted.");
}

/// GameOver 상태에서 R 키 → restart_world 호출.
pub struct RestartSystem;

impl System for RestartSystem {
    fn run(&mut self, world: &mut World, _dt: f32) {
        let in_gameover = matches!(world.resource::<GameState>(), Some(GameState::GameOver));
        if !in_gameover {
            return;
        }

        let r_pressed = world
            .resource::<InputState>()
            .map(|i| i.just_pressed(KeyCode::KeyR))
            .unwrap_or(false);
        if !r_pressed {
            return;
        }

        restart_world(world);
    }
}
