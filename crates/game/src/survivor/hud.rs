use engine::{System, World};
use engine::components::GameState;
use engine::renderer::text::{DrawText, TextQueue};
use glam::Vec2;
use super::player::Player;
use super::health::Health;
use super::xp::XpAccumulator;
use super::levelup::PendingLevelUp;
use super::passive::PassiveInventory;

/// 게임 진행 통계. 매 프레임 누적/조회.
#[derive(Debug, Default)]
pub struct GameStats {
    pub elapsed: f32, // 누적 게임 시간(초). Playing 중에만 증가.
    pub kills:   u32, // 누적 처치 수.
}

pub struct HudSystem;

impl System for HudSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
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
        let mm = (elapsed as u32) / 60;
        let ss = (elapsed as u32) % 60;

        // 3) 좌상단 HUD 한 줄 (800×600 viewport 기준 좌상단 (10, 10))
        if let (Some((hp, hp_max)), Some((xp, lv, xp_max))) = (player_info, xp_info) {
            let line = format!(
                "{:02}:{:02}  Lv {}  XP {}/{}  HP {:.0}/{:.0}  Passives {}  Kills {}",
                mm, ss, lv, xp, xp_max, hp, hp_max, passive_count, kills
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
    }
}
