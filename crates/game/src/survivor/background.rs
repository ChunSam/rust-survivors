use super::meta::SurvivorMode;
use super::sprites::RENDER_LAYER_BACKGROUND;
use super::stage::{SelectedStage, StageKind};
use engine::{Camera, Entity, RenderLayer, Sprite, System, Transform, ViewportSize, World};
use glam::Vec2;

const TILE_SIZE: f32 = 96.0;
const TILE_Z: f32 = -10.0;

/// Camera 주변에 저채도 월드 타일을 깔아 단색 배경을 피한다.
pub struct BackgroundSystem {
    tiles: Vec<Entity>,
}

impl Default for BackgroundSystem {
    fn default() -> Self {
        Self { tiles: Vec::new() }
    }
}

impl System for BackgroundSystem {
    fn run(&mut self, world: &mut World, _dt: f32) {
        let mode = world
            .resource::<SurvivorMode>()
            .copied()
            .unwrap_or_default();
        if !matches!(mode, SurvivorMode::InGame | SurvivorMode::PauseMenu) {
            return;
        }

        let camera = world.resource::<Camera>().copied().unwrap_or_default();
        let viewport = world
            .resource::<ViewportSize>()
            .copied()
            .unwrap_or_default();
        let stage = world
            .resource::<SelectedStage>()
            .copied()
            .unwrap_or_default()
            .0;

        let (cols, rows) = tile_grid_size(viewport, camera.zoom);
        let needed = (cols * rows).max(0) as usize;

        self.tiles.retain(|&e| world.get::<Transform>(e).is_some());
        while self.tiles.len() < needed {
            let entity = world.spawn();
            world.add_component(
                entity,
                Transform {
                    position: Vec2::ZERO,
                    scale: Vec2::splat(TILE_SIZE + 1.0),
                    rotation: 0.0,
                    z: TILE_Z,
                },
            );
            world.add_component(entity, Sprite::colored(0.12, 0.14, 0.18));
            world.add_component(entity, RenderLayer(RENDER_LAYER_BACKGROUND));
            self.tiles.push(entity);
        }
        while self.tiles.len() > needed {
            if let Some(entity) = self.tiles.pop() {
                world.despawn(entity);
            }
        }

        let start_x = (camera.position.x / TILE_SIZE).floor() as i32 - 1;
        let start_y = (camera.position.y / TILE_SIZE).floor() as i32 - 1;
        for row in 0..rows {
            for col in 0..cols {
                let idx = (row * cols + col) as usize;
                let Some(&entity) = self.tiles.get(idx) else {
                    continue;
                };
                let gx = start_x + col;
                let gy = start_y + row;
                if let Some(t) = world.get_mut::<Transform>(entity) {
                    t.position =
                        Vec2::new((gx as f32 + 0.5) * TILE_SIZE, (gy as f32 + 0.5) * TILE_SIZE);
                    t.scale = Vec2::splat(TILE_SIZE + 1.0);
                    t.z = TILE_Z;
                }
                if let Some(sprite) = world.get_mut::<Sprite>(entity) {
                    sprite.color = tile_color(stage, gx, gy);
                }
            }
        }
    }
}

fn tile_color(stage: StageKind, x: i32, y: i32) -> [f32; 4] {
    let noise = hash01(x, y);
    let stripe = if (x + y).rem_euclid(5) == 0 {
        0.018
    } else {
        0.0
    };
    let vignette = if (x - y).rem_euclid(9) == 0 {
        0.014
    } else {
        0.0
    };
    let v = (noise - 0.5) * 0.035 + stripe - vignette;
    let (r, g, b) = match stage {
        StageKind::MadForest => (0.105, 0.135, 0.115),
        StageKind::InlaidLibrary => (0.135, 0.118, 0.098),
        StageKind::DairyPlant => (0.120, 0.130, 0.142),
    };
    [r + v, g + v, b + v, 1.0]
}

fn tile_grid_size(viewport: ViewportSize, zoom: f32) -> (i32, i32) {
    let zoom = zoom.max(0.01);
    let visible_w = viewport.width / zoom;
    let visible_h = viewport.height / zoom;
    (
        (visible_w / TILE_SIZE).ceil() as i32 + 4,
        (visible_h / TILE_SIZE).ceil() as i32 + 4,
    )
}

fn hash01(x: i32, y: i32) -> f32 {
    let mut n = x
        .wrapping_mul(374_761_393)
        .wrapping_add(y.wrapping_mul(668_265_263));
    n = (n ^ (n >> 13)).wrapping_mul(1_274_126_177);
    ((n ^ (n >> 16)) as u32 & 0xffff) as f32 / 65_535.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine::System;

    #[test]
    fn tile_grid_size_accounts_for_zoomed_out_camera() {
        let viewport = ViewportSize {
            width: 800.0,
            height: 600.0,
        };

        let normal = tile_grid_size(viewport, 1.0);
        let boss_zoom = tile_grid_size(viewport, 0.65);

        assert!(
            boss_zoom.0 > normal.0,
            "zooming out increases visible world width, so more tile columns are needed"
        );
        assert!(
            boss_zoom.1 > normal.1,
            "zooming out increases visible world height, so more tile rows are needed"
        );
    }

    #[test]
    fn background_tiles_cover_800x600_boss_zoom_view() {
        let mut world = World::new();
        let viewport = ViewportSize {
            width: 800.0,
            height: 600.0,
        };
        let camera = Camera::new(Vec2::new(-400.0, -300.0), 0.65);

        world.insert_resource(SurvivorMode::InGame);
        world.insert_resource(viewport);
        world.insert_resource(camera);
        world.insert_resource(SelectedStage(StageKind::MadForest));

        let mut system = BackgroundSystem::default();
        system.run(&mut world, 0.0);

        let visible_left = camera.position.x;
        let visible_right = camera.position.x + viewport.width / camera.zoom;
        let visible_top = camera.position.y;
        let visible_bottom = camera.position.y + viewport.height / camera.zoom;

        let mut covered_left = f32::INFINITY;
        let mut covered_right = f32::NEG_INFINITY;
        let mut covered_top = f32::INFINITY;
        let mut covered_bottom = f32::NEG_INFINITY;

        for (_, transform) in world.query::<Transform>() {
            if (transform.z - TILE_Z).abs() > f32::EPSILON {
                continue;
            }
            let half = transform.scale.x * 0.5;
            covered_left = covered_left.min(transform.position.x - half);
            covered_right = covered_right.max(transform.position.x + half);
            covered_top = covered_top.min(transform.position.y - half);
            covered_bottom = covered_bottom.max(transform.position.y + half);
        }

        assert!(
            covered_left <= visible_left,
            "tile coverage must reach visible left edge: covered={covered_left}, visible={visible_left}"
        );
        assert!(
            covered_right >= visible_right,
            "tile coverage must reach visible right edge: covered={covered_right}, visible={visible_right}"
        );
        assert!(
            covered_top <= visible_top,
            "tile coverage must reach visible top edge: covered={covered_top}, visible={visible_top}"
        );
        assert!(
            covered_bottom >= visible_bottom,
            "tile coverage must reach visible bottom edge: covered={covered_bottom}, visible={visible_bottom}"
        );
    }
}
