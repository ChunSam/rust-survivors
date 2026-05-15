use std::sync::Arc;
use std::time::Instant;

use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

use crate::{
    components::GameState,
    ecs::{System, World},
    input::InputState,
    renderer::{GpuContext, SpriteRenderer},
};

/// 엔진 진입점.
///
/// # 사용법
/// ```rust,no_run
/// # use engine::App;
/// let mut app = App::new();
/// app.world.spawn();
/// // app.add_system(MySystem);
/// app.run();
/// ```
pub struct App {
    /// ECS 세계 (엔티티·컴포넌트·리소스)
    pub world:       World,
    /// 배경색 (RGBA, 선형 공간)
    pub clear_color: wgpu::Color,

    systems:          Vec<Box<dyn System>>,
    window:           Option<Arc<Window>>,
    gpu:              Option<GpuContext>,
    sprite_renderer:  Option<SpriteRenderer>,
    last_frame:       Option<Instant>,
    /// GPU 초기화 전에 등록된 텍스처 경로를 보관한다. resumed()에서 실제로 로드한다.
    pending_textures: Vec<String>,
}

impl App {
    pub fn new() -> Self {
        let mut world = World::new();
        world.insert_resource(InputState::default());
        world.insert_resource(GameState::Playing);
        Self {
            world,
            clear_color: wgpu::Color { r: 0.08, g: 0.08, b: 0.12, a: 1.0 },
            systems:          Vec::new(),
            window:           None,
            gpu:              None,
            sprite_renderer:  None,
            last_frame:       None,
            pending_textures: Vec::new(),
        }
    }

    /// 시스템을 등록한다. 매 프레임 등록 순서대로 실행된다.
    pub fn add_system<S: System + 'static>(&mut self, system: S) {
        self.systems.push(Box::new(system));
    }

    /// PNG 텍스처를 로드 대기열에 추가한다.
    ///
    /// GPU가 준비되기 전(`run()` 호출 전)에도 안전하게 호출할 수 있다.
    /// 실제 GPU 업로드는 `resumed()` 시점에 일괄 처리된다.
    pub fn load_texture(&mut self, path: impl Into<String>) {
        self.pending_textures.push(path.into());
    }

    /// ECS 월드를 초기화하고 기본 리소스를 재삽입한다.
    ///
    /// 씬 전환 시 엔티티·컴포넌트를 전부 지우고 싶을 때 사용한다.
    /// 시스템은 유지되므로 필요하면 `add_system`으로 새로 등록한다.
    pub fn reload_scene(&mut self) {
        self.world = World::new();
        self.world.insert_resource(InputState::default());
        self.world.insert_resource(GameState::Playing);
    }

    /// 이벤트 루프를 시작한다. 창이 닫힐 때까지 블로킹된다.
    pub fn run(mut self) {
        let event_loop = EventLoop::new().expect("이벤트 루프 생성 실패");
        event_loop.run_app(&mut self).expect("이벤트 루프 오류");
    }

    // ── 내부 메서드 ─────────────────────────────────────────────────────────

    fn update(&mut self, dt: f32) {
        for system in &mut self.systems {
            system.run(&mut self.world, dt);
        }
        if let Some(input) = self.world.resource_mut::<InputState>() {
            input.flush();
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let gpu = match self.gpu.as_mut() {
            Some(g) => g,
            None    => return Ok(()),
        };

        let frame   = gpu.surface.get_current_texture()?;
        let view    = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut enc = gpu.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { label: Some("frame encoder") },
        );

        // 1단계: 배경 Clear
        {
            let _pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("clear pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view:           &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load:  wgpu::LoadOp::Clear(self.clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set:      None,
                timestamp_writes:         None,
            });
        }

        // 2단계: 스프라이트 그리기
        if let Some(sr) = &mut self.sprite_renderer {
            sr.render(
                &gpu.device,
                &gpu.queue,
                &view,
                &mut enc,
                &self.world,
                gpu.config.width,
                gpu.config.height,
            );
        }

        gpu.queue.submit(std::iter::once(enc.finish()));
        frame.present();
        Ok(())
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

// ─── winit ApplicationHandler 구현 ───────────────────────────────────────────

impl ApplicationHandler for App {
    /// 앱이 활성화될 때 호출 (macOS: Resumed, 기타: 시작 시 1회)
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attrs = Window::default_attributes()
            .with_title("Rust 2D Engine")
            .with_inner_size(winit::dpi::LogicalSize::new(800u32, 600u32));
        let window = Arc::new(event_loop.create_window(attrs).expect("창 생성 실패"));

        let gpu             = pollster::block_on(GpuContext::new(window.clone()));
        let mut sprite_renderer = SpriteRenderer::new(&gpu.device, &gpu.queue, gpu.config.format);

        // 대기열에 있던 텍스처를 GPU에 일괄 로드한다.
        for path in self.pending_textures.drain(..) {
            sprite_renderer.load_texture(&gpu.device, &gpu.queue, &path);
        }

        self.sprite_renderer = Some(sprite_renderer);
        self.gpu             = Some(gpu);
        self.window          = Some(window);
        self.last_frame      = Some(Instant::now());

        log::info!("엔진 초기화 완료");
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _id:        WindowId,
        event:      WindowEvent,
    ) {
        match event {
            // ── 창 닫기 ──────────────────────────────────────────────────────
            WindowEvent::CloseRequested => event_loop.exit(),

            // ── 창 크기 변경 ─────────────────────────────────────────────────
            WindowEvent::Resized(size) => {
                if let Some(gpu) = &mut self.gpu {
                    gpu.resize(size);
                }
            }

            // ── 키보드 입력 ──────────────────────────────────────────────────
            WindowEvent::KeyboardInput {
                event: KeyEvent {
                    physical_key: PhysicalKey::Code(key),
                    state,
                    ..
                },
                ..
            } => {
                if let Some(input) = self.world.resource_mut::<InputState>() {
                    match state {
                        ElementState::Pressed  => input.press(key),
                        ElementState::Released => input.release(key),
                    }
                }
                if key == KeyCode::Escape {
                    event_loop.exit();
                }
            }

            // ── 프레임 렌더 ──────────────────────────────────────────────────
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let dt  = self.last_frame
                    .map(|t| (now - t).as_secs_f32().min(0.1))
                    .unwrap_or(1.0 / 60.0);
                self.last_frame = Some(now);

                self.update(dt);
                match self.render() {
                    Ok(()) => {}
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        if let Some(gpu) = &self.gpu {
                            gpu.reconfigure();
                        }
                    }
                    Err(e) => log::error!("렌더링 오류: {e:?}"),
                }
            }

            _ => {}
        }
    }

    /// 이벤트 큐가 비었을 때 → 매 프레임 redraw 요청
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}
