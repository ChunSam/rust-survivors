use engine::{System, World};
use serde::{Deserialize, Serialize};

use super::boss::{Boss, StageProgress};
use super::character::CharacterKind;
use super::hud::GameStats;
use super::inventory::WeaponInventory;
use super::levelup::PendingLevelUp;
use super::locale::{loc, Lang};
use super::meta::MetaSave;
use super::passive::PassiveInventory;
use super::player::Player;
use super::powerup::PowerUpKind;
use super::stage::{SelectedStage, StageKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AchievementKind {
    FirstRun,
    Survive5Min,
    Survive10Min,
    Survive20Min,
    Kill100,
    Kill1000,
    Kill5000,
    ReachLevel10,
    ReachLevel20,
    FirstWeaponLv8,
    FirstEvolution,
    SixWeapons,
    SixPassives,
    Earn100Gold,
    Earn1000Gold,
    FirstStageClear,
    MadForestClear,
    InlaidLibraryClear,
    DairyPlantClear,
    DefeatBoss,
}

impl AchievementKind {
    pub const ALL: &'static [Self] = &[
        Self::FirstRun,
        Self::Survive5Min,
        Self::Survive10Min,
        Self::Survive20Min,
        Self::Kill100,
        Self::Kill1000,
        Self::Kill5000,
        Self::ReachLevel10,
        Self::ReachLevel20,
        Self::FirstWeaponLv8,
        Self::FirstEvolution,
        Self::SixWeapons,
        Self::SixPassives,
        Self::Earn100Gold,
        Self::Earn1000Gold,
        Self::FirstStageClear,
        Self::MadForestClear,
        Self::InlaidLibraryClear,
        Self::DairyPlantClear,
        Self::DefeatBoss,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Self::FirstRun => "FirstRun",
            Self::Survive5Min => "Survive5Min",
            Self::Survive10Min => "Survive10Min",
            Self::Survive20Min => "Survive20Min",
            Self::Kill100 => "Kill100",
            Self::Kill1000 => "Kill1000",
            Self::Kill5000 => "Kill5000",
            Self::ReachLevel10 => "ReachLevel10",
            Self::ReachLevel20 => "ReachLevel20",
            Self::FirstWeaponLv8 => "FirstWeaponLv8",
            Self::FirstEvolution => "FirstEvolution",
            Self::SixWeapons => "SixWeapons",
            Self::SixPassives => "SixPassives",
            Self::Earn100Gold => "Earn100Gold",
            Self::Earn1000Gold => "Earn1000Gold",
            Self::FirstStageClear => "FirstStageClear",
            Self::MadForestClear => "MadForestClear",
            Self::InlaidLibraryClear => "InlaidLibraryClear",
            Self::DairyPlantClear => "DairyPlantClear",
            Self::DefeatBoss => "DefeatBoss",
        }
    }

    pub fn title(self, lang: Lang) -> &'static str {
        match self {
            Self::FirstRun => loc(lang, "첫 출격", "First Run"),
            Self::Survive5Min => loc(lang, "5분 생존", "Survive 5 Minutes"),
            Self::Survive10Min => loc(lang, "10분 생존", "Survive 10 Minutes"),
            Self::Survive20Min => loc(lang, "20분 생존", "Survive 20 Minutes"),
            Self::Kill100 => loc(lang, "100 처치", "100 Kills"),
            Self::Kill1000 => loc(lang, "1000 처치", "1000 Kills"),
            Self::Kill5000 => loc(lang, "5000 처치", "5000 Kills"),
            Self::ReachLevel10 => loc(lang, "레벨 10", "Reach Level 10"),
            Self::ReachLevel20 => loc(lang, "레벨 20", "Reach Level 20"),
            Self::FirstWeaponLv8 => loc(lang, "첫 무기 완성", "First Level 8 Weapon"),
            Self::FirstEvolution => loc(lang, "첫 진화", "First Evolution"),
            Self::SixWeapons => loc(lang, "무기 6칸", "Six Weapons"),
            Self::SixPassives => loc(lang, "패시브 6칸", "Six Passives"),
            Self::Earn100Gold => loc(lang, "골드 100", "Earn 100 Gold"),
            Self::Earn1000Gold => loc(lang, "골드 1000", "Earn 1000 Gold"),
            Self::FirstStageClear => loc(lang, "첫 클리어", "First Stage Clear"),
            Self::MadForestClear => loc(lang, "광란의 숲 클리어", "Clear Mad Forest"),
            Self::InlaidLibraryClear => loc(lang, "상감 도서관 클리어", "Clear Inlaid Library"),
            Self::DairyPlantClear => loc(lang, "유제품 공장 클리어", "Clear Dairy Plant"),
            Self::DefeatBoss => loc(lang, "보스 격파", "Defeat a Boss"),
        }
    }

    pub fn reward(self, lang: Lang) -> &'static str {
        match self {
            Self::FirstRun => loc(lang, "보상: 재추첨 +1", "Reward: Reroll +1"),
            Self::Survive5Min => loc(lang, "보상: 이멜다 해금", "Reward: Unlock Imelda"),
            Self::Survive10Min => loc(lang, "보상: 파스콸리나 해금", "Reward: Unlock Pasqualina"),
            Self::Survive20Min => loc(lang, "보상: 건너뛰기 +1", "Reward: Skip +1"),
            Self::Kill100 => loc(lang, "보상: 공격력 +1", "Reward: Might +1"),
            Self::Kill1000 => loc(lang, "보상: 젠나로 해금", "Reward: Unlock Gennaro"),
            Self::Kill5000 => loc(lang, "보상: 아르카 해금", "Reward: Unlock Arca"),
            Self::ReachLevel10 => loc(lang, "보상: 성장 +1", "Reward: Growth +1"),
            Self::ReachLevel20 => loc(lang, "보상: 추방 +1", "Reward: Banish +1"),
            Self::FirstWeaponLv8 => loc(lang, "보상: 포르타 해금", "Reward: Unlock Porta"),
            Self::FirstEvolution => loc(lang, "보상: 재추첨 +1", "Reward: Reroll +1"),
            Self::SixWeapons => loc(lang, "보상: 수량 +1", "Reward: Amount +1"),
            Self::SixPassives => loc(lang, "보상: 지속 시간 +1", "Reward: Duration +1"),
            Self::Earn100Gold => loc(lang, "보상: 탐욕 +1", "Reward: Greed +1"),
            Self::Earn1000Gold => loc(lang, "보상: 운 +1", "Reward: Luck +1"),
            Self::FirstStageClear => loc(lang, "보상: 상감 도서관 해금", "Reward: Unlock Library"),
            Self::MadForestClear => loc(lang, "보상: 상감 도서관 해금", "Reward: Unlock Library"),
            Self::InlaidLibraryClear => {
                loc(lang, "보상: 유제품 공장 해금", "Reward: Unlock Dairy Plant")
            }
            Self::DairyPlantClear => loc(lang, "보상: 쿨다운 +1", "Reward: Cooldown +1"),
            Self::DefeatBoss => loc(lang, "보상: 방어력 +1", "Reward: Armor +1"),
        }
    }
}

