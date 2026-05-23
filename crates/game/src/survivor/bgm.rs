//! BGM 자동 전환 시스템 — Phase 11-H.
//!
//! ## 흐름
//! - `SurvivorMode` 가 바뀔 때마다 대응 BGM 파일을 `AudioManager::play("bgm", ...)` 로 교체.
//! - `GameState::GameOver` 진입 시 별도 gameover 트랙 재생.
//! - 같은 트랙을 다시 요청하면 restart 하지 않음 (모드 재진입 방지).
//!
//! ## 음원 파일 규격
//! `assets/audio/` 아래 WAV 또는 OGG (rodio vorbis feature 활성화).
//! 플레이스홀더 파일을 교체하면 즉시 반영된다.

use engine::audio::AudioManager;
use engine::components::GameState;
use engine::{System, World};

use super::boss::Boss;
use super::meta::MetaSave;
use super::meta::SurvivorMode;

/// 모드 → BGM 파일명 매핑.
fn bgm_key(mode: SurvivorMode, state: &GameState, boss_active: bool) -> &'static str {
    match (mode, state, boss_active) {
        (_, GameState::GameOver, _) => "bgm_gameover",
        (SurvivorMode::StageClear, _, _) => "bgm_stageclear",
        (SurvivorMode::InGame, _, true) => "bgm_boss",
        (SurvivorMode::InGame, _, false) => "bgm_ingame",
        _ => "bgm_title", // Title / CharacterSelect / StageSelect / Shop
    }
}

/// BGM 파일이 위치하는 디렉토리.
const AUDIO_DIR: &str = "assets/audio";

/// 지원 확장자 목록 — 먼저 발견되는 것 사용.
const EXTENSIONS: &[&str] = &["ogg", "wav"];

fn find_audio_file(key: &str) -> Option<String> {
    for ext in EXTENSIONS {
        let path = format!("{AUDIO_DIR}/{key}.{ext}");
        if std::path::Path::new(&path).exists() {
            return Some(path);
        }
    }
    None
}

/// SurvivorMode 전환 시 BGM 파일을 자동 교체.
///
/// `GameState::GameOver` 에서도 gameover 트랙으로 전환한다.
pub struct BgmSystem {
    /// 현재 재생 중인 BGM 키 ("bgm_title" 등). None 이면 아직 미재생.
    current: Option<&'static str>,
}

impl Default for BgmSystem {
    fn default() -> Self {
        Self { current: None }
    }
}

impl System for BgmSystem {
    fn run(&mut self, world: &mut World, _dt: f32) {
        let mode = world
            .resource::<SurvivorMode>()
            .copied()
            .unwrap_or(SurvivorMode::Title);
        let state = world
            .resource::<GameState>()
            .cloned()
            .unwrap_or(GameState::Playing);
        let boss_active = world.query::<Boss>().next().is_some();

        let target = bgm_key(mode, &state, boss_active);

        if self.current == Some(target) {
            let volume = world
                .resource::<MetaSave>()
                .map(|m| m.bgm_volume)
                .unwrap_or(1.0);
            if let Some(audio) = world.resource_mut::<AudioManager>() {
                audio.set_volume("bgm", volume);
            }
            return; // 이미 재생 중인 트랙 — 아무것도 하지 않음
        }

        let path = match find_audio_file(target) {
            Some(p) => p,
            None => {
                // 파일이 없으면 BGM 정지하고 키는 갱신
                if let Some(audio) = world.resource_mut::<AudioManager>() {
                    audio.stop("bgm");
                }
                self.current = Some(target);
                return;
            }
        };

        // gameover / stageclear 트랙은 한 번만 재생, 나머지는 루프
        let repeat = !matches!(target, "bgm_gameover" | "bgm_stageclear");

        let volume = world
            .resource::<MetaSave>()
            .map(|m| m.bgm_volume)
            .unwrap_or(1.0);
        if let Some(audio) = world.resource_mut::<AudioManager>() {
            audio.play("bgm", &path, repeat);
            audio.set_volume("bgm", volume);
        }
        self.current = Some(target);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bgm_key_switches_to_boss_track_only_during_ingame_boss() {
        assert_eq!(
            bgm_key(SurvivorMode::InGame, &GameState::Playing, false),
            "bgm_ingame"
        );
        assert_eq!(
            bgm_key(SurvivorMode::InGame, &GameState::Playing, true),
            "bgm_boss"
        );
        assert_eq!(
            bgm_key(SurvivorMode::StageClear, &GameState::Paused, true),
            "bgm_stageclear"
        );
        assert_eq!(
            bgm_key(SurvivorMode::InGame, &GameState::GameOver, true),
            "bgm_gameover"
        );
    }
}
