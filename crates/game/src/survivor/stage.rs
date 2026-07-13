/// Phase 10: 다중 스테이지 3종 + StageSelect 화면 + 클리어 시 다음 스테이지 해금.
///
/// - `StageKind` — MadForest / InlaidLibrary / DairyPlant (3종).
/// - `SelectedStage` — 현재 선택된 스테이지 리소스.
/// - `StageCursor` — StageSelect 화면 커서 리소스.
/// - `StageSelectSystem` — W/S/Enter/Esc 입력 처리 + SpawnDirector 갱신.
use engine::{InputState, KeyCode, System, World};
use serde::{Deserialize, Serialize};

use super::director::{SpawnDirector, WaveDef, WavesFile};
use super::locale::{loc, Lang};
use super::meta::{MetaSave, SurvivorMode};

// ─── StageKind ───────────────────────────────────────────────────────────────

/// 스테이지 3종.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StageKind {
    #[default]
    MadForest,
    InlaidLibrary,
    DairyPlant,
}

impl StageKind {
    pub const ALL: &'static [StageKind] = &[
        StageKind::MadForest,
        StageKind::InlaidLibrary,
        StageKind::DairyPlant,
    ];

    pub fn label(self, lang: Lang) -> &'static str {
        match self {
            StageKind::MadForest => loc(lang, "광란의 숲 (1)", "Mad Forest (1)"),
            StageKind::InlaidLibrary => loc(lang, "상감 도서관 (2)", "Inlaid Library (2)"),
            StageKind::DairyPlant => loc(lang, "유제품 공장 (3)", "Dairy Plant (3)"),
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            StageKind::MadForest => "MadForest",
            StageKind::InlaidLibrary => "InlaidLibrary",
            StageKind::DairyPlant => "DairyPlant",
        }
    }

    /// 배경색 (HUD 표시용 색상 레퍼런스 — wgpu clear color 는 App 통제).
    pub fn background_label_color(self) -> [u8; 4] {
        match self {
            StageKind::MadForest => [100, 180, 100, 255],
            StageKind::InlaidLibrary => [180, 140, 80, 255],
            StageKind::DairyPlant => [200, 220, 240, 255],
        }
    }

    /// 이전 스테이지 — 클리어해야 해금. MadForest 는 None (기본 해금).
    pub fn prerequisite(self) -> Option<StageKind> {
        match self {
            StageKind::MadForest => None,
            StageKind::InlaidLibrary => Some(StageKind::MadForest),
            StageKind::DairyPlant => Some(StageKind::InlaidLibrary),
        }
    }

    /// 컴파일 타임에 임베딩된 스테이지별 wave RON 문자열.
    pub fn waves_ron(self) -> &'static str {
        match self {
            StageKind::MadForest => include_str!("../../../../assets/data/waves_mad_forest.ron"),
            StageKind::InlaidLibrary => {
                include_str!("../../../../assets/data/waves_inlaid_library.ron")
            }
            StageKind::DairyPlant => include_str!("../../../../assets/data/waves_dairy_plant.ron"),
        }
    }

    /// RON 파싱 → `Vec<WaveDef>`.
    pub fn load_waves(self) -> Vec<WaveDef> {
        let parsed: WavesFile = ron::from_str(self.waves_ron())
            .unwrap_or_else(|e| panic!("stage {} waves.ron 파싱 실패: {:?}", self.key(), e));
        parsed.waves
    }
}

// ─── 리소스 ──────────────────────────────────────────────────────────────────

/// 현재 선택된 스테이지를 보관하는 리소스.
#[derive(Debug, Default, Clone, Copy)]
pub struct SelectedStage(pub StageKind);

/// StageSelect 화면 커서.
#[derive(Debug, Default, Clone, Copy)]
pub struct StageCursor {
    pub index: usize,
}

// ─── StageSelectSystem ───────────────────────────────────────────────────────

/// StageSelect 모드에서 W/S/Enter/Esc 입력을 처리.
///
/// - W/↑ : 커서 위
/// - S/↓ : 커서 아래
/// - Enter : 선택 (해금 검증 포함) → SpawnDirector 갱신 + Title 복귀
/// - Esc  : 취소 → Title 복귀
pub struct StageSelectSystem;

