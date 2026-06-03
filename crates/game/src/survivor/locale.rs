/// Phase 11 로컬라이제이션 — 한국어 / 영어 지원.
///
/// `MetaSave.language_setting` 이 System/Ko/En 중 하나를 저장하고, UI 는
/// `text(lang, UiText::...)` 또는 도메인별 `label(lang)` 함수로 문자열을 고른다.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Lang {
    #[default]
    Ko,
    En,
}

impl Lang {
    pub fn toggle(self) -> Self {
        match self {
            Lang::Ko => Lang::En,
            Lang::En => Lang::Ko,
        }
    }
}

#[inline]
pub fn loc(lang: Lang, ko: &'static str, en: &'static str) -> &'static str {
    match lang {
        Lang::Ko => ko,
        Lang::En => en,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiText {
    GameTitle,
    PressEnterStart,
    TitleMenuHelp,
    CharacterSelect,
    StageSelect,
    PowerupShop,
    Settings,
    Achievements,
    Gold,
    Best,
    Kills,
    Time,
    Level,
    Lv,
    Hp,
    Xp,
    Might,
    Cooldown,
    Area,
    Amount,
    MoveSpeed,
    CostPrefix,
    Locked,
    NavigateSelectBack,
    NavigateBuyBack,
    NavigateChangeApplyBack,
    PauseHelp,
    Paused,
    Resume,
    ReturnToTitle,
    QuitGame,
    StageClear,
    PressEnterReturn,
    LevelUp,
    LevelUpActions,
    GameOver,
    RestartHint,
    Weapons,
    Passives,
    EmptySlot,
    Language,
    HudDetail,
    BgmVolume,
    SfxVolume,
    Resolution,
}

impl UiText {
    pub const ALL: &'static [Self] = &[
        Self::GameTitle,
        Self::PressEnterStart,
        Self::TitleMenuHelp,
        Self::CharacterSelect,
        Self::StageSelect,
        Self::PowerupShop,
        Self::Settings,
        Self::Achievements,
        Self::Gold,
        Self::Best,
        Self::Kills,
        Self::Time,
        Self::Level,
        Self::Lv,
        Self::Hp,
        Self::Xp,
        Self::Might,
        Self::Cooldown,
        Self::Area,
        Self::Amount,
        Self::MoveSpeed,
        Self::CostPrefix,
        Self::Locked,
        Self::NavigateSelectBack,
        Self::NavigateBuyBack,
        Self::NavigateChangeApplyBack,
        Self::PauseHelp,
        Self::Paused,
        Self::Resume,
        Self::ReturnToTitle,
        Self::QuitGame,
        Self::StageClear,
        Self::PressEnterReturn,
        Self::LevelUp,
        Self::LevelUpActions,
        Self::GameOver,
        Self::RestartHint,
        Self::Weapons,
        Self::Passives,
        Self::EmptySlot,
        Self::Language,
        Self::HudDetail,
        Self::BgmVolume,
        Self::SfxVolume,
        Self::Resolution,
    ];
}

