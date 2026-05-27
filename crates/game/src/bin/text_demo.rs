//! 텍스트 렌더링 데모 — 한글(NotoSansKR 동봉) + 영문 출력.

use engine::{App, DrawText, System, TextQueue, World};
use glam::Vec2;

/// 매 프레임 한글·영문 텍스트를 큐에 푸시하는 시스템.
struct TextDemoSystem {
    elapsed: f32,
}

impl System for TextDemoSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        self.elapsed += dt;
        let fps = if dt > 0.0 { 1.0 / dt } else { 0.0 };

        if let Some(q) = world.resource_mut::<TextQueue>() {
            q.push(DrawText::new(
                "Rust 2D Game Engine",
                Vec2::new(40.0, 40.0),
                32.0,
                [255, 255, 255, 255],
            ));
            q.push(DrawText::new(
                "안녕하세요! 한글이 보입니다.",
                Vec2::new(40.0, 90.0),
                28.0,
                [255, 220, 120, 255],
            ));
            q.push(DrawText::new(
                format!("Elapsed: {:.1}s   FPS: {:.0}", self.elapsed, fps),
                Vec2::new(40.0, 140.0),
                20.0,
                [180, 220, 255, 255],
            ));
            q.push(DrawText::new(
                "ESC 키로 종료",
                Vec2::new(40.0, 540.0),
                18.0,
                [200, 200, 200, 255],
            ));
        }
    }
}

fn main() {
    env_logger::init();

    let mut app = App::new();
    app.add_system(TextDemoSystem { elapsed: 0.0 });
    app.run();
}
