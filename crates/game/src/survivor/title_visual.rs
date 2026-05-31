use engine::{
    Camera, DrawImage, GameState, RenderLayer, System, Transform, UiImageQueue, ViewportSize, World,
};
use glam::Vec2;

use super::locale::Lang;
use super::meta::{title_button_layout, MetaSave, SurvivorMode};
use super::sprites::{
    survivor_texture_aspect, survivor_texture_handle, survivor_textured_sprite,
    MENU_BUTTON_ACHIEVEMENTS_EN_PATH, MENU_BUTTON_ACHIEVEMENTS_KO_PATH,
    MENU_BUTTON_CHARACTER_EN_PATH, MENU_BUTTON_CHARACTER_KO_PATH, MENU_BUTTON_SETTINGS_EN_PATH,
    MENU_BUTTON_SETTINGS_KO_PATH, MENU_BUTTON_SHOP_EN_PATH, MENU_BUTTON_SHOP_KO_PATH,
    MENU_BUTTON_STAGE_EN_PATH, MENU_BUTTON_STAGE_KO_PATH, MENU_BUTTON_START_EN_PATH,
    MENU_BUTTON_START_KO_PATH, RENDER_LAYER_BACKGROUND, TITLE_BACKDROP_PATH,
    TITLE_LOGO_PLAQUE_PATH,
};
use super::ui_layout::{
    title_logo_rect as ui_title_logo_rect, title_menu_button_image_rect, title_start_image_rect,
    ScreenRect,
};

#[derive(Debug, Clone, Copy)]
pub struct TitleBackdrop;

pub struct TitleVisualSystem;

const TITLE_MENU_LOGO_Z: f32 = 28.0;
const TITLE_MENU_BUTTON_Z: f32 = 30.0;

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
                world.add_component(e, survivor_textured_sprite(world, TITLE_BACKDROP_PATH));
                world.add_component(e, RenderLayer(RENDER_LAYER_BACKGROUND));
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
            let image_rect = ScreenRect::new(0.0, 0.0, visible.x, visible.y).aspect_fill(
                survivor_texture_aspect(TITLE_BACKDROP_PATH).unwrap_or(visible.x / visible.y),
            );
            t.scale = Vec2::new(image_rect.w, image_rect.h) * 1.04;
            t.rotation = 0.0;
            t.z = -10.0;
        }

        let lang = world
            .resource::<MetaSave>()
            .map(|m| m.effective_lang())
            .unwrap_or(Lang::Ko);
        queue_title_menu_images(world, viewport, lang);
    }
}

fn queue_title_menu_images(world: &mut World, viewport: Vec2, lang: Lang) {
    let layout = title_button_layout(viewport.x, viewport.y);
    let logo = ui_title_logo_rect(viewport.x, viewport.y);
    let start =
        title_start_image_rect(ScreenRect::from_tuple(layout.start), viewport.x, viewport.y);
    let start_path = title_start_button_path(lang);
    let logo_image = image_rect_for_path(logo, TITLE_LOGO_PLAQUE_PATH);
    let start_image = image_rect_for_path(start, start_path);

    queue_screen_image(world, logo_image, TITLE_LOGO_PLAQUE_PATH, TITLE_MENU_LOGO_Z);
    queue_screen_image(world, start_image, start_path, TITLE_MENU_BUTTON_Z);

    let paths = title_menu_button_paths(lang);
    for (path, rect) in paths.into_iter().zip(layout.buttons) {
        let rect =
            title_menu_button_image_rect(ScreenRect::from_tuple(rect), viewport.x, viewport.y);
        let image_rect = image_rect_for_path(rect, path);
        queue_screen_image(world, image_rect, path, TITLE_MENU_BUTTON_Z);
    }
}

fn title_start_button_path(lang: Lang) -> &'static str {
    match lang {
        Lang::Ko => MENU_BUTTON_START_KO_PATH,
        Lang::En => MENU_BUTTON_START_EN_PATH,
    }
}

fn title_menu_button_paths(lang: Lang) -> [&'static str; 5] {
    match lang {
        Lang::Ko => [
            MENU_BUTTON_CHARACTER_KO_PATH,
            MENU_BUTTON_STAGE_KO_PATH,
            MENU_BUTTON_SHOP_KO_PATH,
            MENU_BUTTON_ACHIEVEMENTS_KO_PATH,
            MENU_BUTTON_SETTINGS_KO_PATH,
        ],
        Lang::En => [
            MENU_BUTTON_CHARACTER_EN_PATH,
            MENU_BUTTON_STAGE_EN_PATH,
            MENU_BUTTON_SHOP_EN_PATH,
            MENU_BUTTON_ACHIEVEMENTS_EN_PATH,
            MENU_BUTTON_SETTINGS_EN_PATH,
        ],
    }
}

