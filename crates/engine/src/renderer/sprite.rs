use std::collections::HashMap;
use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::components::{Sprite, Transform};
use crate::ecs::World;
use crate::renderer::texture::Texture;

// ─── GPU에 올라가는 버텍스 구조체 ─────────────────────────────────────────────
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct Vertex {
    position: [f32; 2],
    uv:       [f32; 2],
}

// 단위 쿼드: 중심 (0,0), 크기 1×1
const VERTICES: &[Vertex] = &[
    Vertex { position: [-0.5, -0.5], uv: [0.0, 1.0] },
    Vertex { position: [ 0.5, -0.5], uv: [1.0, 1.0] },
    Vertex { position: [ 0.5,  0.5], uv: [1.0, 0.0] },
    Vertex { position: [-0.5,  0.5], uv: [0.0, 0.0] },
];
const INDICES: &[u16] = &[0, 1, 2, 2, 3, 0];

// ─── 인스턴스(스프라이트 1개)의 GPU 데이터 ────────────────────────────────────
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct InstanceRaw {
    model: [[f32; 4]; 4], // 4×4 행렬
    color: [f32; 4],
}

impl InstanceRaw {
    fn from(transform: &Transform, sprite: &Sprite) -> Self {
        Self {
            model: transform.to_matrix().to_cols_array_2d(),
            color: sprite.color,
        }
    }

    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceRaw>() as u64,
            step_mode:    wgpu::VertexStepMode::Instance,
            attributes:   &[
                wgpu::VertexAttribute { offset: 0,  shader_location: 2, format: wgpu::VertexFormat::Float32x4 },
                wgpu::VertexAttribute { offset: 16, shader_location: 3, format: wgpu::VertexFormat::Float32x4 },
                wgpu::VertexAttribute { offset: 32, shader_location: 4, format: wgpu::VertexFormat::Float32x4 },
                wgpu::VertexAttribute { offset: 48, shader_location: 5, format: wgpu::VertexFormat::Float32x4 },
                wgpu::VertexAttribute { offset: 64, shader_location: 6, format: wgpu::VertexFormat::Float32x4 },
            ],
        }
    }
}

// ─── 카메라 유니폼 ─────────────────────────────────────────────────────────────
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

// ─── 스프라이트 렌더러 ─────────────────────────────────────────────────────────
pub struct SpriteRenderer {
    pipeline:           wgpu::RenderPipeline,
    vertex_buf:         wgpu::Buffer,
    index_buf:          wgpu::Buffer,
    instance_buf:       wgpu::Buffer,
    instance_capacity:  usize,
    camera_buf:         wgpu::Buffer,
    camera_bind_group:  wgpu::BindGroup,
    #[allow(dead_code)]
    texture_layout:     wgpu::BindGroupLayout,
    white_texture:      Texture,
    #[allow(dead_code)]
    texture_cache:      HashMap<String, Arc<Texture>>,
}

