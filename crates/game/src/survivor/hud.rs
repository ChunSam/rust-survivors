use engine::{System, World};
use engine::components::GameState;
use engine::renderer::text::{DrawText, TextQueue};
use glam::Vec2;
use super::boss::Boss;
use super::character::{CharacterCursor, CharacterKind};
use super::health::Health;
use super::levelup::PendingLevelUp;
use super::meta::{MetaSave, SurvivorMode};
use super::passive::PassiveInventory;
use super::pickup::GoldWallet;
use super::player::Player;
use super::powerup::{PowerUpKind, ShopCursor};
use super::xp::XpAccumulator;

/// 게임 진행 통계. 매 프레임 누적/조회.
#[derive(Debug, Default)]
pub struct GameStats {
    pub elapsed: f32, // 누적 게임 시간(초). Playing 중에만 증가.
    pub kills:   u32, // 누적 처치 수.
}

pub struct HudSystem;

impl System for HudSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        // Phase 8-A: SurvivorMode 별 화면 분기
        let mode = world.resource::<SurvivorMode>().copied().unwrap_or(SurvivorMode::InGame);

        match mode {
            SurvivorMode::Title => {
                if let Some(q) = world.resource_mut::<TextQueue>() {
                    q.push(DrawText {
                        text:     "RUST SURVIVORS".to_string(),
                        position: Vec2::new(220.0, 180.0),
                        size:     56.0,
                        color:    [255, 220, 80, 255],
                    });
                    q.push(DrawText {
                        text:     "Press ENTER to start".to_string(),
                        position: Vec2::new(280.0, 280.0),
                        size:     22.0,
                        color:    [255, 255, 255, 255],
                    });
                    q.push(DrawText {
                        text:     "C = CHAR  S = SHOP".to_string(),
                        position: Vec2::new(310.0, 380.0),
                        size:     18.0,
                        color:    [200, 200, 255, 255],
                    });
                }
                // 메타 정보 표시
                let meta_info = world.resource::<MetaSave>().map(|m| {
                    (m.gold_total, m.best_time, m.kills_total)
                });
                if let Some((gold, best, kills)) = meta_info {
                    if let Some(q) = world.resource_mut::<TextQueue>() {
                        q.push(DrawText {
                            text:     format!(
                                "Gold {}  Best {:02}:{:02}  Kills {}",
                                gold, (best as u32) / 60, (best as u32) % 60, kills
                            ),
                            position: Vec2::new(260.0, 340.0),
                            size:     18.0,
                            color:    [200, 200, 200, 255],
                        });
                    }
                }
                return; // Title 모드에서는 인게임 HUD 그리지 않음
            }
            SurvivorMode::CharacterSelect => {
                let cursor_idx = world
                    .resource::<CharacterCursor>()
                    .map(|c| c.index)
                    .unwrap_or(0);
                let gold = world
                    .resource::<MetaSave>()
                    .map(|m| m.gold_total)
                    .unwrap_or(0);
                if let Some(q) = world.resource_mut::<TextQueue>() {
                    q.push(DrawText {
                        text:     "CHARACTER SELECT".to_string(),
                        position: Vec2::new(250.0, 30.0),
                        size:     32.0,
                        color:    [255, 220, 80, 255],
                    });
                    q.push(DrawText {
                        text:     format!("Gold: {}", gold),
                        position: Vec2::new(550.0, 30.0),
                        size:     20.0,
                        color:    [255, 255, 100, 255],
                    });
                    for (i, kind) in CharacterKind::ALL.iter().enumerate() {
                        let need     = kind.unlock_gold();
                        let unlocked = gold >= need;
                        let prefix   = if i == cursor_idx { ">" } else { " " };
                        let lock_str = if unlocked {
                            String::new()
                        } else {
                            format!("[LOCKED need {}]", need)
                        };
                        let color = if i == cursor_idx {
                            [255, 255, 80, 255]
                        } else if unlocked {
                            [200, 200, 200, 255]
                        } else {
                            [120, 120, 120, 255]
                        };
                        q.push(DrawText {
                            text:     format!("{} {} {}", prefix, kind.label(), lock_str),
                            position: Vec2::new(120.0, 100.0 + i as f32 * 32.0),
                            size:     18.0,
                            color,
                        });
                    }
                    q.push(DrawText {
                        text:     "W/S = navigate  ENTER = select  ESC = back".to_string(),
                        position: Vec2::new(150.0, 320.0),
                        size:     14.0,
                        color:    [180, 180, 180, 255],
                    });
                }
                return;
            }
            SurvivorMode::StageClear => {
                if let Some(q) = world.resource_mut::<TextQueue>() {
                    q.push(DrawText {
                        text:     "STAGE CLEAR!".to_string(),
                        position: Vec2::new(260.0, 200.0),
                        size:     56.0,
                        color:    [255, 220, 80, 255],
                    });
                    q.push(DrawText {
                        text:     "Press ENTER to return".to_string(),
                        position: Vec2::new(280.0, 290.0),
                        size:     22.0,
                        color:    [255, 255, 255, 255],
                    });
                }
                return;
            }
            SurvivorMode::Shop => {
                let meta_clone  = world.resource::<MetaSave>().cloned();
                let cursor_idx  = world.resource::<ShopCursor>().map(|c| c.index).unwrap_or(0);
                if let Some(q) = world.resource_mut::<TextQueue>() {
                    q.push(DrawText {
                        text:     "POWERUP SHOP".to_string(),
                        position: Vec2::new(280.0, 30.0),
                        size:     32.0,
                        color:    [255, 220, 80, 255],
                    });
                    if let Some(meta) = &meta_clone {
                        q.push(DrawText {
                            text:     format!("Gold: {}", meta.gold_total),
                            position: Vec2::new(550.0, 30.0),
                            size:     20.0,
                            color:    [255, 255, 100, 255],
                        });
                    }
                    // PowerUp 목록 (19종) — 현재 커서 위치 강조
                    for (i, kind) in PowerUpKind::ALL.iter().enumerate() {
                        let lv   = meta_clone.as_ref()
                            .map(|m| *m.powerup_levels.get(kind.key()).unwrap_or(&0))
                            .unwrap_or(0);
                        let cost = kind.cost(lv);
                        let prefix = if i == cursor_idx { ">" } else { " " };
                        let color  = if i == cursor_idx {
                            [255, 255, 80, 255]
                        } else {
                            [200, 200, 200, 255]
                        };
                        q.push(DrawText {
                            text: format!(
                                "{} {:<12} Lv {}/{}  cost {}",
                                prefix,
                                kind.key(),
                                lv,
                                kind.max_level(),
                                cost
                            ),
                            position: Vec2::new(80.0, 80.0 + i as f32 * 26.0),
                            size:     16.0,
                            color,
                        });
                    }
                    q.push(DrawText {
                        text:     "W/S = navigate  ENTER = buy  ESC = back".to_string(),
                        position: Vec2::new(150.0, 80.0 + PowerUpKind::ALL.len() as f32 * 26.0 + 20.0),
                        size:     14.0,
                        color:    [180, 180, 180, 255],
                    });
                }
                return;
            }
            SurvivorMode::InGame => {
                // 아래 기존 HUD 코드 실행
            }
        }

        // 1) Playing 중에만 timer 누적
        let state = world.resource::<GameState>().cloned().unwrap_or(GameState::Playing);
        if matches!(state, GameState::Playing) {
            if let Some(stats) = world.resource_mut::<GameStats>() {
                stats.elapsed += dt;
            }
        }

        // 2) Player 상태 캐시 (borrow 즉시 종료)
        let player_info = world
            .query2::<Player, Health>()
            .next()
            .map(|(_, _, h)| (h.current, h.max));
        let xp_info = world
            .query2::<Player, XpAccumulator>()
            .next()
            .map(|(_, _, acc)| (acc.current, acc.level, acc.next_threshold));
        let passive_count = world
            .query2::<Player, PassiveInventory>()
            .next()
            .map(|(_, _, inv)| inv.passives.len())
            .unwrap_or(0);

        let elapsed = world.resource::<GameStats>().map(|s| s.elapsed).unwrap_or(0.0);
        let kills   = world.resource::<GameStats>().map(|s| s.kills).unwrap_or(0);
        let gold    = world.resource::<GoldWallet>().map(|w| w.current).unwrap_or(0);
        let mm = (elapsed as u32) / 60;
        let ss = (elapsed as u32) % 60;

        // 3) 좌상단 HUD 한 줄 (800×600 viewport 기준 좌상단 (10, 10))
        if let (Some((hp, hp_max)), Some((xp, lv, xp_max))) = (player_info, xp_info) {
            let line = format!(
                "{:02}:{:02}  Lv {}  XP {}/{}  HP {:.0}/{:.0}  Passives {}  Gold {}  Kills {}",
                mm, ss, lv, xp, xp_max, hp, hp_max, passive_count, gold, kills
            );
            if let Some(q) = world.resource_mut::<TextQueue>() {
                q.push(DrawText {
                    text:     line,
                    position: Vec2::new(10.0, 10.0),
                    size:     18.0,
                    color:    [255, 255, 255, 255],
                });
            }
        }

        // 4) Paused + PendingLevelUp: 화면 중앙 카드 안내 (800×600 기준 중앙 x≈320)
        if matches!(state, GameState::Paused) {
            if let Some(p) = world.resource::<PendingLevelUp>() {
                if !p.consumed {
                    let offered = p.offered; // [CardKind; 3] 복사
                    if let Some(q) = world.resource_mut::<TextQueue>() {
                        q.push(DrawText {
                            text:     "LEVEL UP!".to_string(),
                            position: Vec2::new(320.0, 220.0),
                            size:     48.0,
                            color:    [255, 220, 80, 255],
                        });
                        q.push(DrawText {
                            text:     format!("1. {}", offered[0].label()),
                            position: Vec2::new(320.0, 290.0),
                            size:     22.0,
                            color:    [255, 255, 255, 255],
                        });
                        q.push(DrawText {
                            text:     format!("2. {}", offered[1].label()),
                            position: Vec2::new(320.0, 325.0),
                            size:     22.0,
                            color:    [255, 255, 255, 255],
                        });
                        q.push(DrawText {
                            text:     format!("3. {}", offered[2].label()),
                            position: Vec2::new(320.0, 360.0),
                            size:     22.0,
                            color:    [255, 255, 255, 255],
                        });
                    }
                }
            }
        }

        // 5) GameOver: 화면 중앙 사망 + 재시작 안내 (800×600 기준 중앙 x≈310)
        if matches!(state, GameState::GameOver) {
            if let Some(q) = world.resource_mut::<TextQueue>() {
                q.push(DrawText {
                    text:     "YOU DIED".to_string(),
                    position: Vec2::new(310.0, 250.0),
                    size:     56.0,
                    color:    [255, 60, 60, 255],
                });
                q.push(DrawText {
                    text:     "Press R to restart".to_string(),
                    position: Vec2::new(310.0, 325.0),
                    size:     22.0,
                    color:    [255, 255, 255, 255],
                });
            }
        }

        // 6) 보스 HP 바 — 보스가 존재하면 화면 상단 중앙에 표시
        let boss_info: Option<(super::boss::BossKind, f32, f32)> = world
            .query2::<Boss, Health>()
            .next()
            .map(|(_, b, h)| (b.kind, h.current, h.max));
        if let Some((kind, hp, max)) = boss_info {
            if let Some(q) = world.resource_mut::<TextQueue>() {
                q.push(DrawText {
                    text:     format!("{}  {:.0}/{:.0}", kind.label(), hp.max(0.0), max),
                    position: Vec2::new(220.0, 50.0),
                    size:     22.0,
                    color:    [255, 100, 100, 255],
                });
            }
        }

        // 7) StageClear 오버레이 — Phase 8-A: SurvivorMode::StageClear 로 전환 후 처리하므로
        //    InGame 모드에서는 표시하지 않음 (ModeTransitionSystem 이 다음 프레임에 전환).
        // (기존 cleared 체크 제거 — SurvivorMode 분기에서 처리됨)
    }
}
