pub mod animation;
pub mod app;
pub mod audio;
pub mod components;
pub mod ecs;
pub mod input;
pub mod physics;
pub mod renderer;

// 편의를 위한 최상위 재수출
pub use animation::AnimationSystem;
pub use app::App;
pub use audio::AudioManager;
pub use components::{AnimationClip, AnimationPlayer, GameState, Sprite, Transform, UvRect};
pub use ecs::{Entity, System, World};
pub use input::InputState;
pub use physics::{PhysicsBody, PhysicsSystem, PhysicsWorld};