pub struct AchievementSystem;

impl System for AchievementSystem {
    fn run(&mut self, world: &mut World, _dt: f32) {
        let mut completed = Vec::new();

        let stats = world
            .resource::<GameStats>()
            .map(|s| (s.elapsed, s.kills))
            .unwrap_or((0.0, 0));
        let stage = world
            .resource::<SelectedStage>()
            .copied()
            .unwrap_or_default()
            .0;
        let stage_cleared = world
            .resource::<StageProgress>()
            .map(|p| p.cleared)
            .unwrap_or(false);
        let boss_active = world.query::<Boss>().next().is_some();
        let player_level = world
            .query2::<Player, super::xp::XpAccumulator>()
            .next()
            .map(|(_, _, xp)| xp.level)
            .unwrap_or(1);
        let weapon_info = world
            .query2::<Player, WeaponInventory>()
            .next()
            .map(|(_, _, inv)| {
                (
                    inv.slots.len(),
                    inv.slots.iter().any(|s| s.level >= 8),
                    inv.slots.iter().any(|s| s.evolved),
                )
            })
            .unwrap_or((0, false, false));
        let passive_count = world
            .query2::<Player, PassiveInventory>()
            .next()
            .map(|(_, _, inv)| inv.passives.len())
            .unwrap_or(0);
        let pending_levelup = world.resource::<PendingLevelUp>().is_some();

        let meta_snapshot = world.resource::<MetaSave>().cloned().unwrap_or_default();
        let total_kills = meta_snapshot.kills_total.saturating_add(stats.1);
        let total_gold = meta_snapshot.gold_total;
        let best_or_current = meta_snapshot.best_time.max(stats.0);

        if stats.0 > 0.0 {
            completed.push(AchievementKind::FirstRun);
        }
        if best_or_current >= 300.0 {
            completed.push(AchievementKind::Survive5Min);
        }
        if best_or_current >= 600.0 {
            completed.push(AchievementKind::Survive10Min);
        }
        if best_or_current >= 1200.0 {
            completed.push(AchievementKind::Survive20Min);
        }
        if total_kills >= 100 {
            completed.push(AchievementKind::Kill100);
        }
        if total_kills >= 1000 {
            completed.push(AchievementKind::Kill1000);
        }
        if total_kills >= 5000 {
            completed.push(AchievementKind::Kill5000);
        }
        if player_level >= 10 || pending_levelup && player_level >= 9 {
            completed.push(AchievementKind::ReachLevel10);
        }
        if player_level >= 20 || pending_levelup && player_level >= 19 {
            completed.push(AchievementKind::ReachLevel20);
        }
        if weapon_info.1 {
            completed.push(AchievementKind::FirstWeaponLv8);
        }
        if weapon_info.2 {
            completed.push(AchievementKind::FirstEvolution);
        }
        if weapon_info.0 >= 6 {
            completed.push(AchievementKind::SixWeapons);
        }
        if passive_count >= 6 {
            completed.push(AchievementKind::SixPassives);
        }
        if total_gold >= 100 {
            completed.push(AchievementKind::Earn100Gold);
        }
        if total_gold >= 1000 {
            completed.push(AchievementKind::Earn1000Gold);
        }
        if meta_snapshot.best_time > 0.0 || stage_cleared {
            completed.push(AchievementKind::FirstStageClear);
        }
        if stage_cleared && stage == StageKind::MadForest {
            completed.push(AchievementKind::MadForestClear);
        }
        if stage_cleared && stage == StageKind::InlaidLibrary {
            completed.push(AchievementKind::InlaidLibraryClear);
        }
        if stage_cleared && stage == StageKind::DairyPlant {
            completed.push(AchievementKind::DairyPlantClear);
        }
        if !boss_active && stats.0 >= 600.0 && stats.1 > 0 {
            completed.push(AchievementKind::DefeatBoss);
        }

        for kind in completed {
            unlock_achievement(world, kind);
        }
    }
}

