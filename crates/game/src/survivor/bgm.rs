//! BGM 자동 전환 시스템 — Phase 11-H.
//!
//! ## 흐름
//! - `SurvivorMode` 가 바뀔 때마다 대응 BGM 파일을 `AudioManager::play("bgm", ...)` 로 교체.
//! - `GameState::GameOver` 진입 시 별도 gameover 트랙 재생.
//! - 같은 트랙을 다시 요청하면 restart 하지 않음 (모드 재진입 방지).
//! - 반복형 BGM 은 곡이 자연 종료되면 다음 variant 로 이어서 재생.
//!
//! ## 음원 파일 규격
//! `assets/audio/` 아래 MP3.
//! 플레이스홀더 파일을 교체하면 즉시 반영된다.

use std::array;
use std::path::{Path, PathBuf};

use engine::{AudioManager, GameState, System, World};

use super::asset_root;
use super::boss::Boss;
use super::meta::MetaSave;
use super::meta::SurvivorMode;

const BGM_TRACK_COUNT: usize = 5;
const BGM_KEYS: [&str; BGM_TRACK_COUNT] = [
    "bgm_title",
    "bgm_ingame",
    "bgm_boss",
    "bgm_stageclear",
    "bgm_gameover",
];

/// BGM 은 최종 교체 음원인 MP3 만 사용한다.
const EXTENSION: &str = "mp3";

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

fn bgm_slot(key: &str) -> usize {
    match key {
        "bgm_title" => 0,
        "bgm_ingame" => 1,
        "bgm_boss" => 2,
        "bgm_stageclear" => 3,
        "bgm_gameover" => 4,
        _ => 0,
    }
}

fn bgm_variant_stems(key: &str) -> &'static [&'static str] {
    match key {
        "bgm_title" => &["rustsurvivors title1", "rustsurvivors title2"],
        "bgm_ingame" => &["rustsurvivors ingame1", "rustsurvivors ingame2"],
        "bgm_boss" => &["rustsurvivors boss1", "rustsurvivors boss2"],
        "bgm_stageclear" => &["rustsurvivors stageclear1", "rustsurvivors stageclear2"],
        "bgm_gameover" => &["rustsurvivors gameover1", "rustsurvivors gameover2"],
        _ => &[],
    }
}

fn bgm_repeats(key: &str) -> bool {
    !matches!(key, "bgm_gameover" | "bgm_stageclear")
}

fn bgm_restarts_after_finish(key: &str) -> bool {
    bgm_repeats(key)
}

#[cfg(test)]
fn bgm_playlist_advances(key: &str, variant_count: usize) -> bool {
    bgm_restarts_after_finish(key) && variant_count > 1
}