pub fn text(lang: Lang, key: UiText) -> &'static str {
    match key {
        UiText::GameTitle => loc(lang, "러스트 서바이버즈", "RUST SURVIVORS"),
        UiText::PressEnterStart => loc(lang, "엔터 키로 시작", "Press ENTER to start"),
        UiText::TitleMenuHelp => loc(
            lang,
            "C = 캐릭터  T = 스테이지  S = 상점  O = 설정",
            "C = CHAR  T = STAGE  S = SHOP  O = SETTINGS",
        ),
        UiText::CharacterSelect => loc(lang, "캐릭터 선택", "CHARACTER SELECT"),
        UiText::StageSelect => loc(lang, "스테이지 선택", "STAGE SELECT"),
        UiText::PowerupShop => loc(lang, "파워업 상점", "POWERUP SHOP"),
        UiText::Settings => loc(lang, "설정", "SETTINGS"),
        UiText::Achievements => loc(lang, "업적", "Achievements"),
        UiText::Gold => loc(lang, "골드:", "Gold:"),
        UiText::Best => loc(lang, "최고", "Best"),
        UiText::Kills => loc(lang, "처치", "Kills"),
        UiText::Time => loc(lang, "시간", "Time"),
        UiText::Level => loc(lang, "레벨", "Level"),
        UiText::Lv => loc(lang, "Lv", "Lv"),
        UiText::Hp => loc(lang, "HP", "HP"),
        UiText::Xp => loc(lang, "XP", "XP"),
        UiText::Might => loc(lang, "공격", "Might"),
        UiText::Cooldown => loc(lang, "쿨다운", "Cooldown"),
        UiText::Area => loc(lang, "범위", "Area"),
        UiText::Amount => loc(lang, "투사체", "Amount"),
        UiText::MoveSpeed => loc(lang, "이동", "Move"),
        UiText::CostPrefix => loc(lang, "비용 ", "cost "),
        UiText::Locked => loc(lang, "잠김", "LOCKED"),
        UiText::NavigateSelectBack => loc(
            lang,
            "W/S = 이동  엔터 = 선택  ESC = 뒤로",
            "W/S = navigate  ENTER = select  ESC = back",
        ),
        UiText::NavigateBuyBack => loc(
            lang,
            "W/S = 이동  엔터 = 구매  ESC = 뒤로",
            "W/S = navigate  ENTER = buy  ESC = back",
        ),
        UiText::NavigateChangeApplyBack => loc(
            lang,
            "W/S = 이동  A/D = 변경  엔터 = 해상도 적용  ESC = 뒤로",
            "W/S = navigate  A/D = change  ENTER = apply resolution  ESC = back",
        ),
        UiText::PauseHelp => loc(
            lang,
            "W/S = 이동  엔터 = 선택  ESC = 재개",
            "W/S = navigate  ENTER = select  ESC = resume",
        ),
        UiText::Paused => loc(lang, "일시 정지", "PAUSED"),
        UiText::Resume => loc(lang, "계속하기", "Resume"),
        UiText::ReturnToTitle => loc(lang, "타이틀로 돌아가기", "Return to Title"),
        UiText::QuitGame => loc(lang, "게임 종료", "Quit Game"),
        UiText::StageClear => loc(lang, "스테이지 클리어!", "STAGE CLEAR!"),
        UiText::PressEnterReturn => loc(lang, "엔터 키로 돌아가기", "Press ENTER to return"),
        UiText::LevelUp => loc(lang, "레벨 업!", "LEVEL UP!"),
        UiText::LevelUpActions => loc(
            lang,
            "R 재추첨 {reroll}  S 건너뛰기 {skip}  4/5/6 추방 {banish}",
            "R Reroll {reroll}  S Skip {skip}  4/5/6 Banish {banish}",
        ),
        UiText::GameOver => loc(lang, "사망", "YOU DIED"),
        UiText::RestartHint => loc(lang, "R 키로 재시작", "Press R to restart"),
        UiText::Weapons => loc(lang, "무기", "Weapons"),
        UiText::Passives => loc(lang, "패시브", "Passives"),
        UiText::EmptySlot => loc(lang, "빈 슬롯", "Empty"),
        UiText::Language => loc(lang, "언어", "Language"),
        UiText::HudDetail => loc(lang, "HUD 정보량", "HUD Detail"),
        UiText::BgmVolume => loc(lang, "BGM 볼륨", "BGM Volume"),
        UiText::SfxVolume => loc(lang, "SFX 볼륨", "SFX Volume"),
        UiText::Resolution => loc(lang, "해상도", "Resolution"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ui_text_has_translations_for_both_languages() {
        for key in UiText::ALL {
            assert!(!text(Lang::Ko, *key).is_empty());
            assert!(!text(Lang::En, *key).is_empty());
        }
    }
}
