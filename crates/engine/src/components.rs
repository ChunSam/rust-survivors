use glam::{Mat4, Quat, Vec2, Vec3};

// ─── 게임 상태 ─────────────────────────────────────────────────────────────────
/// 게임 상태 머신 값 (ECS 리소스로 삽입)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameState {
    Playing,
    Paused,
    GameOver,
}

/// 위치·크기·회전을 담는 컴포넌트
#[derive(Debug, Clone)]
pub struct Transform {
    pub position: Vec2,
    pub scale:    Vec2,
    /// 회전 각도 (라디안, Z축)
    pub rotation: f32,
}

impl Transform {
    pub fn new(position: Vec2, scale: Vec2, rotation: f32) -> Self {
        Self { position, scale, rotation }
    }

    /// ECS → GPU에 넘길 4×4 모델 행렬 생성
    pub fn to_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(
            Vec3::new(self.scale.x, self.scale.y, 1.0),
            Quat::from_rotation_z(self.rotation),
            Vec3::new(self.position.x, self.position.y, 0.0),
        )
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            scale:    Vec2::ONE * 64.0, // 기본 64×64 픽셀
            rotation: 0.0,
        }
    }
}

/// 스프라이트 외형을 담는 컴포넌트
#[derive(Debug, Clone)]
pub struct Sprite {
    /// 텍스처 파일 경로 (None이면 단색 사각형)
    pub texture: Option<String>,
    /// RGBA 색상 배율 (흰색 = 텍스처 원본)
    pub color: [f32; 4],
}

impl Sprite {
    pub fn colored(r: f32, g: f32, b: f32) -> Self {
        Self { texture: None, color: [r, g, b, 1.0] }
    }

    pub fn textured(path: impl Into<String>) -> Self {
        Self { texture: Some(path.into()), color: [1.0; 4] }
    }
}

impl Default for Sprite {
    fn default() -> Self {
        Self::colored(1.0, 1.0, 1.0)
    }
}

// ─── 스프라이트 애니메이션 ──────────────────────────────────────────────────────
/// 텍스처 내 한 프레임 영역을 UV 좌표로 표현
///
/// 예) 4열 2행 스프라이트시트의 (2열, 1행) 프레임:
/// `UvRect::from_grid(2, 1, 4, 2)`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UvRect {
    pub u_offset: f32,
    pub v_offset: f32,
    pub u_size:   f32,
    pub v_size:   f32,
}

impl UvRect {
    /// 텍스처 전체를 사용하는 기본값
    pub const FULL: Self = Self { u_offset: 0.0, v_offset: 0.0, u_size: 1.0, v_size: 1.0 };

    /// 그리드 형태 스프라이트시트에서 특정 프레임의 UV를 계산한다.
    /// `col`, `row`: 0부터 시작하는 프레임 위치
    pub fn from_grid(col: u32, row: u32, cols: u32, rows: u32) -> Self {
        let u_size = 1.0 / cols as f32;
        let v_size = 1.0 / rows as f32;
        Self {
            u_offset: col as f32 * u_size,
            v_offset: row as f32 * v_size,
            u_size,
            v_size,
        }
    }
}

/// 하나의 애니메이션 클립: 프레임 목록과 재생 속도
#[derive(Debug, Clone)]
pub struct AnimationClip {
    pub frames:  Vec<UvRect>,
    pub fps:     f32,
    pub looping: bool,
}

/// 엔티티에 붙이는 애니메이션 플레이어 컴포넌트
#[derive(Debug, Clone)]
pub struct AnimationPlayer {
    pub clips:         Vec<AnimationClip>,
    pub current_clip:  usize,
    pub current_frame: usize,
    /// 다음 프레임까지 누적된 시간(초)
    pub timer:         f32,
}

impl AnimationPlayer {
    pub fn new(clips: Vec<AnimationClip>) -> Self {
        Self { clips, current_clip: 0, current_frame: 0, timer: 0.0 }
    }

    /// 클립을 전환한다. 이미 재생 중인 클립이면 아무것도 하지 않는다.
    pub fn play(&mut self, clip_index: usize) {
        if self.current_clip != clip_index {
            self.current_clip  = clip_index;
            self.current_frame = 0;
            self.timer         = 0.0;
        }
    }

    /// 현재 프레임의 UV를 반환한다. 클립·프레임이 없으면 전체 텍스처를 사용한다.
    pub fn current_uv(&self) -> UvRect {
        self.clips
            .get(self.current_clip)
            .and_then(|c| c.frames.get(self.current_frame))
            .copied()
            .unwrap_or(UvRect::FULL)
    }
}
