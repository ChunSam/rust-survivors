use engine::{
    Camera, DrawImage, GameState, RenderLayer, System, Transform, UiImageQueue, ViewportSize, World,
};
use glam::Vec2;

use super::meta::{title_button_layout, SurvivorMode};
use super::sprites::{
    survivor_texture_handle, survivor_textured_sprite, vertically_flipped_full_uv,
    MENU_BUTTON_CHARACTER_PATH, MENU_BUTTON_SETTINGS_PATH, MENU_BUTTON_SHOP_PATH,
    MENU_BUTTON_STAGE_PATH, MENU_BUTTON_START_PATH, RENDER_LAYER_BACKGROUND, TITLE_BACKDROP_PATH,
    TITLE_LOGO_PLAQUE_PATH,
};

#[derive(Debug, Clone, Copy)]
pub struct TitleBackdrop;

pub struct TitleVisualSystem;

#[derive(Debug, Clone, Copy)]
struct ScreenRect {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

impl ScreenRect {
    fn from_tuple(rect: (f32, f32, f32, f32)) -> Self {
        Self {
            x: rect.0,
            y: rect.1,
            w: rect.2,
            h: rect.3,
        }
    }

    fn inflated(self, x: f32, y: f32) -> Self {
        Self {
            x: self.x - x,
            y: self.y - y,
            w: self.w + x * 2.0,
            h: self.h + y * 2.0,
        }
    }

    #[cfg(test)]
    fn bottom(self) -> f32 {
        self.y + self.h
    }
}

const TITLE_MENU_LOGO_Z: f32 = 28.0;
const TITLE_MENU_BUTTON_Z: f32 = 30.0;
const TITLE_MENU_BACKING_Z: f32 = 27.0;

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
                world.add_component(e, vertically_flipped_full_uv());
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

        queue_title_menu_images(world, viewport);
    }
}

fn queue_title_menu_images(world: &mut World, viewport: Vec2) {
    let layout = title_button_layout(viewport.x, viewport.y);
    let compact = viewport.x <= 900.0 || viewport.y <= 640.0;
    let logo = title_logo_rect(viewport.x, viewport.y, compact);
    let start = start_image_rect(ScreenRect::from_tuple(layout.start), compact);

    queue_screen_color(
        world,
        logo,
        [0.015, 0.014, 0.018, 1.0],
        TITLE_MENU_BACKING_Z,
    );
    queue_screen_image(world, logo, TITLE_LOGO_PLAQUE_PATH, TITLE_MENU_LOGO_Z);
    queue_screen_color(
        world,
        start,
        [0.035, 0.012, 0.012, 1.0],
        TITLE_MENU_BACKING_Z,
    );
    queue_screen_image(world, start, MENU_BUTTON_START_PATH, TITLE_MENU_BUTTON_Z);

    let paths = [
        MENU_BUTTON_CHARACTER_PATH,
        MENU_BUTTON_STAGE_PATH,
        MENU_BUTTON_SHOP_PATH,
        MENU_BUTTON_SETTINGS_PATH,
    ];
    for (path, rect) in paths.into_iter().zip(layout.buttons) {
        let rect = menu_button_image_rect(ScreenRect::from_tuple(rect), compact);
        queue_screen_color(
            world,
            rect,
            [0.026, 0.022, 0.024, 1.0],
            TITLE_MENU_BACKING_Z,
        );
        queue_screen_image(world, rect, path, TITLE_MENU_BUTTON_Z);
    }
}

fn title_logo_rect(vw: f32, vh: f32, compact: bool) -> ScreenRect {
    let w = if compact {
        (vw - 52.0).clamp(560.0, 660.0)
    } else {
        (vw * 0.76).clamp(760.0, 1040.0)
    };
    let h = if compact {
        (vh * 0.25).clamp(132.0, 168.0)
    } else {
        (vh * 0.29).clamp(190.0, 236.0)
    };
    ScreenRect {
        x: vw * 0.5 - w * 0.5,
        y: if compact { 36.0 } else { 28.0 },
        w,
        h,
    }
}

fn start_image_rect(rect: ScreenRect, compact: bool) -> ScreenRect {
    if compact {
        rect.inflated(28.0, 18.0)
    } else {
        rect.inflated(64.0, 36.0)
    }
}

fn menu_button_image_rect(rect: ScreenRect, compact: bool) -> ScreenRect {
    if compact {
        rect.inflated(16.0, 14.0)
    } else {
        rect.inflated(24.0, 20.0)
    }
}

fn queue_screen_image(world: &mut World, rect: ScreenRect, path: &str, z: f32) {
    let handle = survivor_texture_handle(world, path);
    if let Some(queue) = world.resource_mut::<UiImageQueue>() {
        queue.push(
            DrawImage::textured_with_handle(rect.x, rect.y, rect.w, rect.h, path, handle).with_z(z),
        );
    }
}

fn queue_screen_color(world: &mut World, rect: ScreenRect, color: [f32; 4], z: f32) {
    if let Some(queue) = world.resource_mut::<UiImageQueue>() {
        queue.push(DrawImage::colored(rect.x, rect.y, rect.w, rect.h, color).with_z(z));
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
        let logo = title_logo_rect(800.0, 600.0, true);
        let start = start_image_rect(ScreenRect::from_tuple(layout.start), true);
        assert_inside_viewport(logo, 800.0, 600.0);
        assert_inside_viewport(start, 800.0, 600.0);
        for rect in layout.buttons {
            assert_inside_viewport(
                menu_button_image_rect(ScreenRect::from_tuple(rect), true),
                800.0,
                600.0,
            );
        }
        assert!(logo.bottom() < start.y);
        assert!(
            start.bottom()
                < menu_button_image_rect(ScreenRect::from_tuple(layout.buttons[0]), true).y
        );
    }

    #[test]
    fn default_title_menu_images_fit_1280x720() {
        let layout = title_button_layout(1280.0, 720.0);
        assert_inside_viewport(title_logo_rect(1280.0, 720.0, false), 1280.0, 720.0);
        assert_inside_viewport(
            start_image_rect(ScreenRect::from_tuple(layout.start), false),
            1280.0,
            720.0,
        );
        for rect in layout.buttons {
            assert_inside_viewport(
                menu_button_image_rect(ScreenRect::from_tuple(rect), false),
                1280.0,
                720.0,
            );
        }
    }

    #[test]
    fn title_menu_images_queue_opaque_backings() {
        let mut world = World::new();
        world.insert_resource(UiImageQueue::default());

        queue_title_menu_images(&mut world, Vec2::new(1280.0, 720.0));

        let queue = world.resource::<UiImageQueue>().unwrap();
        let backing_count = queue
            .items
            .iter()
            .filter(|image| image.texture.is_none() && image.image_handle.is_none())
            .count();
        let textured_count = queue
            .items
            .iter()
            .filter(|image| image.texture.is_some())
            .count();

        assert_eq!(backing_count, 6);
        assert_eq!(textured_count, 6);
        assert!(queue
            .items
            .iter()
            .filter(|image| image.texture.is_none())
            .all(|image| image.color[3] >= 1.0 && image.z < TITLE_MENU_LOGO_Z));
    }
}