pub fn unlock_achievement(world: &mut World, kind: AchievementKind) -> bool {
    let key = kind.key().to_string();
    let mut unlocked = false;
    if let Some(meta) = world.resource_mut::<MetaSave>() {
        if meta.achievements.iter().any(|a| a == &key) {
            return false;
        }
        meta.achievements.push(key);
        apply_reward(meta, kind);
        meta.save_to_disk();
        unlocked = true;
    }
    unlocked
}

fn apply_reward(meta: &mut MetaSave, kind: AchievementKind) {
    match kind {
        AchievementKind::Survive5Min => unlock_char(meta, CharacterKind::Imelda),
        AchievementKind::Survive10Min => unlock_char(meta, CharacterKind::Pasqualina),
        AchievementKind::Kill1000 => unlock_char(meta, CharacterKind::Gennaro),
        AchievementKind::Kill5000 => unlock_char(meta, CharacterKind::Arca),
        AchievementKind::FirstWeaponLv8 => unlock_char(meta, CharacterKind::Porta),
        AchievementKind::FirstStageClear | AchievementKind::MadForestClear => {
            unlock_stage(meta, StageKind::InlaidLibrary)
        }
        AchievementKind::InlaidLibraryClear => unlock_stage(meta, StageKind::DairyPlant),
        AchievementKind::FirstRun | AchievementKind::FirstEvolution => {
            grant_powerup(meta, PowerUpKind::Reroll)
        }
        AchievementKind::Survive20Min => grant_powerup(meta, PowerUpKind::Skip),
        AchievementKind::Kill100 => grant_powerup(meta, PowerUpKind::Might),
        AchievementKind::ReachLevel10 => grant_powerup(meta, PowerUpKind::Growth),
        AchievementKind::ReachLevel20 => grant_powerup(meta, PowerUpKind::Banish),
        AchievementKind::SixWeapons => grant_powerup(meta, PowerUpKind::Amount),
        AchievementKind::SixPassives => grant_powerup(meta, PowerUpKind::Duration),
        AchievementKind::Earn100Gold => grant_powerup(meta, PowerUpKind::Greed),
        AchievementKind::Earn1000Gold => grant_powerup(meta, PowerUpKind::Luck),
        AchievementKind::DairyPlantClear => grant_powerup(meta, PowerUpKind::Cooldown),
        AchievementKind::DefeatBoss => grant_powerup(meta, PowerUpKind::Armor),
    }
}

