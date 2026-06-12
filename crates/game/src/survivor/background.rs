use super::meta::SurvivorMode;
use super::sprites::{
    survivor_textured_sprite, DAIRY_PLANT_TILES_PATH, INLAID_LIBRARY_TILES_PATH,
    MAD_FOREST_TILES_PATH, RENDER_LAYER_BACKGROUND,
};
use super::stage::{SelectedStage, StageKind};
use engine::{Camera, Entity, RenderLayer, Sprite, System, Transform, UvRect, ViewportSize, World};
use glam::Vec2;

const TILE_SIZE: f32 = 96.0;
const TILE_Z: f32 = -10.0;
const TILE_ATLAS_COLS: u32 = 4;
const TILE_ATLAS_ROWS: u32 = 4;

/// Camera 주변에 저채도 월드 타일을 깔아 단색 배경을 피한다.
#[derive(Default)]
pub struct BackgroundSystem {
    tiles: Vec<Entity>,
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
        let tileset = stage_tileset(stage);

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
            let sprite = stage_tile_sprite(world, tileset);
            world.add_component(entity, sprite);
            world.add_component(
                entity,
                UvRect::from_grid(0, 0, TILE_ATLAS_COLS, TILE_ATLAS_ROWS),
            );
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
                let next_sprite = stage_tile_sprite(world, tileset);
                if let Some(sprite) = world.get_mut::<Sprite>(entity) {
                    *sprite = next_sprite;
                }
                if let Some(uv) = world.get_mut::<UvRect>(entity) {
                    *uv = tile_uv(stage, gx, gy);
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct StageTileset {
    path: &'static str,
    tint: [f32; 4],
}

fn stage_tileset(stage: StageKind) -> StageTileset {
    match stage {
        StageKind::MadForest => StageTileset {
            path: MAD_FOREST_TILES_PATH,
            tint: [0.62, 0.74, 0.58, 1.0],
        },
        StageKind::InlaidLibrary => StageTileset {
            path: INLAID_LIBRARY_TILES_PATH,
            tint: [0.70, 0.60, 0.52, 1.0],
        },
        StageKind::DairyPlant => StageTileset {
            path: DAIRY_PLANT_TILES_PATH,
            tint: [0.60, 0.66, 0.72, 1.0],
        },
    }
}

fn stage_tile_sprite(world: &World, tileset: StageTileset) -> Sprite {
    let mut sprite = survivor_textured_sprite(world, tileset.path);
    sprite.color = engine::Color::from(tileset.tint);
    sprite
}

fn tile_uv(stage: StageKind, x: i32, y: i32) -> UvRect {
    let index = tile_variant_index(stage, x, y);
    UvRect::from_grid(
        index % TILE_ATLAS_COLS,
        index / TILE_ATLAS_COLS,
        TILE_ATLAS_COLS,
        TILE_ATLAS_ROWS,
    )
}

fn tile_variant_index(stage: StageKind, x: i32, y: i32) -> u32 {
    let stage_offset = match stage {
        StageKind::MadForest => 0,
        StageKind::InlaidLibrary => 10_000,
        StageKind::DairyPlant => 20_000,
    };
    let variant_count = TILE_ATLAS_COLS * TILE_ATLAS_ROWS;
    let index = (hash01(x.wrapping_add(stage_offset), y) * variant_count as f32).floor() as u32;
    index.min(variant_count - 1)
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

    #[test]
    fn stage_tilesets_use_generated_asset_paths() {
        let forest = stage_tileset(StageKind::MadForest);
        let library = stage_tileset(StageKind::InlaidLibrary);
        let dairy = stage_tileset(StageKind::DairyPlant);

        assert_eq!(forest.path, MAD_FOREST_TILES_PATH);
        assert_eq!(library.path, INLAID_LIBRARY_TILES_PATH);
        assert_eq!(dairy.path, DAIRY_PLANT_TILES_PATH);
        assert_ne!(forest.path, library.path);
        assert_ne!(library.path, dairy.path);
        assert_ne!(dairy.path, forest.path);
    }

    #[test]
    fn stage_tileset_assets_exist() {
        let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let repo_root = manifest_dir.join("../..");

        for stage in [
            StageKind::MadForest,
            StageKind::InlaidLibrary,
            StageKind::DairyPlant,
        ] {
            let tileset = stage_tileset(stage);
            assert!(
                repo_root.join(tileset.path).exists(),
                "stage tileset asset should exist: {}",
                tileset.path
            );
        }
    }

    #[test]
    fn tile_variants_are_deterministic_and_inside_stage_atlas() {
        for stage in [
            StageKind::MadForest,
            StageKind::InlaidLibrary,
            StageKind::DairyPlant,
        ] {
            let first = tile_variant_index(stage, -12, 34);
            let second = tile_variant_index(stage, -12, 34);
            assert_eq!(first, second);
            assert!(first < TILE_ATLAS_COLS * TILE_ATLAS_ROWS);

            let uv = tile_uv(stage, -12, 34);
            assert!((uv.u_size - 0.25).abs() < f32::EPSILON);
            assert!((uv.v_size - 0.25).abs() < f32::EPSILON);
            assert!(uv.u_offset >= 0.0 && uv.u_offset < 1.0);
            assert!(uv.v_offset >= 0.0 && uv.v_offset < 1.0);
        }
    }

    #[test]
    fn background_tiles_use_selected_stage_texture_and_uvs() {
        let mut world = World::new();
        world.insert_resource(SurvivorMode::InGame);
        world.insert_resource(ViewportSize {
            width: 800.0,
            height: 600.0,
        });
        world.insert_resource(Camera::new(Vec2::ZERO, 1.0));
        world.insert_resource(SelectedStage(StageKind::MadForest));

        let mut system = BackgroundSystem::default();
        system.run(&mut world, 0.0);

        let first_tile = *system
            .tiles
            .first()
            .expect("background system should spawn visible tiles");
        let sprite = world.get::<Sprite>(first_tile).unwrap();
        assert_eq!(sprite.texture.as_deref(), Some(MAD_FOREST_TILES_PATH));
        assert!(world.get::<UvRect>(first_tile).is_some());

        world.insert_resource(SelectedStage(StageKind::DairyPlant));
        system.run(&mut world, 0.0);

        let sprite = world.get::<Sprite>(first_tile).unwrap();
        assert_eq!(sprite.texture.as_deref(), Some(DAIRY_PLANT_TILES_PATH));
    }
}
