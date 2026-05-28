use engine::{CollisionLayer, Entity, SpatialGrid, Transform, World};
use glam::Vec2;
use rand::seq::SliceRandom;

use super::damage::apply_damage_to_enemy;
use super::hud::GameStats;
use super::player::{Player, PlayerStats};
use super::stats::read_player_stats;
use super::LAYER_ENEMY;

pub(crate) struct WeaponFireContext {
    pub player_entity: Entity,
    pub player_pos: Vec2,
    pub stats: PlayerStats,
}

pub(crate) fn weapon_fire_context(world: &World) -> Option<WeaponFireContext> {
    let (player_entity, player_pos) = world
        .query2::<Player, Transform>()
        .next()
        .map(|(e, _, t)| (e, t.position))?;
    Some(WeaponFireContext {
        player_entity,
        player_pos,
        stats: read_player_stats(world),
    })
}

pub(crate) fn apply_damage_events<I>(world: &mut World, hits: I) -> u32
where
    I: IntoIterator<Item = (Entity, f32)>,
{
    let killed = hits
        .into_iter()
        .filter(|(enemy, damage)| apply_damage_to_enemy(world, *enemy, *damage))
        .count() as u32;
    add_kills(world, killed);
    killed
}

pub(crate) fn apply_damage_to_targets<I>(world: &mut World, targets: I, damage: f32) -> u32
where
    I: IntoIterator<Item = Entity>,
{
    apply_damage_events(world, targets.into_iter().map(|enemy| (enemy, damage)))
}

pub(crate) fn nearest_enemy_in_radius(
    world: &World,
    grid: &SpatialGrid,
    origin: Vec2,
    radius: f32,
) -> Option<(Entity, Vec2)> {
    grid.query_radius(origin, radius, CollisionLayer(LAYER_ENEMY))
        .into_iter()
        .filter_map(|e| world.get::<Transform>(e).map(|t| (e, t.position)))
        .min_by(|a, b| {
            let da = (a.1 - origin).length_squared();
            let db = (b.1 - origin).length_squared();
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        })
}

pub(crate) fn random_enemy_in_radius(
    world: &World,
    grid: &SpatialGrid,
    origin: Vec2,
    radius: f32,
) -> Option<(Entity, Vec2)> {
    random_enemies_in_radius(world, grid, origin, radius, 1).pop()
}

pub(crate) fn random_enemies_in_radius(
    world: &World,
    grid: &SpatialGrid,
    origin: Vec2,
    radius: f32,
    count: usize,
) -> Vec<(Entity, Vec2)> {
    let candidates = grid.query_radius(origin, radius, CollisionLayer(LAYER_ENEMY));
    let mut rng = rand::thread_rng();
    candidates
        .choose_multiple(&mut rng, count)
        .copied()
        .filter_map(|e| world.get::<Transform>(e).map(|t| (e, t.position)))
        .collect()
}

pub(crate) fn direction_or_right(from: Vec2, to: Vec2) -> Vec2 {
    let dir = to - from;
    if dir.length_squared() > 0.0 {
        dir.normalize()
    } else {
        Vec2::new(1.0, 0.0)
    }
}

fn add_kills(world: &mut World, killed: u32) {
    if killed > 0 {
        if let Some(stats) = world.resource_mut::<GameStats>() {
            stats.kills += killed;
        }
    }
}
