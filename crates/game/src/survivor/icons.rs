use super::inventory::WeaponKind;
use super::passive::PassiveKind;

pub const ICONS_PATH: &str = "assets/textures/survivor/survivor_icons.png";
pub const ICON_COLS: u32 = 6;
pub const ICON_ROWS: u32 = 6;
pub const ICON_SIZE: f32 = 64.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IconFrame {
    pub col: u32,
    pub row: u32,
}

impl IconFrame {
    pub const fn new(index: u32) -> Self {
        Self {
            col: index % ICON_COLS,
            row: index / ICON_COLS,
        }
    }

    pub fn uv(self) -> [f32; 4] {
        let u = self.col as f32 / ICON_COLS as f32;
        let v = self.row as f32 / ICON_ROWS as f32;
        [u, v, 1.0 / ICON_COLS as f32, 1.0 / ICON_ROWS as f32]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiIcon {
    Weapon(&'static str),
    Passive(PassiveKind),
    Evolution(&'static str),
}

impl UiIcon {
    pub fn frame(self) -> IconFrame {
        IconFrame::new(match self {
            UiIcon::Weapon("Whip") => 0,
            UiIcon::Weapon("MagicWand") => 1,
            UiIcon::Weapon("Knife") => 2,
            UiIcon::Weapon("Axe") => 3,
            UiIcon::Weapon("Cross") => 4,
            UiIcon::Weapon("FireWand") => 5,
            UiIcon::Weapon("Garlic") => 6,
            UiIcon::Weapon("HolyWater") => 7,
            UiIcon::Weapon("KingBible") => 8,
            UiIcon::Weapon("LightningRing") => 9,
            UiIcon::Passive(PassiveKind::Spinach) => 10,
            UiIcon::Passive(PassiveKind::Armor) => 11,
            UiIcon::Passive(PassiveKind::HollowHeart) => 12,
            UiIcon::Passive(PassiveKind::Pummarola) => 13,
            UiIcon::Passive(PassiveKind::EmptyTome) => 14,
            UiIcon::Passive(PassiveKind::Candelabrador) => 15,
            UiIcon::Passive(PassiveKind::Bracer) => 16,
            UiIcon::Passive(PassiveKind::Spellbinder) => 17,
            UiIcon::Passive(PassiveKind::Duplicator) => 18,
            UiIcon::Passive(PassiveKind::Wings) => 19,
            UiIcon::Passive(PassiveKind::Attractorb) => 20,
            UiIcon::Passive(PassiveKind::Clover) => 21,
            UiIcon::Passive(PassiveKind::Crown) => 22,
            UiIcon::Passive(PassiveKind::StoneMask) => 23,
            UiIcon::Passive(PassiveKind::SkullOManiac) => 24,
            UiIcon::Passive(PassiveKind::Tiragisu) => 25,
            UiIcon::Evolution("Whip") => 26,
            UiIcon::Evolution("MagicWand") => 27,
            UiIcon::Evolution("Knife") => 28,
            UiIcon::Evolution("Axe") => 29,
            UiIcon::Evolution("Cross") => 30,
            UiIcon::Evolution("FireWand") => 31,
            UiIcon::Evolution("Garlic") => 32,
            UiIcon::Evolution("HolyWater") => 33,
            UiIcon::Weapon(_) | UiIcon::Evolution(_) => 35,
        })
    }
}

pub fn weapon_icon(kind: &WeaponKind) -> UiIcon {
    UiIcon::Weapon(match kind {
        WeaponKind::Whip { .. } => "Whip",
        WeaponKind::MagicWand { .. } => "MagicWand",
        WeaponKind::Knife { .. } => "Knife",
        WeaponKind::Axe { .. } => "Axe",
        WeaponKind::Cross { .. } => "Cross",
        WeaponKind::FireWand { .. } => "FireWand",
        WeaponKind::Garlic { .. } => "Garlic",
        WeaponKind::HolyWater { .. } => "HolyWater",
        WeaponKind::KingBible { .. } => "KingBible",
        WeaponKind::LightningRing { .. } => "LightningRing",
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::survivor::chest::EVOLUTION_RULES;
    use crate::survivor::data::load_starter_weapons;

    const ALL_PASSIVES: &[PassiveKind] = &[
        PassiveKind::Spinach,
        PassiveKind::Armor,
        PassiveKind::HollowHeart,
        PassiveKind::Pummarola,
        PassiveKind::EmptyTome,
        PassiveKind::Candelabrador,
        PassiveKind::Bracer,
        PassiveKind::Spellbinder,
        PassiveKind::Duplicator,
        PassiveKind::Wings,
        PassiveKind::Attractorb,
        PassiveKind::Clover,
        PassiveKind::Crown,
        PassiveKind::StoneMask,
        PassiveKind::SkullOManiac,
        PassiveKind::Tiragisu,
    ];

    #[test]
    fn all_starter_weapons_have_icons() {
        for slot in load_starter_weapons() {
            let frame = weapon_icon(&slot.kind).frame();
            assert!(frame.row < ICON_ROWS);
            assert!(frame.col < ICON_COLS);
        }
    }

    #[test]
    fn all_passives_have_icons() {
        for passive in ALL_PASSIVES {
            let frame = UiIcon::Passive(*passive).frame();
            assert!(frame.row < ICON_ROWS);
            assert!(frame.col < ICON_COLS);
        }
    }

    #[test]
    fn all_evolution_rules_have_icons() {
        for rule in EVOLUTION_RULES {
            let frame = UiIcon::Evolution(rule.weapon).frame();
            assert!(frame.row < ICON_ROWS);
            assert!(frame.col < ICON_COLS);
        }
    }
}