fn unlock_char(meta: &mut MetaSave, kind: CharacterKind) {
    let key = kind.key().to_string();
    if !meta.unlocked_chars.iter().any(|c| c == &key) {
        meta.unlocked_chars.push(key);
    }
}

fn unlock_stage(meta: &mut MetaSave, kind: StageKind) {
    let key = kind.key().to_string();
    if !meta.unlocked_stages.iter().any(|s| s == &key) {
        meta.unlocked_stages.push(key);
    }
}

fn grant_powerup(meta: &mut MetaSave, kind: PowerUpKind) {
    let key = kind.key().to_string();
    let current = *meta.powerup_levels.get(&key).unwrap_or(&0);
    if current < kind.max_level() {
        meta.powerup_levels.insert(key, current + 1);
    }
}

pub fn achievement_completed(meta: &MetaSave, kind: AchievementKind) -> bool {
    let key = kind.key();
    meta.achievements.iter().any(|a| a == key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine::World;

    #[test]
    fn achievement_count_is_twenty() {
        assert_eq!(AchievementKind::ALL.len(), 20);
    }

    #[test]
    fn unlock_is_not_duplicated() {
        let mut world = World::new();
        world.insert_resource(MetaSave::default());

        assert!(unlock_achievement(&mut world, AchievementKind::Kill100));
        assert!(!unlock_achievement(&mut world, AchievementKind::Kill100));

        let meta = world.resource::<MetaSave>().unwrap();
        assert_eq!(meta.achievements.len(), 1);
        assert_eq!(meta.powerup_levels.get("Might"), Some(&1));
    }

    #[test]
    fn survive_5min_unlocks_imelda() {
        let mut world = World::new();
        world.insert_resource(MetaSave::default());

        assert!(unlock_achievement(&mut world, AchievementKind::Survive5Min));

        let meta = world.resource::<MetaSave>().unwrap();
        assert!(meta.unlocked_chars.iter().any(|c| c == "Imelda"));
    }

    #[test]
    fn first_stage_clear_unlocks_library() {
        let mut world = World::new();
        world.insert_resource(MetaSave::default());

        assert!(unlock_achievement(
            &mut world,
            AchievementKind::FirstStageClear
        ));

        let meta = world.resource::<MetaSave>().unwrap();
        assert!(meta.unlocked_stages.iter().any(|s| s == "InlaidLibrary"));
    }

    #[test]
    fn completed_lookup_uses_key() {
        let meta = MetaSave {
            achievements: vec!["FirstRun".to_string()],
            ..MetaSave::default()
        };
        assert!(achievement_completed(&meta, AchievementKind::FirstRun));
        assert!(!achievement_completed(&meta, AchievementKind::Kill100));
    }
}