impl System for StageSelectSystem {
    fn run(&mut self, world: &mut World, _dt: f32) {
        let mode = world
            .resource::<SurvivorMode>()
            .copied()
            .unwrap_or(SurvivorMode::Title);
        if !matches!(mode, SurvivorMode::StageSelect) {
            return;
        }

        // 입력 캐시 (borrow 분리)
        let (up, down, enter, esc) = {
            let i = match world.resource::<InputState>() {
                Some(i) => i,
                None => return,
            };
            (
                i.just_pressed(KeyCode::KeyW) || i.just_pressed(KeyCode::ArrowUp),
                i.just_pressed(KeyCode::KeyS) || i.just_pressed(KeyCode::ArrowDown),
                i.just_pressed(KeyCode::Enter),
                i.just_pressed(KeyCode::Escape),
            )
        };

        // unlocked_stages 캐시 (borrow 분리)
        let unlocked: Vec<String> = world
            .resource::<MetaSave>()
            .map(|m| m.unlocked_stages.clone())
            .unwrap_or_else(|| vec!["MadForest".to_string()]);

        // 커서 이동
        if up || down {
            if let Some(c) = world.resource_mut::<StageCursor>() {
                if up && c.index > 0 {
                    c.index -= 1;
                }
                if down && c.index < StageKind::ALL.len() - 1 {
                    c.index += 1;
                }
            }
        }

        // 선택 확정
        if enter {
            let idx = world
                .resource::<StageCursor>()
                .map(|c| c.index)
                .unwrap_or(0);
            let stage = StageKind::ALL[idx];

            // 해금 검증: 선행 stage 가 없거나(MadForest), 선행 stage 가 unlocked 목록에 있으면 진입 가능
            let is_unlocked = stage.prerequisite().is_none()
                || stage
                    .prerequisite()
                    .map(|p| unlocked.iter().any(|s| s == p.key()))
                    .unwrap_or(false);

            if is_unlocked {
                // SelectedStage 갱신
                if let Some(sel) = world.resource_mut::<SelectedStage>() {
                    sel.0 = stage;
                }
                // SpawnDirector waves 교체 — borrow 분리 패턴
                let waves = stage.load_waves();
                if let Some(d) = world.resource_mut::<SpawnDirector>() {
                    d.waves = waves;
                    d.spawn_elapsed = 0.0;
                }
                // Title 복귀
                if let Some(m) = world.resource_mut::<SurvivorMode>() {
                    *m = SurvivorMode::Title;
                }
                println!("Selected stage: {}", stage.label(Lang::En));
            } else {
                println!(
                    "{} locked — clear previous stage first",
                    stage.label(Lang::En)
                );
            }
        }

        // 취소
        if esc {
            if let Some(m) = world.resource_mut::<SurvivorMode>() {
                *m = SurvivorMode::Title;
            }
        }
    }
}

// ─── 테스트 ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn spawn_pressure(wave: &WaveDef) -> f32 {
        wave.count_per_burst.max(1) as f32 / wave.spawn_interval
    }

    #[test]
    fn stage_count_is_three() {
        assert_eq!(StageKind::ALL.len(), 3, "스테이지는 정확히 3종이어야 함");
    }

    #[test]
    fn mad_forest_waves_load() {
        let waves = StageKind::MadForest.load_waves();
        assert!(
            !waves.is_empty(),
            "MadForest waves.ron 파싱 결과가 비어있음"
        );
    }

    #[test]
    fn stage_prerequisite_chain() {
        assert_eq!(StageKind::MadForest.prerequisite(), None);
        assert_eq!(
            StageKind::InlaidLibrary.prerequisite(),
            Some(StageKind::MadForest),
        );
        assert_eq!(
            StageKind::DairyPlant.prerequisite(),
            Some(StageKind::InlaidLibrary),
        );
    }

    #[test]
    fn all_stage_waves_cover_30_minutes_without_gaps() {
        for stage in StageKind::ALL {
            let waves = stage.load_waves();
            assert!(!waves.is_empty(), "{stage:?} must define waves");
            assert!(
                (waves[0].start_time - 0.0).abs() < f32::EPSILON,
                "{stage:?} must start at 0:00"
            );

            for pair in waves.windows(2) {
                let current = &pair[0];
                let next = &pair[1];
                assert!(
                    current.end_time > current.start_time,
                    "{stage:?} wave has invalid duration"
                );
                assert!(
                    (current.end_time - next.start_time).abs() < f32::EPSILON,
                    "{stage:?} wave gap or overlap: {} -> {}",
                    current.end_time,
                    next.start_time
                );
            }

            let last = waves.last().unwrap();
            assert!(
                (last.end_time - 1800.0).abs() < f32::EPSILON,
                "{stage:?} must cover through 30:00"
            );
        }
    }

    #[test]
    fn stage_wave_pressure_ramps_up_over_run() {
        for stage in StageKind::ALL {
            let waves = stage.load_waves();
            let first = spawn_pressure(waves.first().unwrap());
            let final_wave = spawn_pressure(waves.last().unwrap());
            assert!(
                final_wave >= first * 2.0,
                "{stage:?} final pressure should be at least double opening pressure"
            );
            assert!(
                waves.iter().all(|w| (0.0..=0.25).contains(&w.elite_chance)),
                "{stage:?} elite chance should stay readable"
            );
        }
    }

    #[test]
    fn stages_have_distinct_enemy_identities() {
        let stage_enemy_sets: Vec<(StageKind, HashSet<String>)> = StageKind::ALL
            .iter()
            .map(|stage| {
                let enemies = stage
                    .load_waves()
                    .iter()
                    .flat_map(|wave| wave.enemies.iter().map(|enemy| enemy.kind.clone()))
                    .collect();
                (*stage, enemies)
            })
            .collect();

        for pair in stage_enemy_sets.windows(2) {
            let (left_stage, left) = &pair[0];
            let (right_stage, right) = &pair[1];
            assert_ne!(
                left, right,
                "{left_stage:?} and {right_stage:?} should not share identical enemy pools"
            );
        }

        let forest_opening: HashSet<_> = StageKind::MadForest.load_waves()[0]
            .enemies
            .iter()
            .map(|enemy| enemy.kind.clone())
            .collect();
        let library_opening: HashSet<_> = StageKind::InlaidLibrary.load_waves()[0]
            .enemies
            .iter()
            .map(|enemy| enemy.kind.clone())
            .collect();
        let dairy_opening: HashSet<_> = StageKind::DairyPlant.load_waves()[0]
            .enemies
            .iter()
            .map(|enemy| enemy.kind.clone())
            .collect();

        assert!(forest_opening.contains("Zombie"));
        assert!(library_opening.contains("Ghost"));
        assert!(dairy_opening.contains("Slime"));
        assert_ne!(forest_opening, library_opening);
        assert_ne!(library_opening, dairy_opening);
    }
}