fn bgm_repeat_flag(_key: &str) -> bool {
    false
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum BgmAction {
    KeepCurrent,
    PlayPath(String),
}

fn choose_bgm_action(
    current: Option<&'static str>,
    target: &'static str,
    playlist: &[String],
    next_variant: &mut usize,
    finished: Option<bool>,
) -> Option<BgmAction> {
    if playlist.is_empty() {
        return None;
    }

    if current == Some(target) {
        if bgm_restarts_after_finish(target) && finished == Some(true) {
            let path = playlist[*next_variant % playlist.len()].clone();
            *next_variant = next_variant.wrapping_add(1);
            return Some(BgmAction::PlayPath(path));
        }
        return Some(BgmAction::KeepCurrent);
    }

    let path = playlist[*next_variant % playlist.len()].clone();
    *next_variant = next_variant.wrapping_add(1);
    Some(BgmAction::PlayPath(path))
}

fn play_bgm_file(audio: &mut AudioManager, path: &str, key: &str) {
    audio.play("bgm", path, bgm_repeat_flag(key));
}

/// Audio lives at `<asset root>/assets/audio`. Root discovery (macOS bundle Resources,
/// then executable-relative, then working-directory-relative) belongs to [`asset_root`],
/// which owns it for textures and SFX too - one policy, stated in one place.
///
/// Executable-relative candidates still win over the working directory there, so a packaged
/// build never plays audio from a stray folder it happened to be launched next to.
fn resolve_audio_base_dir() -> Option<PathBuf> {
    let dir = asset_root::resolve_asset_root()?
        .join("assets")
        .join("audio");
    dir.is_dir().then_some(dir)
}

fn resolve_audio_file_from_base(base_dir: &Path, stem: &str) -> Option<String> {
    let path = base_dir.join(format!("{stem}.{EXTENSION}"));
    if !path.exists() {
        return None;
    }
    let absolute = std::fs::canonicalize(&path).ok().unwrap_or(path);
    Some(absolute.to_string_lossy().into_owned())
}

fn build_bgm_playlist_from_base(base_dir: &Path, key: &str) -> Vec<String> {
    bgm_variant_stems(key)
        .iter()
        .filter_map(|stem| resolve_audio_file_from_base(base_dir, stem))
        .collect()
}

fn build_cached_playlists() -> Option<[Vec<String>; BGM_TRACK_COUNT]> {
    let base_dir = resolve_audio_base_dir()?;
    Some(array::from_fn(|slot| {
        build_bgm_playlist_from_base(&base_dir, BGM_KEYS[slot])
    }))
}

/// SurvivorMode 전환 시 BGM 파일을 자동 교체.
///
/// `GameState::GameOver` 에서도 gameover 트랙으로 전환한다.
pub struct BgmSystem {
    /// 현재 재생 중인 BGM 키 ("bgm_title" 등). None 이면 아직 미재생.
    current: Option<&'static str>,
    /// 상황별 다음 BGM variant 인덱스. 상황 재진입 시 1 → 2 → 1 순서로 고른다.
    next_variants: [usize; BGM_TRACK_COUNT],
    /// 에셋 루트(`asset_root`) 기준으로 해석한 상황별 BGM playlist 캐시.
    playlists: Option<[Vec<String>; BGM_TRACK_COUNT]>,
}

impl Default for BgmSystem {
    fn default() -> Self {
        Self {
            current: None,
            next_variants: [0; BGM_TRACK_COUNT],
            playlists: None,
        }
    }
}

impl BgmSystem {
    fn ensure_playlists(&mut self) -> Option<&[Vec<String>; BGM_TRACK_COUNT]> {
        if self.playlists.is_none() {
            self.playlists = build_cached_playlists();
        }
        self.playlists.as_ref()
    }

    fn choose_action(&mut self, target: &'static str, finished: Option<bool>) -> Option<BgmAction> {
        let slot = bgm_slot(target);
        let playlist = self.ensure_playlists()?[slot].clone();
        choose_bgm_action(
            self.current,
            target,
            &playlist,
            &mut self.next_variants[slot],
            finished,
        )
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
        let Some(playlists) = self.ensure_playlists() else {
            log::warn!(
                "BGM audio root not found for key {target}; no assets/audio under the resolved asset root"
            );
            if let Some(audio) = world.resource_mut::<AudioManager>() {
                audio.stop("bgm");
            }
            self.current = Some(target);
            return;
        };
        let slot = bgm_slot(target);
        let variant_count = playlists[slot].len();

        if variant_count == 0 {
            log::warn!("BGM playlist is empty for key {target}; stopping channel");
            if let Some(audio) = world.resource_mut::<AudioManager>() {
                audio.stop("bgm");
            }
            self.current = Some(target);
            return;
        }

        if self.current == Some(target) {
            let volume = world
                .resource::<MetaSave>()
                .map(|m| m.bgm_volume)
                .unwrap_or(1.0);
            if let Some(audio) = world.resource_mut::<AudioManager>() {
                let finished = audio.is_finished("bgm");
                if let Some(BgmAction::PlayPath(path)) = self.choose_action(target, finished) {
                    play_bgm_file(audio, &path, target);
                }
                audio.set_volume("bgm", volume);
            }
            return; // 같은 상황 유지 — 볼륨 갱신 또는 playlist 다음 곡 처리
        }

        let volume = world
            .resource::<MetaSave>()
            .map(|m| m.bgm_volume)
            .unwrap_or(1.0);
        if let Some(audio) = world.resource_mut::<AudioManager>() {
            if let Some(BgmAction::PlayPath(path)) = self.choose_action(target, Some(false)) {
                play_bgm_file(audio, &path, target);
            }
            audio.set_volume("bgm", volume);
        }
        self.current = Some(target);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest_audio_dir() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../assets/audio")
            .canonicalize()
            .expect("assets/audio should exist")
    }

    fn test_variant_count(key: &str) -> usize {
        let base = manifest_audio_dir();
        build_bgm_playlist_from_base(&base, key).len()
    }

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

    #[test]
    fn bgm_variants_exist_for_each_survivor_situation() {
        for key in BGM_KEYS {
            assert_eq!(
                test_variant_count(key),
                2,
                "expected two BGM variants for key {key}"
            );
        }
    }

    #[test]
    fn bgm_file_selection_wraps_between_available_variants() {
        let base = manifest_audio_dir();
        let variants = build_bgm_playlist_from_base(&base, "bgm_title");

        assert_eq!(variants.len(), 2);
        assert!(variants[0].ends_with("rustsurvivors title1.mp3"));
        assert!(variants[1].ends_with("rustsurvivors title2.mp3"));
        assert_eq!(variants[0], variants[2 % variants.len()]);
    }

    #[test]
    fn bgm_repeat_flag_is_false_for_one_shot_keys() {
        // All survivor BGM is re-queued manually so finished states stay observable.
        assert!(!bgm_repeat_flag("bgm_stageclear"));
        assert!(!bgm_repeat_flag("bgm_gameover"));
        assert!(!bgm_repeat_flag("bgm_title"));
        assert!(!bgm_repeat_flag("bgm_ingame"));
        assert!(!bgm_repeat_flag("bgm_boss"));
    }

    #[test]
    fn bgm_loop_policy_keeps_clear_and_gameover_one_shot() {
        assert!(bgm_repeats("bgm_title"));
        assert!(bgm_repeats("bgm_ingame"));
        assert!(bgm_repeats("bgm_boss"));
        assert!(!bgm_repeats("bgm_stageclear"));
        assert!(!bgm_repeats("bgm_gameover"));
    }

    #[test]
    fn bgm_playlist_advances_only_for_repeating_multi_track_bgm() {
        assert!(bgm_playlist_advances("bgm_title", 2));
        assert!(bgm_playlist_advances("bgm_ingame", 2));
        assert!(bgm_playlist_advances("bgm_boss", 2));
        assert!(bgm_restarts_after_finish("bgm_title"));
        assert!(!bgm_playlist_advances("bgm_title", 1));
        assert!(!bgm_playlist_advances("bgm_stageclear", 2));
        assert!(!bgm_playlist_advances("bgm_gameover", 2));
        assert!(!bgm_restarts_after_finish("bgm_stageclear"));
        assert!(!bgm_restarts_after_finish("bgm_gameover"));
    }

    #[test]
    fn bgm_variant_slots_are_stable() {
        assert_eq!(bgm_slot("bgm_title"), 0);
        assert_eq!(bgm_slot("bgm_ingame"), 1);
        assert_eq!(bgm_slot("bgm_boss"), 2);
        assert_eq!(bgm_slot("bgm_stageclear"), 3);
        assert_eq!(bgm_slot("bgm_gameover"), 4);
    }

    /// Audio resolution is delegated to `asset_root`, which is what enforces the priority
    /// order (bundle Resources, then executable-relative, then working directory) and is
    /// tested there. This asserts the delegation itself lands on the repo's audio directory.
    #[test]
    fn audio_base_dir_resolves_under_the_asset_root() {
        let base =
            resolve_audio_base_dir().expect("repo audio dir should resolve under cargo test");

        assert_eq!(base, manifest_audio_dir());
    }

    #[test]
    fn missing_asset_resolves_to_none() {
        let base = manifest_audio_dir();
        assert!(resolve_audio_file_from_base(&base, "does_not_exist").is_none());
    }

    #[test]
    fn repeating_bgm_requeues_next_variant_after_finish() {
        let playlist = vec!["a.mp3".to_string(), "b.mp3".to_string()];
        let mut next_variant = 1;

        let action = choose_bgm_action(
            Some("bgm_title"),
            "bgm_title",
            &playlist,
            &mut next_variant,
            Some(true),
        );

        assert_eq!(action, Some(BgmAction::PlayPath("b.mp3".to_string())));
        assert_eq!(next_variant, 2);
    }

    #[test]
    fn repeating_bgm_does_not_restart_while_still_playing() {
        let playlist = vec!["a.mp3".to_string(), "b.mp3".to_string()];
        let mut next_variant = 1;

        let action = choose_bgm_action(
            Some("bgm_title"),
            "bgm_title",
            &playlist,
            &mut next_variant,
            Some(false),
        );

        assert_eq!(action, Some(BgmAction::KeepCurrent));
        assert_eq!(next_variant, 1);
    }

    #[test]
    fn one_shot_bgm_does_not_auto_advance_after_finish() {
        let playlist = vec!["clear1.mp3".to_string(), "clear2.mp3".to_string()];
        let mut next_variant = 1;

        let action = choose_bgm_action(
            Some("bgm_stageclear"),
            "bgm_stageclear",
            &playlist,
            &mut next_variant,
            Some(true),
        );

        assert_eq!(action, Some(BgmAction::KeepCurrent));
        assert_eq!(next_variant, 1);
    }

    #[test]
    fn single_variant_repeating_bgm_restarts_same_track_after_finish() {
        let playlist = vec!["solo.mp3".to_string()];
        let mut next_variant = 0;

        let action = choose_bgm_action(
            Some("bgm_ingame"),
            "bgm_ingame",
            &playlist,
            &mut next_variant,
            Some(true),
        );

        assert_eq!(action, Some(BgmAction::PlayPath("solo.mp3".to_string())));
        assert_eq!(next_variant, 1);
    }
}
