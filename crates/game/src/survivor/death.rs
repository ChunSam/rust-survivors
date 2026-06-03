use super::boss::{BossSpawnQueue, CameraShake, StageProgress};
use super::enemy::Enemy;
use super::health::Health;
use super::hud::GameStats;
use super::player::{Player, PlayerStats};
use super::sfx::{SfxEvent, SfxQueue};
use super::{world_setup, LAYER_ENEMY};
use engine::{
    Camera, CollisionLayer, GameState, InputState, SpatialGrid, System, Transform, World,
};
use glam::Vec2;
use winit::keyboard::KeyCode;

/// 플레이어가 실제로 적에게 닿았다고 보는 논리 반경.
///
/// 스프라이트는 가시성을 위해 크게 렌더링하지만, 피격 판정까지 같이 커지면
/// Whip 같은 근접 공격 중에도 닿지 않은 적에게 접촉 데미지를 받을 수 있다.
pub const PLAYER_CONTACT_RADIUS: f32 = 30.0;

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
            contact_radius: PLAYER_CONTACT_RADIUS,
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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RevivalState {
    pub used: u32,
}

impl System for DeathSystem {
    fn run(&mut self, world: &mut World, _dt: f32) {
        if !matches!(world.resource::<GameState>(), Some(GameState::Playing)) {
            return;
        }
        let player = world
            .query2::<Player, Health>()
            .next()
            .map(|(e, _, h)| (e, h.current, h.max));
        if let Some((player_entity, hp, max_hp)) = player {
            if hp > 0.0 {
                return;
            }

            let revival_limit = world
                .get::<PlayerStats>(player_entity)
                .map(|stats| stats.revival)
                .unwrap_or(0);
            let used = world
                .resource::<RevivalState>()
                .map(|state| state.used)
                .unwrap_or(0);

            if used < revival_limit {
                if world.resource::<RevivalState>().is_none() {
                    world.insert_resource(RevivalState::default());
                }
                if let Some(state) = world.resource_mut::<RevivalState>() {
                    state.used += 1;
                }
                if let Some(h) = world.get_mut::<Health>(player_entity) {
                    h.current = (max_hp * 0.5).max(1.0);
                }
                if let Some(q) = world.resource_mut::<SfxQueue>() {
                    q.push(SfxEvent::LevelUp);
                }
                println!("Revived ({}/{})", used + 1, revival_limit);
                return;
            }

            if let Some(gs) = world.resource_mut::<GameState>() {
                *gs = GameState::GameOver;
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
    reset_run_state(world);

    // Player + 관련 리소스 재설정
    world_setup::setup_survivor_world(world);

    // Playing 으로 복귀
    if let Some(gs) = world.resource_mut::<GameState>() {
        *gs = GameState::Playing;
    }

    println!("Restarted.");
}

/// Title/Menu 로 돌아갈 때 사용하는 정리 경로.
///
/// `restart_world`와 달리 새 Player를 즉시 스폰하지 않는다. Title 상태에 Player가 남으면
/// CameraFollowSystem이 메뉴 화면에서도 플레이어를 따라가 다음 런 진입 시 카메라가 꼬일 수 있다.
pub fn reset_to_title_world(world: &mut World) {
    reset_run_state(world);
    if let Some(gs) = world.resource_mut::<GameState>() {
        *gs = GameState::Paused;
    }
    if let Some(camera) = world.resource_mut::<Camera>() {
        camera.position = Vec2::ZERO;
        camera.zoom = 1.0;
    }
}

fn reset_run_state(world: &mut World) {
    // 모든 게임 엔티티 정리 (Player/Zombie/XpGem/TitleBackdrop 은 모두 Transform 보유)
    let to_despawn: Vec<engine::Entity> = world.query::<Transform>().map(|(e, _)| e).collect();
    for e in to_despawn {
        world.despawn(e);
    }

    // 게임 진행 상태 리셋
    if let Some(stats) = world.resource_mut::<GameStats>() {
        *stats = GameStats::default();
    }
    world.remove_resource::<super::levelup::PendingLevelUp>();
    world.remove_resource::<super::levelup::LevelUpActions>();
    world.remove_resource::<RevivalState>();
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::survivor::enemy::{Enemy, EnemyKind};
    use crate::survivor::levelup::{CardKind, PendingLevelUp};
    use crate::survivor::LAYER_ENEMY;
    use engine::{Collider, World};

    #[test]
    fn restart_world_spawns_player_and_sets_playing() {
        let mut world = World::new();
        world.insert_resource(GameState::Paused);

        restart_world(&mut world);

        assert!(matches!(
            world.resource::<GameState>(),
            Some(GameState::Playing)
        ));
        assert!(world.query::<Player>().next().is_some());
    }

    #[test]
    fn reset_to_title_world_despawns_player_and_pauses() {
        let mut world = World::new();
        world.insert_resource(GameState::Playing);
        world.insert_resource(PendingLevelUp {
            offered: [
                CardKind::WhipDamage,
                CardKind::WhipArea,
                CardKind::WhipCooldown,
            ],
        });
        world_setup::setup_survivor_world(&mut world);

        reset_to_title_world(&mut world);

        assert!(matches!(
            world.resource::<GameState>(),
            Some(GameState::Paused)
        ));
        assert!(world.query::<Player>().next().is_none());
        assert!(
            world.resource::<PendingLevelUp>().is_none(),
            "reset should remove pending level-up state"
        );
    }

    fn setup_contact_world(enemy_x: f32) -> World {
        let mut world = World::new();
        world.insert_resource(GameState::Playing);
        world.insert_resource(CameraShake::default());
        world.insert_resource(SfxQueue::default());

        let player = world.spawn();
        world.add_component(player, Player);
        world.add_component(
            player,
            Transform {
                position: Vec2::ZERO,
                scale: Vec2::splat(150.0),
                rotation: 0.0,
                z: 0.0,
            },
        );
        world.add_component(player, Health::new(100.0));
        world.add_component(player, PlayerStats::default());

        let enemy = world.spawn();
        world.add_component(
            enemy,
            Transform {
                position: Vec2::new(enemy_x, 0.0),
                scale: Vec2::splat(150.0),
                rotation: 0.0,
                z: 0.0,
            },
        );
        world.add_component(
            enemy,
            Enemy {
                kind: EnemyKind::Zombie,
                contact_damage: 10.0,
                split_remaining: 0,
                is_elite: false,
            },
        );
        world.add_component(enemy, CollisionLayer(LAYER_ENEMY));
        world.add_component(enemy, Collider::Circle { radius: 20.0 });
        world
    }

    #[test]
    fn contact_damage_uses_logical_radius_not_visual_size() {
        let mut world = setup_contact_world(70.0);
        let mut system = EnemyContactDamageSystem::default();

        system.run(&mut world, 1.0);

        let hp = world
            .query2::<Player, Health>()
            .next()
            .map(|(_, _, h)| h.current);
        assert_eq!(hp, Some(100.0));
    }

    #[test]
    fn contact_damage_applies_when_bodies_overlap() {
        let mut world = setup_contact_world(45.0);
        let mut system = EnemyContactDamageSystem::default();

        system.run(&mut world, 1.0);

        let hp = world
            .query2::<Player, Health>()
            .next()
            .map(|(_, _, h)| h.current);
        assert_eq!(hp, Some(90.0));
    }

    #[test]
    fn death_system_consumes_revival_before_game_over() {
        let mut world = World::new();
        world.insert_resource(GameState::Playing);
        world.insert_resource(SfxQueue::default());
        let player = world.spawn();
        world.add_component(player, Player);
        world.add_component(player, Health::new(100.0));
        world.get_mut::<Health>(player).unwrap().current = 0.0;
        world.add_component(
            player,
            PlayerStats {
                revival: 1,
                ..PlayerStats::default()
            },
        );
        let mut system = DeathSystem;

        system.run(&mut world, 0.0);

        assert_eq!(world.resource::<GameState>(), Some(&GameState::Playing));
        assert_eq!(world.get::<Health>(player).map(|h| h.current), Some(50.0));
        assert_eq!(
            world.resource::<RevivalState>().map(|state| state.used),
            Some(1)
        );
    }

    #[test]
    fn death_system_game_over_after_revival_is_spent() {
        let mut world = World::new();
        world.insert_resource(GameState::Playing);
        world.insert_resource(RevivalState { used: 1 });
        let player = world.spawn();
        world.add_component(player, Player);
        world.add_component(player, Health::new(100.0));
        world.get_mut::<Health>(player).unwrap().current = 0.0;
        world.add_component(
            player,
            PlayerStats {
                revival: 1,
                ..PlayerStats::default()
            },
        );
        let mut system = DeathSystem;

        system.run(&mut world, 0.0);

        assert_eq!(world.resource::<GameState>(), Some(&GameState::GameOver));
    }

    #[test]
    fn reset_to_title_clears_run_only_action_state() {
        let mut world = World::new();
        world.insert_resource(GameState::Playing);
        world.insert_resource(super::super::levelup::LevelUpActions {
            reroll_remaining: 1,
            ..super::super::levelup::LevelUpActions::default()
        });
        world.insert_resource(RevivalState { used: 1 });
        world_setup::setup_survivor_world(&mut world);

        reset_to_title_world(&mut world);

        assert!(world
            .resource::<super::super::levelup::LevelUpActions>()
            .is_none());
        assert!(world.resource::<RevivalState>().is_none());
    }
}
