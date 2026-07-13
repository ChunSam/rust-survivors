//! Asset root resolution.
//!
//! Textures (`Sprite::textured` / `App::load_image`) and SFX are loaded through
//! `assets/...` relative paths, which the OS resolves against the process working
//! directory. Launching the executable from anywhere other than the repo root
//! (Explorer double-click, a shortcut, a packaged bundle) therefore fails every
//! texture load and the engine paints its magenta fallback over the whole screen.
//!
//! `set_working_dir_to_asset_root` runs once at startup and points the working
//! directory at whichever directory actually contains `assets/`, so the relative
//! paths hold no matter how the game was launched. Save files are unaffected: they
//! go through `engine::save::save_path`, which is rooted at the OS data dir.

use std::path::{Path, PathBuf};

/// How far to walk up from the executable / working directory while looking for an
/// `assets/` sibling. A dev build sits at `<root>/target/release/survivor.exe`, so the
/// root is 2 levels above the executable's directory; 3 leaves a little headroom.
const ANCESTOR_DEPTH: usize = 4;

/// `Foo.app/Contents/MacOS/foo` → `Foo.app/Contents/Resources`, where the packaging
/// script puts `assets/`.
fn macos_bundle_resources(exe: &Path) -> Option<PathBuf> {
    let macos_dir = exe.parent()?;
    if macos_dir.file_name()? != "MacOS" {
        return None;
    }
    let contents_dir = macos_dir.parent()?;
    if contents_dir.file_name()? != "Contents" {
        return None;
    }
    Some(contents_dir.join("Resources"))
}

fn push_unique(out: &mut Vec<PathBuf>, dir: PathBuf) {
    if !out.contains(&dir) {
        out.push(dir);
    }
}

/// Directories that may hold `assets/`, in priority order. Pure path arithmetic — the
/// filesystem is only consulted by [`resolve_asset_root`].
///
/// The packaged layout (assets beside the executable) wins over the working directory
/// so that a shipped build never silently reads a stray `assets/` it was launched from.
fn asset_root_candidates(exe: Option<&Path>, cwd: Option<&Path>) -> Vec<PathBuf> {
    let mut out = Vec::new();

    if let Some(exe) = exe {
        if let Some(resources) = macos_bundle_resources(exe) {
            push_unique(&mut out, resources);
        }
        if let Some(exe_dir) = exe.parent() {
            for ancestor in exe_dir.ancestors().take(ANCESTOR_DEPTH) {
                push_unique(&mut out, ancestor.to_path_buf());
            }
        }
    }

    if let Some(cwd) = cwd {
        for ancestor in cwd.ancestors().take(ANCESTOR_DEPTH) {
            push_unique(&mut out, ancestor.to_path_buf());
        }
    }

    out
}

fn holds_assets(dir: &Path) -> bool {
    dir.join("assets").is_dir()
}

/// First candidate directory that actually contains `assets/`.
pub fn resolve_asset_root() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok();
    let cwd = std::env::current_dir().ok();
    asset_root_candidates(exe.as_deref(), cwd.as_deref())
        .into_iter()
        .find(|dir| holds_assets(dir))
        .map(|dir| std::fs::canonicalize(&dir).unwrap_or(dir))
}

/// Points the working directory at the asset root. Call once, before any asset load.
pub fn set_working_dir_to_asset_root() {
    let Some(root) = resolve_asset_root() else {
        log::error!(
            "no assets/ directory found near the executable or working directory; \
             textures will render as the magenta fallback"
        );
        return;
    };

    match std::env::set_current_dir(&root) {
        Ok(()) => log::info!("asset root: {}", root.display()),
        Err(err) => log::error!(
            "failed to set working directory to asset root {}: {err}",
            root.display()
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn macos_bundle_resources_dir_is_the_first_candidate() {
        let exe = PathBuf::from("/Applications/Rust Survivors.app/Contents/MacOS/survivor");
        let candidates = asset_root_candidates(Some(&exe), None);

        assert_eq!(
            candidates.first().map(PathBuf::as_path),
            Some(Path::new(
                "/Applications/Rust Survivors.app/Contents/Resources"
            ))
        );
    }

    #[test]
    fn plain_executable_yields_no_bundle_candidate() {
        let exe = PathBuf::from("/opt/survivor/bin/survivor");
        let candidates = asset_root_candidates(Some(&exe), None);

        assert_eq!(
            candidates.first().map(PathBuf::as_path),
            Some(Path::new("/opt/survivor/bin"))
        );
    }

    #[test]
    fn executable_ancestors_reach_the_workspace_root_of_a_dev_build() {
        let exe = PathBuf::from("/work/rust-survivors/target/release/survivor");
        let candidates = asset_root_candidates(Some(&exe), None);

        assert!(candidates.contains(&PathBuf::from("/work/rust-survivors")));
    }

    #[test]
    fn executable_candidates_precede_working_directory_candidates() {
        let exe = PathBuf::from("/opt/survivor/bin/survivor");
        let cwd = PathBuf::from("/home/player");
        let candidates = asset_root_candidates(Some(&exe), Some(&cwd));

        let exe_dir = candidates
            .iter()
            .position(|dir| dir == Path::new("/opt/survivor/bin"))
            .expect("executable dir should be a candidate");
        let cwd_dir = candidates
            .iter()
            .position(|dir| dir == Path::new("/home/player"))
            .expect("working dir should be a candidate");

        assert!(exe_dir < cwd_dir);
    }

    #[test]
    fn candidates_are_deduplicated_when_exe_and_cwd_overlap() {
        let exe = PathBuf::from("/work/game/survivor");
        let cwd = PathBuf::from("/work/game");
        let candidates = asset_root_candidates(Some(&exe), Some(&cwd));

        let hits = candidates
            .iter()
            .filter(|dir| *dir == Path::new("/work/game"))
            .count();
        assert_eq!(hits, 1);
    }

    #[test]
    fn resolve_finds_the_repo_asset_root_from_the_test_working_directory() {
        let root = resolve_asset_root().expect("repo asset root should resolve under cargo test");

        assert!(root.join("assets").is_dir());
        assert!(root.join("assets/textures/survivor").is_dir());
    }
}