fn image_rect_for_path(rect: ScreenRect, path: &str) -> ScreenRect {
    survivor_texture_aspect(path)
        .map(|aspect| rect.aspect_fit(aspect))
        .unwrap_or(rect)
}

fn queue_screen_image(world: &mut World, rect: ScreenRect, path: &str, z: f32) {
    let rect = image_rect_for_path(rect, path);
    let handle = survivor_texture_handle(world, path);
    if let Some(queue) = world.resource_mut::<UiImageQueue>() {
        queue.push(
            DrawImage::textured_with_handle(rect.x, rect.y, rect.w, rect.h, path, handle).with_z(z),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_inside_viewport(rect: ScreenRect, vw: f32, vh: f32) {
        assert!(rect.x >= 0.0);
        assert!(rect.y >= 0.0);
        assert!(rect.x + rect.w <= vw);
        assert!(rect.bottom() <= vh);
    }

    #[test]
    fn compact_title_menu_images_fit_800x600() {
        let layout = title_button_layout(800.0, 600.0);
        let logo = ui_title_logo_rect(800.0, 600.0);
        let start = title_start_image_rect(ScreenRect::from_tuple(layout.start), 800.0, 600.0);
        assert_inside_viewport(logo, 800.0, 600.0);
        assert_inside_viewport(start, 800.0, 600.0);
        for rect in layout.buttons {
            assert_inside_viewport(
                title_menu_button_image_rect(ScreenRect::from_tuple(rect), 800.0, 600.0),
                800.0,
                600.0,
            );
        }
        assert!(logo.bottom() < start.y);
        assert!(
            start.bottom()
                < title_menu_button_image_rect(
                    ScreenRect::from_tuple(layout.buttons[0]),
                    800.0,
                    600.0
                )
                .y
        );
    }

    #[test]
    fn default_title_menu_images_fit_1280x720() {
        let layout = title_button_layout(1280.0, 720.0);
        assert_inside_viewport(ui_title_logo_rect(1280.0, 720.0), 1280.0, 720.0);
        assert_inside_viewport(
            title_start_image_rect(ScreenRect::from_tuple(layout.start), 1280.0, 720.0),
            1280.0,
            720.0,
        );
        for rect in layout.buttons {
            assert_inside_viewport(
                title_menu_button_image_rect(ScreenRect::from_tuple(rect), 1280.0, 720.0),
                1280.0,
                720.0,
            );
        }
    }

    #[test]
    fn title_image_rects_share_hit_rect_centers() {
        let layout = title_button_layout(1280.0, 720.0);
        let start_hit = ScreenRect::from_tuple(layout.start);
        let start_image = title_start_image_rect(start_hit, 1280.0, 720.0);
        assert_eq!(start_image.center(), start_hit.center());

        for rect in layout.buttons {
            let hit = ScreenRect::from_tuple(rect);
            let image = title_menu_button_image_rect(hit, 1280.0, 720.0);
            assert_eq!(image.center(), hit.center());
        }
    }

    #[test]
    fn title_menu_images_queue_only_transparent_pngs() {
        let mut world = World::new();
        world.insert_resource(UiImageQueue::default());

        queue_title_menu_images(&mut world, Vec2::new(1280.0, 720.0), Lang::Ko);

        let queue = world.resource::<UiImageQueue>().unwrap();
        let textured_count = queue
            .items
            .iter()
            .filter(|image| image.texture.is_some())
            .count();

        assert!(queue.items.iter().all(|image| image.texture.is_some()));
        assert_eq!(textured_count, 7);
        assert!(queue
            .items
            .iter()
            .filter(|image| image.texture.is_some())
            .all(|image| image.uv == engine::UvRect::FULL));
    }

    #[test]
    fn title_menu_images_preserve_source_aspects() {
        let mut world = World::new();
        world.insert_resource(UiImageQueue::default());

        queue_title_menu_images(&mut world, Vec2::new(1280.0, 720.0), Lang::En);

        let queue = world.resource::<UiImageQueue>().unwrap();
        for image in queue.items.iter().filter(|image| image.texture.is_some()) {
            let path = image.texture.as_deref().unwrap();
            let aspect = survivor_texture_aspect(path).unwrap();
            assert!(
                (image.w / image.h - aspect).abs() < 0.001,
                "{path} should keep original aspect"
            );
        }
    }
}
