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

fn bgm_playlist_advances(key: &str, variant_count: usize) -> bool {
    bgm_repeats(key) && variant_count > 1
}

fn play_bgm_file(audio: &mut AudioManager, path: &str, key: &str, variant_count: usize) {
    audio.play("bgm", path, !bgm_playlist_advances(key, variant_count));
}

fn normalize_existing_dir(path: PathBuf) -> Option<PathBuf> {
    if !path.is_dir() {
        return None;
    }
    std::fs::canonicalize(&path).ok().or(Some(path))
}

fn macos_app_audio_dir_for_exe(exe_path: &Path) -> Option<PathBuf> {
    let macos_dir = exe_path.parent()?;
    if macos_dir.file_name()? != "MacOS" {
        return None;
    }
    let contents_dir = macos_dir.parent()?;
    if contents_dir.file_name()? != "Contents" {
        return None;
    }
    normalize_existing_dir(contents_dir.join("Resources").join("assets").join("audio"))
}

fn candidate_audio_dirs_for_exe(exe_path: &Path, include_dev_fallback: bool) -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    if let Some(dir) = macos_app_audio_dir_for_exe(exe_path) {
        dirs.push(dir);
    }

    if let Some(exe_dir) = exe_path.parent() {
        if let Some(dir) = normalize_existing_dir(exe_dir.join("assets").join("audio")) {
            dirs.push(dir);
        }
        if let Some(parent_dir) = exe_dir.parent() {
            if let Some(dir) = normalize_existing_dir(parent_dir.join("assets").join("audio")) {
                dirs.push(dir);
            }
        }
    }

    if include_dev_fallback {
        let dev_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../assets/audio");
        if let Some(dir) = normalize_existing_dir(dev_dir) {
            dirs.push(dir);
        }
    }

    dirs
}

fn resolve_audio_base_from_exe(exe_path: &Path, include_dev_fallback: bool) -> Option<PathBuf> {
    candidate_audio_dirs_for_exe(exe_path, include_dev_fallback)
        .into_iter()
        .next()
}

fn resolve_audio_base_dir() -> Option<PathBuf> {
    let exe_path = std::env::current_exe().ok()?;
    resolve_audio_base_from_exe(&exe_path, cfg!(debug_assertions))
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
    /// 실행 파일/번들 기준으로 해석한 상황별 BGM playlist 캐시.
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
        let slot = bgm_slot(target);
        let Some(playlists) = self.ensure_playlists() else {
            log::warn!(
                "BGM audio root not found for key {target}; checked executable-relative paths only"
            );
            if let Some(audio) = world.resource_mut::<AudioManager>() {
                audio.stop("bgm");
            }
            self.current = Some(target);
            return;
        };
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
                let next_path = if bgm_playlist_advances(target, variant_count)
                    && audio.is_finished("bgm") == Some(true)
                {
                    let variant_index = self.next_variants[slot];
                    let path = self.playlists.as_ref().unwrap()[slot][variant_index % variant_count]
                        .clone();
                    self.next_variants[slot] = variant_index.wrapping_add(1);
                    Some(path)
                } else {
                    None
                };
                if bgm_playlist_advances(target, variant_count)
                    && next_path.is_some()
                {
                    play_bgm_file(audio, next_path.as_deref().unwrap(), target, variant_count);
                }
                audio.set_volume("bgm", volume);
            }
            return; // 같은 상황 유지 — 볼륨 갱신 또는 playlist 다음 곡 처리
        }

        let variant_index = self.next_variants[slot];
        let path = self.playlists.as_ref().unwrap()[slot][variant_index % variant_count].clone();
        self.next_variants[slot] = variant_index.wrapping_add(1);

        let volume = world
            .resource::<MetaSave>()
            .map(|m| m.bgm_volume)
            .unwrap_or(1.0);
        if let Some(audio) = world.resource_mut::<AudioManager>() {
            play_bgm_file(audio, &path, target, variant_count);
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
        assert!(!bgm_playlist_advances("bgm_title", 1));
        assert!(!bgm_playlist_advances("bgm_stageclear", 2));
        assert!(!bgm_playlist_advances("bgm_gameover", 2));
    }

    #[test]
    fn bgm_variant_slots_are_stable() {
        assert_eq!(bgm_slot("bgm_title"), 0);
        assert_eq!(bgm_slot("bgm_ingame"), 1);
        assert_eq!(bgm_slot("bgm_boss"), 2);
        assert_eq!(bgm_slot("bgm_stageclear"), 3);
        assert_eq!(bgm_slot("bgm_gameover"), 4);
    }

    #[test]
    fn macos_bundle_audio_dir_has_priority() {
        let root = Path::new("/tmp/RustSurvivors.app/Contents");
        let exe = root.join("MacOS/RustSurvivors");
        let dir = macos_app_audio_dir_for_exe(&exe).unwrap_or_else(|| {
            root.join("Resources").join("assets").join("audio")
        });

        assert_eq!(dir, root.join("Resources").join("assets").join("audio"));
    }

    #[test]
    fn executable_relative_dirs_are_checked_before_dev_fallback() {
        let temp = std::env::temp_dir().join("rust_survivors_bgm_test_exe_root");
        let exe = temp.join("bin/survivor");
        let candidates = candidate_audio_dirs_for_exe(&exe, true);

        let manifest = manifest_audio_dir();
        assert_eq!(candidates.last(), Some(&manifest));
    }

    #[test]
    fn no_relative_cwd_fallback_is_used() {
        let temp = std::env::temp_dir().join("rust_survivors_bgm_test_no_cwd");
        let exe = temp.join("bin/survivor");

        assert!(resolve_audio_base_from_exe(&exe, false).is_none());
    }

    #[test]
    fn missing_asset_resolves_to_none() {
        let base = manifest_audio_dir();
        assert!(resolve_audio_file_from_base(&base, "does_not_exist").is_none());
    }
}
