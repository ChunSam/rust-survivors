use engine::components::GameState;
use engine::{Camera, Sprite, System, Transform, ViewportSize, World};
use glam::Vec2;

use super::meta::SurvivorMode;
use super::sprites::TITLE_BACKDROP_PATH;

#[derive(Debug, Clone, Copy)]
pub struct TitleBackdrop;

pub struct TitleVisualSystem;

impl System for TitleVisualSystem {
    fn run(&mut self, world: &mut World, _dt: f32) {
        let mode = world
            .resource::<SurvivorMode>()
            .copied()
            .unwrap_or(SurvivorMode::Title);

        if !matches!(mode, SurvivorMode::Title) {
            let existing: Vec<_> = world.query::<TitleBackdrop>().map(|(e, _)| e).collect();
            for e in existing {
                world.despawn(e);
            }
            return;
        }

        let state = world
            .resource::<GameState>()
            .cloned()
            .unwrap_or(GameState::Paused);
        if !matches!(state, GameState::Paused) {
            return;
        }

        let entity = world.query::<TitleBackdrop>().next().map(|(e, _)| e);
        let entity = match entity {
            Some(e) => e,
            None => {
                let e = world.spawn();
                world.add_component(e, TitleBackdrop);
                world.add_component(e, Sprite::textured(TITLE_BACKDROP_PATH));
                world.add_component(e, Transform::default());
                e
            }
        };

        let viewport = world
            .resource::<ViewportSize>()
            .map(|v| Vec2::new(v.width, v.height))
            .unwrap_or(Vec2::new(1280.0, 720.0));
        let camera = world.resource::<Camera>().copied().unwrap_or_default();
        let visible = viewport / camera.zoom.max(0.1);
        let center = camera.position + visible * 0.5;

        if let Some(t) = world.get_mut::<Transform>(entity) {
            t.position = center;
            t.scale = visible * 1.04;
            t.rotation = 0.0;
            t.z = -10.0;
        }
    }
}