impl SpriteRenderer {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        // ── 셰이더 로드 (파일에서) ───────────────────────────────────────────
        let shader_src = std::fs::read_to_string("assets/shaders/sprite.wgsl")
            .expect("assets/shaders/sprite.wgsl 을 찾지 못했습니다");
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label:  Some("sprite shader"),
            source: wgpu::ShaderSource::Wgsl(shader_src.into()),
        });

        // ── 카메라 유니폼 버퍼 ──────────────────────────────────────────────
        let camera_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label:              Some("camera uniform"),
            size:               std::mem::size_of::<CameraUniform>() as u64,
            usage:              wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let camera_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label:   Some("camera layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding:    0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty:         wgpu::BindingType::Buffer {
                    ty:                 wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size:   None,
                },
                count: None,
            }],
        });
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label:   Some("camera bind group"),
            layout:  &camera_layout,
            entries: &[wgpu::BindGroupEntry {
                binding:  0,
                resource: camera_buf.as_entire_binding(),
            }],
        });

        // ── 텍스처 레이아웃 + 기본 흰색 텍스처 ─────────────────────────────
        let texture_layout = Texture::bind_group_layout(device);
        let white_texture  = Texture::white(device, queue, &texture_layout);

        // ── 렌더 파이프라인 ─────────────────────────────────────────────────
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label:                Some("sprite pipeline layout"),
            bind_group_layouts:   &[&camera_layout, &texture_layout],
            push_constant_ranges: &[],
        });
        let vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as u64,
            step_mode:    wgpu::VertexStepMode::Vertex,
            attributes:   &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2],
        };
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label:  Some("sprite pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module:      &shader,
                entry_point: "vs_main",
                buffers:     &[vertex_layout, InstanceRaw::layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module:      &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend:      Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology:          wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face:        wgpu::FrontFace::Ccw,
                cull_mode:         None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample:   wgpu::MultisampleState::default(),
            multiview:     None,
        });

        // ── 정적 버텍스·인덱스 버퍼 ────────────────────────────────────────
        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label:    Some("quad vertex"),
            contents: bytemuck::cast_slice(VERTICES),
            usage:    wgpu::BufferUsages::VERTEX,
        });
        let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label:    Some("quad index"),
            contents: bytemuck::cast_slice(INDICES),
            usage:    wgpu::BufferUsages::INDEX,
        });

        // ── 초기 인스턴스 버퍼 (128개 분량 예약) ───────────────────────────
        let capacity = 128;
        let instance_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label:              Some("instance buffer"),
            size:               (capacity * std::mem::size_of::<InstanceRaw>()) as u64,
            usage:              wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            vertex_buf,
            index_buf,
            instance_buf,
            instance_capacity: capacity,
            camera_buf,
            camera_bind_group,
            texture_layout,
            white_texture,
            texture_cache: HashMap::new(),
        }
    }

    /// 매 프레임: ECS World에서 스프라이트를 수집해 렌더링한다.
    pub fn render(
        &mut self,
        device:  &wgpu::Device,
        queue:   &wgpu::Queue,
        view:    &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        world:   &World,
        width:   u32,
        height:  u32,
    ) {
        // ── 카메라: 화면 픽셀 좌표계 직교 투영 ─────────────────────────────
        //   왼쪽 위가 (0,0), 오른쪽 아래가 (width, height)
        let proj = glam::Mat4::orthographic_rh(
            0.0, width as f32,
            height as f32, 0.0,
            -1.0, 1.0,
        );
        let cam = CameraUniform { view_proj: proj.to_cols_array_2d() };
        queue.write_buffer(&self.camera_buf, 0, bytemuck::bytes_of(&cam));

        // ── ECS에서 (Transform, Sprite) 쌍 수집 ────────────────────────────
        let mut instances: Vec<InstanceRaw> = Vec::new();
        for (entity, sprite) in world.query::<Sprite>() {
            if let Some(transform) = world.get::<Transform>(entity) {
                instances.push(InstanceRaw::from(transform, sprite));
            }
        }
        if instances.is_empty() {
            return;
        }

        // ── 인스턴스 버퍼 갱신 (용량 초과 시 재할당) ───────────────────────
        if instances.len() > self.instance_capacity {
            self.instance_capacity = instances.len().next_power_of_two();
            self.instance_buf = device.create_buffer(&wgpu::BufferDescriptor {
                label:              Some("instance buffer"),
                size:               (self.instance_capacity * std::mem::size_of::<InstanceRaw>()) as u64,
                usage:              wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
        queue.write_buffer(&self.instance_buf, 0, bytemuck::cast_slice(&instances));

        // ── 렌더 패스 ───────────────────────────────────────────────────────
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("sprite pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load:  wgpu::LoadOp::Load, // 배경색은 App이 먼저 Clear
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set:      None,
            timestamp_writes:         None,
        });

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.camera_bind_group, &[]);
        pass.set_bind_group(1, &self.white_texture.bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertex_buf.slice(..));
        pass.set_vertex_buffer(1, self.instance_buf.slice(..));
        pass.set_index_buffer(self.index_buf.slice(..), wgpu::IndexFormat::Uint16);
        pass.draw_indexed(0..INDICES.len() as u32, 0, 0..instances.len() as u32);
    }
}
